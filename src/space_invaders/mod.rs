extern crate typenum;

use crate::cpu::intel8080::Intel8080;
use crate::cpu::instructions::rst;

use sfml::window::Key;
use sfml::graphics::*;
use sfml::window::*;

use bit_array::BitArray;
use typenum::U64;

const SCREEN_DIM : usize = 256 * 224 * 1;

pub struct SpaceInvadersMachine {
    pub state: Intel8080,
    pub last_timer: f64,
    pub next_interrupt: f64,

    pub lsb_shift: u8,
    pub msb_shift: u8,
    pub shift_offset: u8
}

impl SpaceInvadersMachine {
    pub fn new(state: Intel8080) -> SpaceInvadersMachine {
        SpaceInvadersMachine {
            state: state,
            last_timer: 0.0,
            next_interrupt: 0.0,
            lsb_shift: 0,
            msb_shift: 0,
            shift_offset: 0
        }
    }

    /// The initialization code will make a new 8080 emulator object 
    /// and read the ROM file into its RAM.
    pub fn init(&mut self, file_name: &str) {
        self.state.load_program(file_name);
    }

    /// returns the location of the video memory in the machine. 
    /// The platform code needs it for drawing to the window.
    /// 
    pub fn frame_buffer(&self) -> &[u8] {
        &self.state.memory[0x2400..=0x3fff]
    }

    /// starts the CPU running.
    pub fn start_emulation(&mut self) {
        // &self.state.run();

        let mut window = RenderWindow::new(VideoMode::new(256, 224, 1), "SPACE INVADERS", Style::DEFAULT, &Default::default());
        window.set_vertical_sync_enabled(true);
        
        let pixels = {
            let p = [200_u8; SCREEN_DIM / 8];
            p
        };

        let mut i = 0_usize;
        let mut b_array = Vec::new();
        while i < SCREEN_DIM / 8 {
            let bv = BitArray::<u64, U64>::from_bytes(&pixels[i .. i+8]);
            for b in &bv {
                b_array.push(b);
            }
            i += 8;
        }

        let mut image = Image::new(256, 224);
        for x in 0 .. 256 {
            for y in 0 .. 224 {
                let col = if b_array[224 * x + y] { &Color::WHITE } else { &Color::BLACK };
                image.set_pixel(x as u32, y as u32, col);
            }
        }
        
        let texture = Texture::from_image(&image).unwrap();
        
        while window.is_open() {
            while let Some(event) = window.poll_event() {
                match event {
                    Event::Closed => {window.close(); return; },
                    Event::KeyPressed { code, .. } => {
                        match code {
                            Key::Space => {
                                println!("space key was pressed");
                                self.key_pressed(Key::Space);
                            },
                            Key::Left => {
                                println!("Left arrow key was pressed");
                                self.key_pressed(Key::Left);
                            },
                            Key::Right => {
                                println!("Right arrow key was pressed");
                                self.key_pressed(Key::Right);
                            },
                            _ => {}
                        }
                    },
                    Event::KeyReleased { code, .. } => {
                        match code {
                            Key::Space => {
                                println!("space key was released");
                                self.key_released(Key::Space);
                            },
                            Key::Left => {
                                println!("Left arrow key was released"); 
                                self.key_released(Key::Left);

                                // enable interrupts
                                self.state.int_enable = 1;

                                // call appropraite interrupt service routine to read 
                                // the keyboard input
                                rst(&mut self.state, 3);
                            },
                            Key::Right => {
                                println!("Right arrow key was released"); 
                                self.key_released(Key::Right);
                            },
                            _ => {}
                        }
                    }
                    _ => {
                        let mut sprite = Sprite::new();
                        sprite.set_texture(&texture, false);

                        // window.clear(&Color::BLACK);
                        window.draw(&sprite);
                        window.display();
                    }
                }
            }
        }
        
    }

    // fn timeusec(&self) -> f64 {
    //     0.0
    // }

    pub fn key_pressed(&mut self, key:Key) {
        match key {
            Key::C => self.state.io_port.read_1 |= 0x01,      // Coin
            Key::Numpad1 => self.state.io_port.read_1 |= 0x04, // Player 1 start
            Key::Left => self.state.io_port.read_1 |= 0x20,   // Player 1 Left.  Set bit 5 of port 1
            Key::Right => self.state.io_port.read_1 |= 0x40,  // Player 1 Right. Set bit 6 of port 1
            Key::Space => self.state.io_port.read_1 |= 0x10,  // Player 1 Fire.  Set bit 4 of port 1
            _ => {}
        }
            
    }

    pub fn key_released(&mut self, key: Key) {
        match key {
            Key::C => self.state.io_port.read_1 &= !0x01,      // Coin
            Key::Left => self.state.io_port.read_1 &= !0x20,    // Clear bit 5 of port 1 
            Key::Right => self.state.io_port.read_1 &= !0x40,   // Clear bit 6 of port 1 
            Key::Space => self.state.io_port.read_1 &= !0x10,   // Clear bit 4 of port 1
            Key::Numpad1 => self.state.io_port.read_1 &= !0x04,  // Player 1 start
            _ => {}
        }
    }

    pub fn read_in(&self, port: u8) -> u8 {
        match port {
            0 => 1,
            1 => 0,
            3 => {
                let v = ((self.msb_shift as u16) << 8) | self.lsb_shift as u16;    
                let a = (v >> (8 - self.shift_offset)) & 0xff;
                a as u8
            },
            _ => 0
        }
    }

    pub fn write_out(&mut self, port: u8, value: u8) {
        match port {
            2 => self.shift_offset = value & 0x7,
            4 => {
                self.lsb_shift = self.msb_shift;
                self.msb_shift = value;
            },
            _ => {}
        }
    }
}