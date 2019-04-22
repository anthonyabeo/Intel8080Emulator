pub mod cpu {
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
                a: 0_u8,
                b: 0_u8,
                c: 0_u8,
                d: 0_u8,
                e: 0_u8,
                h: 0_u8,
                l: 0_u8
            }
        }
    }
}


pub mod intel8080 {
    use crate::cpu::{ConditionFlags, Register};


    pub struct Intel8080 {
        pub regs: Register,
        pub flags: ConditionFlags,
        pub pc: usize,
        pub sp: usize,
        pub memory: Vec<u8>
    }

    impl Intel8080 {
        pub fn new() -> Self {
            Intel8080 {
                regs: Register::new(),
                flags: ConditionFlags::new(),
                pc: 0_usize,
                sp: 0_usize,
                memory: vec![0_u8; 0xFFFF]
            }
        }

        pub fn emulate(&self) {
            println!("Running Emulation");
        }
    }
}