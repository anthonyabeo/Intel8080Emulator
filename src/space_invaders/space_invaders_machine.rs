use cpu::intel8080::Intel8080;


pub struct SpaceInvadersMachine {
    pub state: Intel8080;
    pub last_timer: f64;
    pub next_interrupt: f64;

    pub lsb_shift: u8;
    pub msb_shift: u8;
    pub shift_offset: u8;
}

impl SpaceInvadersMachine {
    pub new(&state: Intel8080) -> SpaceInvadersMachine {
        self.state = state;
        self.last_timer = 0.0;
        self.next_interrupt = 0.0;
        self.lsb_shift = 0;
        self.msb_shift = 0;
        self.shift_offset = 0;
    }

    pub key_pressed(&self, Key key) {
        match key {
            Key::Left => self.io_port.read_1 |= 0x20;   // Set bit 5 of port 1
            Key::Right => self.io_port.read_1 |= 0x40;  // Set bit 6 of port 1
            Key::Space => self.io_port.read_1 |= 0x16;  // Set bit 4 of port 1
            _ => {}
        }
            
    }

    pub key_released(&self, Key key) {
        match key {
            Key::Left => self.io_port.read_1 &= 0xDF    // Clear bit 5 of port 1 
            Key::Right => self.io_port.read_1 &= 0xBF   // Clear bit 6 of port 1 
            Key::Space => self.io_port.read_1 &= 0xEF   // Clear bit 4 of port 1
            _ => {}
        }
    }

    pub read_in(&self, port: u8) -> u8 {
        match port {
            0 => 1,
            1 => 0,
            3 => {
                u16 v = (self.msb_shift << 8) | self.lsb_shift;    
                a = ((v >> (8 - self.shift_offset)) & 0xff);
            }
        }

        a
    }

    pub write_out(&self, port: u8, value: u8) {
        match port {
            2 => self.shift_offset = value & 0x7; 
            4 => {
                self.lsb_shift = self.msb_shift;
                self.msb_shift = value;
            }
        }
    }
}