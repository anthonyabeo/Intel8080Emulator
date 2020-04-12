use sdl2::keyboard::Keycode;

pub struct SpaceInvadersMachine {
    pub last_timer: f64,
    pub next_interrupt: f64,

    pub lsb_shift: u8,
    pub msb_shift: u8,
    pub shift_offset: u8,

    port: u16
}

impl SpaceInvadersMachine {
    pub fn new() -> SpaceInvadersMachine {
        SpaceInvadersMachine {
            last_timer: 0.0,
            next_interrupt: 0.0,
            lsb_shift: 0,
            msb_shift: 0,
            shift_offset: 0,
            port: 0
        }
    }

    pub fn key_pressed(&mut self, key:Keycode) {
        match key {
            Keycode::C => self.port |= 0x01,      // Coin
            Keycode::Num1 => self.port |= 0x04, // Player 1 start
            Keycode::Left => self.port |= 0x20,   // Player 1 Left.  Set bit 5 of port 1
            Keycode::Right => self.port |= 0x40,  // Player 1 Right. Set bit 6 of port 1
            Keycode::Space => self.port |= 0x10,  // Player 1 Fire.  Set bit 4 of port 1
            _ => {}
        }
            
    }

    pub fn key_released(&mut self, key: Keycode) {
        match key {
            Keycode::C => self.port &= !0x01,      // Coin
            Keycode::Left => self.port &= !0x20,    // Clear bit 5 of port 1 
            Keycode::Right => self.port &= !0x40,   // Clear bit 6 of port 1 
            Keycode::Space => self.port &= !0x10,   // Clear bit 4 of port 1
            Keycode::Num1 => self.port &= !0x04,  // Player 1 start
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