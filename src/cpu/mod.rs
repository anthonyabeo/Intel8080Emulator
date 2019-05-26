pub mod instructions;

pub mod utils {
    pub fn parity(result: u16, size: usize) -> u8 {
        let mut counter = 0;
        let mut res = result as u8;
        for _ in 0..size {
            if (res & 0x01) == 1 { counter += 1; }
            res >>= 1;
        }
        
        ((counter & 0x01) == 0) as u8
    }
}

pub struct ConditionFlags {
    pub carry: u8,
    pub aux_carry: u8,
    pub zero: u8,
    pub parity: u8,
    pub sign: u8
}

impl ConditionFlags {
    pub fn new() -> ConditionFlags {
        ConditionFlags {
            carry: 0_u8,
            aux_carry: 0_u8,
            zero: 0_u8,
            parity: 0_u8,
            sign: 0_u8
        }
    }
}

pub struct Register {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8
}

impl Register {
    pub fn new() -> Register {
        Register {
            a: 0_u8, b: 0_u8, c: 0_u8, 
            d: 0_u8, e: 0_u8, h: 0_u8, 
            l: 0_u8
        }
    }
}