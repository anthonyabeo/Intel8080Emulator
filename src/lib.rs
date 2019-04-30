#[cfg(test)]
pub mod tests;

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
                a: 0_u8, b: 0_u8, c: 0_u8, 
                d: 0_u8, e: 0_u8, h: 0_u8, 
                l: 0_u8
            }
        }
    }
}


pub mod intel8080 {
    use std::fs::File;
    use std::path::Path;
    use std::io::Read;

    use crate::cpu::{ConditionFlags, Register};

    pub struct Intel8080 {
        pub regs: Register,
        pub flags: ConditionFlags,
        pub pc: usize,
        pub sp: usize,
        pub int_enable: u8,
        pub memory: Vec<u8>
    }

    impl Intel8080 {
        pub fn new() -> Self {
            Intel8080 {
                regs: Register::new(),
                flags: ConditionFlags::new(),
                pc: 0_usize,
                sp: 0_usize,
                int_enable: 0,
                memory: vec![0_u8; 0xFFFF] // 65 KB of Memory
            }
        }

        pub fn load_game_rom(&mut self, file_name: &str) {
            let mut f = match File::open(Path::new(file_name)) {
                Ok(file) => file,
                Err(e) => panic!("Could not open file - {}", e)
            };

            f.read(&mut self.memory).unwrap();
            
        }
        
        pub fn emulate(&mut self) {
            while self.memory[self.pc] != 0x76 { // while opcode != HLT (0x76)
                match self.memory[self.pc] {
                    0x00 => { self.pc += 1; } // NOP
                    0x01 => { 
                        // INSTRUCTION: LXI B
                        // DESCRIPTION: 
                        //      Load 2 next bytes into registers B and C. 

                        // load bytes into register B and C
                        self.regs.b = self.memory[self.pc + 2];
                        self.regs.c = self.memory[self.pc + 1];

                        self.pc += 3;
                    }
                    0x02 => { 
                        // INSTRUCTION: STAX B
                        // DESCRIPTION: 
                        //      The contents of the accumulator are stored in the memory 
                        //      location addressed by register pair BC

                        // get the content of register pair B and C
                        // format them into an address in LE format.
                        let addr = (((self.regs.b as u16) << 8) | 
                                    (self.regs.c as u16)) as usize;

                        // get the value in the A register and store this
                        // value at the address created in the previous step.
                        self.memory[addr] = self.regs.a;

                        self.pc += 1;
                    }
                    0x03 => { 
                        // INSTRUCTION: INX B
                        // DESCRIPTION: 
                        //      The 16-bit number held in register pair BC is incremented by one.

                        // get the content of register pair B and C format them into 
                        // an address in LE format and increment the value from the 
                        // previous step by one. 
                        let value = (((self.regs.b as u16) << 8) | (self.regs.c as u16)) + 1;

                        // split the new value into two. The LO byte is assigned to
                        // register C and the HO byte is assigned to register B. 
                        self.regs.b = ((value & 0xff00) >> 8) as u8;
                        self.regs.c = (value & 0x00ff) as u8;

                        self.pc += 1;
                    }
                    0x04 => { 
                        // INSTRUCTION: INR B
                        // DESCRIPTION: 
                        //      Increment register B by 1;

                        // increment the value in register B by 1.
                        let result = self.regs.b + 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register B with the result of the computation
                        self.regs.b = result;

                        self.pc += 1;
                    }
                    0x05 => {
                        // INSTRUCTION: DCR B
                        // DESCRIPTION:
                        //      The value in register B is decremented by 1;

                        // decrement the value in register B by 1.
                        let result = self.regs.b - 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register B with the result of the computation
                        self.regs.b = result;

                        self.pc += 1;
                    }
                    0x06 => { 
                        // INSTRUCTION: MVI B
                        // DESCRIPTION:
                        //      the immediate data byte is stored in register B. 
                        //      No condition flags are affected. 

                        // load the next byte into register B
                        self.regs.b = self.memory[self.pc + 1];

                        self.pc += 2;
                    }
                    0x07 => { 
                        // INSTRUCTION: RLC
                        // DESCRIPTION:
                        //      The contents of the accumulator are rotated one bit position to 
                        //      the left, with the high-order bit being transferred to the 
                        //      low-order bit position of the accumulator.

                        // compute carry and use it to compute the new value to
                        // be assigned to the accumulator (A) register.
                        let carry = ((self.regs.a & 0x80) >> 7) as u8;
                        self.regs.a = ((self.regs.a << 1) | carry) as u8;

                        // The Carry bit is set equal to the high-order bit of the accumulator.
                        self.flags.carry = carry;

                        self.pc += 1;
                    }
                    0x08 => { self.pc += 1; }
                    0x09 => { 
                        // INSTRUCTION: DAD B
                        // DESCRIPTION:
                        //      The 16-bit number in the specified register pair is added to the 
                        //      16-bit number held in the H and L registers using two's complement 
                        //      arithmetic. The result replaces the contents in the H and L registers. 

                        // create the value of the register pairs for BC and HL
                        let bc = ((self.regs.b as u32) << 8) | (self.regs.c as u32);
                        let hl = ((self.regs.h as u32) << 8) | (self.regs.l as u32);

                        // add the values in the register pairs BC and HL. 
                        // put the HO byte into H and the LO bytes into L.
                        let result =  bc + hl;
                        self.regs.h = ((result & 0x0000ff00) >> 8) as u8;
                        self.regs.l = (result & 0x000000ff) as u8;

                        // set the carry flag
                        self.flags.carry = ((result & 0xffff0000) > 0) as u8;

                        self.pc += 1;
                    }
                    0x0A => {
                        // INSTRUCTION: LDAX B
                        // DESCRIPTION: 
                        //      The contents of the memory location addressed by registers 
                        //      B and C, replace the contents of the accumulator.
                        
                        // compute the address to read from.
                        let addr = (((self.regs.b as u16) << 8) | ((self.regs.c) as u16)) as usize;

                        // load the value at that address into register A
                        self.regs.a = self.memory[addr];

                        self.pc += 1;
                    }
                    0x0B => {
                        // INSTRUCTION: DCX B
                        // DESCRIPTION: 
                        //      The 16-bit number held in the specified register pair is decremented by one.
  
                        // get the content of register pair B and C format them into 
                        // an address in LE format and decrement the value from the 
                        // previous step by one. 
                        let value = (((self.regs.b as u16) << 8) | (self.regs.c as u16)) - 1;

                        // split the new value into two. The LO byte is assigned to
                        // register C and the HO byte is assigned to register B. 
                        self.regs.b = ((value & 0xff00) >> 8) as u8;
                        self.regs.c = (value & 0x00ff) as u8;

                        self.pc += 1;
                    }
                    0x0C => {
                        // INSTRUCTION: INR C
                        // DESCRIPTION: 
                        //      Increment register C by 1;

                        // increment the value in register C by 1.
                        let result = self.regs.c + 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register C with the result of the computation
                        self.regs.c = result;

                        self.pc += 1;
                    }
                    0x0D => {
                        // INSTRUCTION: DCR C
                        // DESCRIPTION:
                        //      The value in register C is decremented by 1;

                        // decrement the value in register C by 1.
                        let result = self.regs.c - 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register C with the result of the computation
                        self.regs.c = result;

                        self.pc += 1;
                    }
                    0x0E => {
                        // INSTRUCTION: MVI C
                        // DESCRIPTION:
                        //      the immediate data byte is stored in register C. 
                        //      No condition flags are affected. 

                        // load the next byte into register C
                        self.regs.c = self.memory[self.pc + 1];

                        self.pc += 2;
                    }
                    0x0F => {
                        // INSTRUCTION: RRC
                        // DESCRIPTION:
                        //      The contents of the accumulator are rotated one bit position to 
                        //      the right, with the low-order bit being transferred to the 
                        //      high-order bit position of the accumulator.

                        // compute carry and use it to compute the new value to
                        // be assigned to the accumulator (A) register.
                        let carry = (self.regs.a & 0x01) as u8;
                        self.regs.a = ((self.regs.a >> 1) | (carry << 7)) as u8;

                        // The Carry bit is set equal to the high-order bit of the accumulator.
                        self.flags.carry = carry;

                        self.pc += 1;
                    }


                    0x10 => { self.pc += 1; }
                    0x11 => {
                        // INSTRUCTION: LXI D
                        // DESCRIPTION: 
                        //      Load the next two bytes into registers D and E. 

                        // load bytes into register D and E
                        self.regs.d = self.memory[self.pc + 2];
                        self.regs.e = self.memory[self.pc + 1];

                        self.pc += 3;
                    }
                    0x12 => {
                        // INSTRUCTION: STAX D
                        // DESCRIPTION: 
                        //      The contents of the accumulator are stored in the memory 
                        //      location addressed by register pair DE

                        // get the content of register pair DE and compute the address
                        // according to the LE format.
                        let addr = (((self.regs.d as u16) << 8) | 
                                    (self.regs.e as u16)) as usize;

                        // get the value in the A register and store this
                        // value at the address created in the previous step.
                        self.memory[addr] = self.regs.a;

                        self.pc += 1;
                    }
                    0x13 => {
                        // INSTRUCTION: INX D
                        // DESCRIPTION: 
                        //      The 16-bit number held in register pair DE is incremented by one.

                        // get the content of register pair D and E format them into 
                        // an address in LE format and increment the value from the 
                        // previous step by one. 
                        let value = (((self.regs.d as u16) << 8) | (self.regs.e as u16)) + 1;

                        // split the new value into two. The LO byte is assigned to
                        // register E and the HO byte is assigned to register D. 
                        self.regs.d = ((value & 0xff00) >> 8) as u8;
                        self.regs.e = (value & 0x00ff) as u8;

                        self.pc += 1;
                    }
                    0x14 => {
                        // INSTRUCTION: INR D
                        // DESCRIPTION: 
                        //      Increment register D by 1;

                        // increment the value in register D by 1.
                        let result = self.regs.d + 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register B with the result of the computation
                        self.regs.d = result;

                        self.pc += 1;
                    }
                    0x15 => {
                        // INSTRUCTION: DCR D
                        // DESCRIPTION:
                        //      The value in register D is decremented by 1;

                        // decrement the value in register D by 1.
                        let result = self.regs.d - 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register D with the result of the computation
                        self.regs.d = result;

                        self.pc += 1;
                    }
                    0x16 => {
                        // INSTRUCTION: MVI D
                        // DESCRIPTION:
                        //      the immediate data byte is stored in register D. 
                        //      No condition flags are affected. 

                        // load the next byte into register D
                        self.regs.d = self.memory[self.pc + 1];

                        self.pc += 2;
                    }
                    0x17 => {
                        // INSTRUCTION: RAL
                        // DESCRIPTION: 
                        //      The contents of the accumulator are rotated one bit position to the left.
                        //      The high-order bit of the accumulator replaces the Carry bit, while the 
                        //      Carry bit replaces the high-order bit of the accumulator.
                        
                        let temp = self.flags.carry;
                        let carry = ((self.regs.a & 0x80) >> 7) as u8;
                        self.regs.a = ((self.regs.a << 1) | (temp << 7)) as u8;

                        // The Carry bit is set equal to the high-order bit of the accumulator.
                        self.flags.carry = carry;

                        self.pc += 1;

                    }
                    0x18 => { self.pc += 1; }
                    0x19 => {
                        // INSTRUCTION: DAD D
                        // DESCRIPTION:
                        //      The 16-bit number in the register pair DE is added to the 
                        //      16-bit number held in the H and L registers using two's complement 
                        //      arithmetic. The result replaces the contents in the H and L registers. 

                        // create the value of the register pairs for DE and HL
                        let de = ((self.regs.d as u32) << 8) | (self.regs.e as u32);
                        let hl = ((self.regs.h as u32) << 8) | (self.regs.l as u32);

                        // add the values in the register pairs DE and HL. 
                        // put the HO byte into H and the LO bytes into L.
                        let result =  de + hl;
                        self.regs.h = ((result & 0x0000ff00) >> 8) as u8;
                        self.regs.l = (result & 0x000000ff) as u8;

                        // set the carry flag
                        self.flags.carry = ((result & 0xffff0000) > 0) as u8;

                        self.pc += 1;
                    }
                    0x1A => {
                        // INSTRUCTION: LDAX D
                        // DESCRIPTION: 
                        //      The contents of the memory location addressed by registers 
                        //      D and E, replace the contents of the accumulator.
                        
                        // compute the address to read from.
                        let addr = (((self.regs.d as u16) << 8) | ((self.regs.e) as u16)) as usize;

                        // load the value at that address into register A
                        self.regs.a = self.memory[addr];

                        self.pc += 1;
                    }
                    0x1B => {
                        // INSTRUCTION: DCX D
                        // DESCRIPTION: 
                        //      The 16-bit number held in the register pair DE is decremented by one.
  
                        // get the content of register pair D and E format them into 
                        // an address in LE format and decrement the value from the 
                        // previous step by one. 
                        let value = (((self.regs.d as u16) << 8) | (self.regs.e as u16)) - 1;

                        // split the new value into two. The LO byte is assigned to
                        // register E and the HO byte is assigned to register D. 
                        self.regs.d = ((value & 0xff00) >> 8) as u8;
                        self.regs.e = (value & 0x00ff) as u8;

                        self.pc += 1;
                    }
                    0x1C => {
                        // INSTRUCTION: INR E
                        // DESCRIPTION: 
                        //      Increment register E by 1;

                        // increment the value in register E by 1.
                        let result = self.regs.e + 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register E with the result of the computation
                        self.regs.e = result;

                        self.pc += 1;
                    }
                    0x1D => {
                        // INSTRUCTION: DCR E
                        // DESCRIPTION:
                        //      The value in register E is decremented by 1;

                        // decrement the value in register E by 1.
                        let result = self.regs.e - 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register E with the result of the computation
                        self.regs.e = result;

                        self.pc += 1;
                    }
                    0x1E => {
                        // INSTRUCTION: MVI E
                        // DESCRIPTION:
                        //      the immediate data byte is stored in register E. 
                        //      No condition flags are affected. 

                        // load the next byte into register E
                        self.regs.e = self.memory[self.pc + 1];

                        self.pc += 2;
                    }
                    0x1F => {
                        // INSTRUCTION: RAR
                        // DESCRIPTION: 
                        //      The contents of the accumulator are rotated one bit position 
                        //      to the right. The low-order bit of the accumulator replaces the
                        //      carry bit, while the carry bit replaces the high-order bit of
                        //      the accumulator.

                        let temp = self.flags.carry;
                        let carry = (self.regs.a & 0x01) as u8;
                        self.regs.a = ((self.regs.a >> 1) | (temp << 7)) as u8;

                        // The Carry bit is set equal to the high-order bit of the accumulator.
                        self.flags.carry = carry;

                        self.pc += 1;
                    }


                    0x20 => { self.pc += 1; }
                    0x21 => {
                        // INSTRUCTION: LXI H
                        // DESCRIPTION: 
                        //      Load the next two bytes into registers H and L. 

                        // load bytes into register H and L
                        self.regs.h = self.memory[self.pc + 2];
                        self.regs.l = self.memory[self.pc + 1];

                        self.pc += 3;
                    }
                    0x22 => {
                        // INSTRUCTION: SHLD
                        // DESCRIPTION: 
                        //      The contents of the L register are stored at the memory address 
                        //      formed by concatenati ng HI AD 0 with LOW ADO. The contents of 
                        //      the H register are stored at the next higher memory address.

                        let mut addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                        (self.memory[self.pc + 1] as u16)) as usize;

                        self.memory[addr] = self.regs.l; addr += 1;
                        self.memory[addr] = self.regs.h;

                        self.pc += 3;
                    }
                    0x23 => {
                        // INSTRUCTION: INX H
                        // DESCRIPTION: 
                        //      The 16-bit number held in register pair HL is incremented by one.

                        // get the content of register pair HL format them into 
                        // an address in LE format and increment the value from the 
                        // previous step by one. 
                        let value = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) + 1;

                        // split the new value into two. The LO byte is assigned to
                        // register L and the HO byte is assigned to register H. 
                        self.regs.h = ((value & 0xff00) >> 8) as u8;
                        self.regs.l = (value & 0x00ff) as u8;

                        self.pc += 1;
                    }
                    0x24 => {
                        // INSTRUCTION: INR H
                        // DESCRIPTION: 
                        //      Increment register H by 1;

                        // increment the value in register H by 1.
                        let result = self.regs.h + 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register H with the result of the computation
                        self.regs.h = result;

                        self.pc += 1;
                    }
                    0x25 => {
                        // INSTRUCTION: DCR H
                        // DESCRIPTION:
                        //      The value in register H is decremented by 1;

                        // decrement the value in register H by 1.
                        let result = self.regs.h - 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register H with the result of the computation
                        self.regs.h = result;

                        self.pc += 1;
                    }
                    0x26 => {
                        // INSTRUCTION: MVI H
                        // DESCRIPTION:
                        //      the immediate data byte is stored in register H. 
                        //      No condition flags are affected. 

                        // load the next byte into register H
                        self.regs.h = self.memory[self.pc + 1];

                        self.pc += 2;
                    }
                    0x27 => {
                        // INSTRUCTION: DAA
                        // DESCRIPTION:
                        
                        if (self.regs.a & 0x0f) > 9 || self.flags.aux_carry == 1 {
                            self.regs.a += 6;
                            self.flags.aux_carry = 1;
                        }

                        let mut ho_nibble = (self.regs.a & 0xf0) >> 4;
                        if ho_nibble > 9 || self.flags.carry == 1 {
                            ho_nibble += 6;
                            self.regs.a = (self.regs.a & 0x0f) | (ho_nibble << 4);
                            self.flags.carry = 1;
                        }

                        self.flags.zero = ((self.regs.a as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((self.regs.a as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = self.regs.a;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };

                        self.pc += 1;
                    }
                    0x28 => { self.pc += 1; }
                    0x29 => {
                        // INSTRUCTION: DAD H
                        // DESCRIPTION:
                        //      The 16-bit number in the specified register pair is added to the 
                        //      16-bit number held in the H and L registers using two's complement 
                        //      arithmetic. The result replaces the contents in the H and L registers. 

                        // create the value of the register pairs HL
                        let hl = ((self.regs.h as u32) << 8) | (self.regs.l as u32);

                        // add the values in the register pairs BC and HL. 
                        // put the HO byte into H and the LO bytes into L.
                        let result =  hl << 1;
                        self.regs.h = ((result & 0x0000ff00) >> 8) as u8;
                        self.regs.l = (result & 0x000000ff) as u8;

                        // set the carry flag
                        self.flags.carry = ((result & 0xffff0000) > 0) as u8;

                        self.pc += 1;
                    }
                    0x2A => {
                        // INSTRUCTION: LHLD
                        // DESCRIPTION: 
                        //      The byte at the memory address formed by concatenating HI ADD 
                        //      with LOW ADD replaces the contents of the L register. The byte 
                        //      at the next higher memory address replaces the contents of the 
                        //      H register.

                        let mut addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                        (self.memory[self.pc + 1] as u16)) as usize;
                        
                        self.regs.l = self.memory[addr]; addr += 1;
                        self.regs.h = self.memory[addr];

                        self.pc += 3;
                    }
                    0x2B => {
                        // INSTRUCTION: DCX H
                        // DESCRIPTION: 
                        //      The 16-bit number held in the register pair HL is decremented by one.
  
                        // get the content of register pair HL format them into 
                        // an address in LE format and decrement the value from the 
                        // previous step by one. 
                        let value = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) - 1;

                        // split the new value into two. The LO byte is assigned to
                        // register L and the HO byte is assigned to register H. 
                        self.regs.h = ((value & 0xff00) >> 8) as u8;
                        self.regs.l = (value & 0x00ff) as u8;

                        self.pc += 1;
                    }
                    0x2C => {
                        // INSTRUCTION: INR L
                        // DESCRIPTION: 
                        //      Increment register L by 1;

                        // increment the value in register L by 1.
                        let result = self.regs.l + 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register L with the result of the computation
                        self.regs.l = result;

                        self.pc += 1;
                    }
                    0x2D => {
                        // INSTRUCTION: DCR L
                        // DESCRIPTION:
                        //      The value in register L is decremented by 1;

                        // decrement the value in register L by 1.
                        let result = self.regs.l - 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register L with the result of the computation
                        self.regs.l = result;

                        self.pc += 1;
                    }
                    0x2E => {
                        // INSTRUCTION: MVI L
                        // DESCRIPTION:
                        //      the immediate data byte is stored in register L. 
                        //      No condition flags are affected. 

                        // load the next byte into register L
                        self.regs.l = self.memory[self.pc + 1];

                        self.pc += 2;
                    }
                    0x2F => {
                        // INSTRUCTION: CMA
                        // DESCRIPTION: 
                        //      Each bit of the contents of the accumulator is complemented 
                        //      (producing the one's complement). 

                        self.regs.a = !self.regs.a;

                        self.pc += 1;
                    }


                    0x30 => { self.pc += 1; }
                    0x31 => {
                        // INSTRUCTION: LXI SP
                        // DESCRIPTION: 
                        //      Load the next two bytes into the stack pointer. 

                        // load bytes into register H and L
                        self.sp = (((self.memory[self.pc + 2] as u16) << 8) | 
                                   (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc += 3;
                    }
                    0x32 => {
                        // INSTRUCTION: STA
                        // DESCRIPTION: 
                        //      The contents of the accumulator replace the byte at the memory 
                        //      address formed by concatenating HI ADD with LOW ADD.

                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.memory[addr] = self.regs.a;

                        self.pc += 3;
                    }
                    0x33 => {
                        // INSTRUCTION: INX SP
                        // DESCRIPTION: 
                        //      The 16-bit number held in register SP is incremented by one.

                        self.sp += 1;

                        self.pc += 1;
                    }
                    0x34 => {
                        // INSTRUCTION: INR M
                        // DESCRIPTION: 

                        let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;

                        let result = self.memory[addr] + 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register B with the result of the computation
                        self.memory[addr] = result;

                        self.pc += 1;
                    }
                    0x35 => {
                        // INSTRUCTION: DCR M
                        let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;

                        let result = self.memory[addr] - 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register B with the result of the computation
                        self.memory[addr] = result;

                        self.pc += 1;
                    }
                    0x36 => {
                        // INSTRUCTION: MVI M

                        let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                        self.memory[addr] = self.memory[self.pc + 1];

                        self.pc += 2;
                    }
                    0x37 => {
                        // INSTRUCTION: STC
                        self.flags.carry = 1;

                        self.pc += 1;
                    }
                    0x38 => { self.pc += 1; }
                    0x39 => {
                        // INSTRUCTION: DAD SP

                        // create the value of the register pairs for BC and HL
                        let hl = ((self.regs.h as u32) << 8) | (self.regs.l as u32);

                        // add the values in the register pairs BC and HL. 
                        // put the HO byte into H and the LO bytes into L.
                        let result =  (self.sp as u32) + hl;
                        self.regs.h = ((result & 0x0000ff00) >> 8) as u8;
                        self.regs.l = (result & 0x000000ff) as u8;

                        // set the carry flag
                        self.flags.carry = ((result & 0xffff0000) > 0) as u8;

                        self.pc += 1;
                    }
                    0x3A => {
                        // INSTRUCTION: LDA
                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.regs.a = self.memory[addr];

                        self.pc += 3;
                    }
                    0x3B => {
                        // INSTRUCTION: DCX SP
                        self.sp -= 1;

                        self.pc += 1;
                    }
                    0x3C => {
                        // INSTRUCTION: INR A
                        // DESCRIPTION: 
                        //      Increment register A by 1;

                        // increment the value in register A by 1.
                        let result = self.regs.a + 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register A with the result of the computation
                        self.regs.a = result;

                        self.pc += 1;
                    }
                    0x3D => {
                        // INSTRUCTION: DCR A
                        // DESCRIPTION:
                        //      The value in register A is decremented by 1;

                        // decrement the value in register A by 1.
                        let result = self.regs.a - 1;

                        // this instruction affects all the condition flags except 
                        // the carry flag.
                        self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                        self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                        self.flags.parity = {
                            let mut counter = 0;
                            let mut r = result;
                            for _ in 0..8 {
                                if (r & 0x01) == 1 { counter += 1; }
                                r >>= 1;
                            }
                            
                            ((counter & 0x01) == 0) as u8
                        };
                        
                        // load register A with the result of the computation
                        self.regs.a = result;

                        self.pc += 1;
                    }
                    0x3E => {
                        // INSTRUCTION: MVI A
                        // DESCRIPTION:
                        //      the immediate data byte is stored in register A. 
                        //      No condition flags are affected. 

                        // load the next byte into register A
                        self.regs.a = self.memory[self.pc + 1];

                        self.pc += 2;
                    }
                    0x3F => {
                        // INSTRUCTION: CMC
                        self.flags.carry = !self.flags.carry;

                        self.pc += 2;
                    }


                    0x40 => {
                        // INSTRUCTION: MOV B, B
                        self.pc += 1;
                    }
                    0x41 => {
                        // INSTRUCTION: MOV B, C
                        self.regs.b = self.regs.c;
                        self.pc += 1;
                    }
                    0x42 => {
                        // INSTRUCTION: MOV B, D
                        self.regs.b = self.regs.d;
                        self.pc += 1;
                    }
                    0x43 => {
                        // INSTRUCTION: MOV B, E
                        self.regs.b = self.regs.e;
                        self.pc += 1;
                    }
                    0x44 => {
                        // INSTRUCTION: MOV B, H
                        self.regs.b = self.regs.h;
                        self.pc += 1;
                    }
                    0x45 => {
                        // INSTRUCTION: MOV B, L
                        self.regs.b = self.regs.l;
                        self.pc += 1;
                    }
                    0x46 => {
                        // INSTRUCTION: MOV B, M
                        // DESCRIPTION: move from memory into B
                        let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                        self.regs.b = self.memory[addr];

                        self.pc += 1;
                    }
                    0x47 => {
                        // INSTRUCTION: MOV B, A
                        self.regs.b = self.regs.a;
                        self.pc += 1;
                    }
                    0x48 => {
                        // INSTRUCTION: MOV C, B
                        self.regs.c = self.regs.b;
                        self.pc += 1;
                    }
                    0x49 => { 
                        // INSTRUCTION: MOV C, C
                        self.pc += 1;
                    }
                    0x4A => {
                        // INSTRUCTION: MOV C, D
                        self.regs.c = self.regs.d;
                        self.pc += 1;
                    }
                    0x4B => {
                        // INSTRUCTION: MOV C, E
                        self.regs.c = self.regs.e;
                        self.pc += 1;
                    }
                    0x4C => {
                        // INSTRUCTION: MOV C, H
                        self.regs.c = self.regs.h;
                        self.pc += 1;
                    }
                    0x4D => {
                        // INSTRUCTION: MOV C, L
                        self.regs.c = self.regs.l;
                        self.pc += 1;
                    }
                    0x4E => {
                        // INSTRUCTION: MOV C, M
                        // DESCRIPTION: move from memory into C
                        let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                        self.regs.c = self.memory[addr];

                        self.pc += 1;
                    }
                    0x4F => {
                        // INSTRUCTION: MOV C, A
                        self.regs.c = self.regs.a;
                        self.pc += 1;
                    }
                    _ => {}

                    // 0x50 => {
                    //     // INSTRUCTION: MOV D, B
                    //     self.regs.d = self.regs.b;
                    //     self.pc += 1;
                    // }
                    // 0x51 => {
                    //     // INSTRUCTION: MOV D, C
                    //     self.regs.d = self.regs.c;
                    //     self.pc += 1;
                    // }
                    // 0x52 => {
                    //     // INSTRUCTION: MOV D, D
                    //     self.pc += 1;
                    // }
                    // 0x53 => {
                    //     // INSTRUCTION: MOV D, E
                    //     self.regs.d = self.regs.e;
                    //     self.pc += 1;
                    // }
                    // 0x54 => {
                    //     // INSTRUCTION: MOV D, H
                    //     self.regs.d = self.regs.h;
                    //     self.pc += 1;
                    // }
                    // 0x55 => {
                    //     // INSTRUCTION: MOV D, L
                    //     self.regs.d = self.regs.l;
                    //     self.pc += 1;
                    // }
                    // 0x56 => {
                    //     // INSTRUCTION: MOV D, M
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     self.regs.d = self.memory[addr];

                    //     self.pc += 1;
                    // }
                    // 0x57 => {
                    //      // INSTRUCTION: MOV D, A
                    //     self.regs.d = self.regs.a;
                    //     self.pc += 1;
                    // }
                    // 0x58 => {
                    //     // INSTRUCTION: MOV E, B
                    //     self.regs.e = self.regs.b;
                    //     self.pc += 1;
                    // }
                    // 0x59 => {
                    //     // INSTRUCTION: MOV E, C
                    //     self.regs.e = self.regs.c;
                    //     self.pc += 1;
                    // }
                    // 0x5A => {
                    //     // INSTRUCTION: MOV E, D
                    //     self.regs.e = self.regs.d;
                    //     self.pc += 1;
                    // }
                    // 0x5B => {
                    //     // INSTRUCTION: MOV E, E
                    //     self.pc += 1;
                    // }
                    // 0x5C => {
                    //     // INSTRUCTION: MOV E, H
                    //     self.regs.e = self.regs.h;
                    //     self.pc += 1;
                    // }
                    // 0x5D => {
                    //     // INSTRUCTION: MOV E, L
                    //     self.regs.e = self.regs.l;
                    //     self.pc += 1;
                    // }
                    // 0x5E => {
                    //     // INSTRUCTION: MOV E, M
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     self.regs.e = self.memory[addr];

                    //     self.pc += 1;
                    // }
                    // 0x5F => {
                    //     // INSTRUCTION: MOV E, A
                    //     self.regs.e = self.regs.a;
                    //     self.pc += 1;
                    // }


                    // 0x60 => {
                    //     // INSTRUCTION: MOV H, B
                    //     self.regs.h = self.regs.b;
                    //     self.pc += 1;
                    // }
                    // 0x61 => {
                    //     // INSTRUCTION: MOV H, C
                    //     self.regs.h = self.regs.c;
                    //     self.pc += 1;
                    // }
                    // 0x62 => {
                    //     // INSTRUCTION: MOV H, D
                    //     self.regs.h = self.regs.d;
                    //     self.pc += 1;
                    // }
                    // 0x63 => {
                    //     // INSTRUCTION: MOV H, E
                    //     self.regs.h = self.regs.e;
                    //     self.pc += 1;
                    // }
                    // 0x64 => {
                    //     // INSTRUCTION: MOV H, H
                    //     self.pc += 1;
                    // }
                    // 0x65 => {
                    //     // INSTRUCTION: MOV H, L
                    //     self.regs.h = self.regs.l;
                    //     self.pc += 1;
                    // }
                    // 0x66 => {
                    //     // INSTRUCTION: MOV H, M
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     self.regs.h = self.memory[addr];

                    //     self.pc += 1;
                    // }
                    // 0x67 => {
                    //     // INSTRUCTION: MOV H, A
                    //     self.regs.h = self.regs.a;
                    //     self.pc += 1;
                    // }
                    // 0x68 => {
                    //     // INSTRUCTION: MOV L, B
                    //     self.regs.l = self.regs.b;
                    //     self.pc += 1;
                    // }
                    // 0x69 => {
                    //     // INSTRUCTION: MOV L, C
                    //     self.regs.l = self.regs.c;
                    //     self.pc += 1;
                    // }
                    // 0x6A => {
                    //     // INSTRUCTION: MOV L, D
                    //     self.regs.l = self.regs.d;
                    //     self.pc += 1;
                    // }
                    // 0x6B => {
                    //     // INSTRUCTION: MOV L, E
                    //     self.regs.l = self.regs.e;
                    //     self.pc += 1;
                    // }
                    // 0x6C => {
                    //     // INSTRUCTION: MOV L, H
                    //     self.regs.l = self.regs.h;
                    //     self.pc += 1;
                    // }
                    // 0x6D => {
                    //     // INSTRUCTION: MOV L, L
                    //     self.pc += 1;
                    // }
                    // 0x6E => {
                    //     // INSTRUCTION: MOV L, M
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     self.regs.l = self.memory[addr];

                    //     self.pc += 1;
                    // }
                    // 0x6F => {
                    //     // INSTRUCTION: MOV L, A
                    //     self.regs.l = self.regs.a;
                    //     self.pc += 1;
                    // }


                    // 0x70 => {
                    //     // INSTRUCTION: MOV M, B
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     self.memory[addr] = self.regs.b;

                    //     self.pc += 1;
                    // }
                    // 0x71 => {
                    //     // INSTRUCTION: MOV M, C
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     self.memory[addr] = self.regs.c;

                    //     self.pc += 1;
                    // }
                    // 0x72 => {
                    //     // INSTRUCTION: MOV M, D
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     self.memory[addr] = self.regs.d;

                    //     self.pc += 1;
                    // }
                    // 0x73 => {
                    //     // INSTRUCTION: MOV M, E
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     self.memory[addr] = self.regs.e;

                    //     self.pc += 1;
                    // }
                    // 0x74 => {
                    //     // INSTRUCTION: MOV M, H
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     self.memory[addr] = self.regs.h;

                    //     self.pc += 1;
                    // }
                    // 0x75 => {
                    //     // INSTRUCTION: MOV M, L
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     self.memory[addr] = self.regs.l;

                    //     self.pc += 1;
                    // }
                    // 0x76 => {}
                    // 0x77 => {
                    //     // INSTRUCTION: MOV M, A
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     self.memory[addr] = self.regs.a;

                    //     self.pc += 1;
                    // }
                    // 0x78 => {
                    //     // INSTRUCTION: MOV A, B
                    //     self.regs.a = self.regs.b;

                    //     self.pc += 1;
                    // }
                    // 0x79 => {
                    //     // INSTRUCTION: MOV A, C
                    //     self.regs.a = self.regs.c;

                    //     self.pc += 1;
                    // }
                    // 0x7A => {
                    //     // INSTRUCTION: MOV A, D
                    //     self.regs.a = self.regs.e;

                    //     self.pc += 1;
                    // }
                    // 0x7B => {
                    //     // INSTRUCTION: MOV A, E
                    //     self.regs.a = self.regs.e;

                    //     self.pc += 1;
                    // }
                    // 0x7C => {
                    //     // INSTRUCTION: MOV A, H
                    //     self.regs.a = self.regs.h;

                    //     self.pc += 1;
                    // }
                    // 0x7D => {
                    //     // INSTRUCTION: MOV A, L
                    //     self.regs.a = self.regs.l;

                    //     self.pc += 1;
                    // }
                    // 0x7E => {
                    //     // INSTRUCTION: MOV A, M
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     self.regs.a = self.memory[addr];

                    //     self.pc += 1;
                    // }
                    // 0x7F => {
                    //     // INSTRUCTION: MOV A, A
                    //     self.pc += 1;
                    // }


                    // 0x80 => {
                    //     // INSTRUCTION: ADD B
                    //     let result = (self.regs.a as u16) + (self.regs.b as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x81 => {
                    //     // INSTRUCTION: ADD C
                    //     let result = (self.regs.a as u16) + (self.regs.c as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x82 => {
                    //     // INSTRUCTION: ADD D
                    //     let result = (self.regs.a as u16) + (self.regs.d as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x83 => {
                    //     // INSTRUCTION: ADD E
                    //     let result = (self.regs.a as u16) + (self.regs.e as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x84 => {
                    //     // INSTRUCTION: ADD H
                    //     let result = (self.regs.a as u16) + (self.regs.h as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x85 => {
                    //     // INSTRUCTION: ADD L
                    //     let result = (self.regs.a as u16) + (self.regs.l as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x86 => {
                    //     // INSTRUCTION: ADD M
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     let result = (self.regs.a as u16) + (self.memory[addr] as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x87 => {
                    //     // INSTRUCTION: ADD A
                    //     let result = (self.regs.a as u16) << 1;

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x88 => {
                    //     // INSTRUCTION: ADC B
                    //     let result = (self.regs.a as u16) + (self.regs.b as u16) + 
                    //                                         (self.flags.carry as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x89 => {
                    //     // INSTRUCTION: ADC C
                    //     let result = (self.regs.a as u16) + (self.regs.c as u16) + 
                    //                                         (self.flags.carry as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x8A => {
                    //     // INSTRUCTION: ADC D
                    //     let result = (self.regs.a as u16) + (self.regs.d as u16) + 
                    //                                         (self.flags.carry as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x8B => {
                    //     // INSTRUCTION: ADC E
                    //     let result = (self.regs.a as u16) + (self.regs.e as u16) + 
                    //                                         (self.flags.carry as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x8C => {
                    //     // INSTRUCTION: ADC H
                    //     let result = (self.regs.a as u16) + (self.regs.h as u16) + 
                    //                                         (self.flags.carry as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x8D => {
                    //     // INSTRUCTION: ADC L
                    //     let result = (self.regs.a as u16) + (self.regs.l as u16) + 
                    //                                         (self.flags.carry as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x8E => {
                    //     // INSTRUCTION: ADC M
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     let result = (self.regs.a as u16) + (self.memory[addr] as u16) + 
                    //                                         (self.flags.carry as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x8F => {
                    //     // INSTRUCTION: ADD A
                    //     let result = ((self.regs.a as u16) << 1) + (self.flags.carry as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }


                    // 0x90 => {
                    //     // INSTRUCTION: SUB B
                    //     let result = (self.regs.a as u16) - (self.regs.b as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x91 => {
                    //     // INSTRUCTION: SUB C
                    //     let result = (self.regs.a as u16) - (self.regs.c as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x92 => {
                    //     // INSTRUCTION: SUB D
                    //     let result = (self.regs.a as u16) - (self.regs.d as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x93 => {
                    //     // INSTRUCTION: SUB E
                    //     let result = (self.regs.a as u16) - (self.regs.e as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x94 => {
                    //     // INSTRUCTION: SUB H
                    //     let result = (self.regs.a as u16) - (self.regs.h as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x95 => {
                    //     // INSTRUCTION: SUB L
                    //     let result = (self.regs.a as u16) - (self.regs.l as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x96 => {
                    //     // INSTRUCTION: SUB M
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     let result = (self.regs.a as u16) - (self.memory[addr] as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x97 => {
                    //     // INSTRUCTION: SUB A
                    //     let result = 0_u16;

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x98 => {
                    //     // INSTRUCTION: SBB B
                    //     let result = (self.regs.a as u16) - (self.regs.b + self.flags.carry) as u16;

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x99 => {
                    //     // INSTRUCTION: SBB C
                    //     let result = (self.regs.a as u16) - (self.regs.c + self.flags.carry) as u16;

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x9A => {
                    //     // INSTRUCTION: SBB D
                    //     let result = (self.regs.a as u16) - (self.regs.d + self.flags.carry) as u16;

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x9B => {
                    //     // INSTRUCTION: SBB E
                    //     let result = (self.regs.a as u16) - (self.regs.e + self.flags.carry) as u16;

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x9C => {
                    //     // INSTRUCTION: SBB H
                    //     let result = (self.regs.a as u16) - (self.regs.h + self.flags.carry) as u16;

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x9D => {
                    //     // INSTRUCTION: SBB L
                    //     let result = (self.regs.a as u16) - (self.regs.l + self.flags.carry) as u16;

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x9E => {
                    //     // INSTRUCTION: SBB M
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     let result = (self.regs.a as u16) - (self.memory[addr] + self.flags.carry) as u16;

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0x9F => {
                    //     // INSTRUCTION: SBB A
                    //     let result = (self.regs.a as u16) - (self.regs.a + self.flags.carry) as u16;

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }


                    // 0xA0 => {
                    //     // INSTRUCTION: ANA B
                    //     let result = (self.regs.a as u16) & (self.regs.b as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xA1 => {
                    //     // INSTRUCTION: ANA C
                    //     let result = (self.regs.a as u16) & (self.regs.c as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xA2 => {
                    //     // INSTRUCTION: ANA D
                    //     let result = (self.regs.a as u16) & (self.regs.d as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xA3 => {
                    //     // INSTRUCTION: ANA E
                    //     let result = (self.regs.a as u16) & (self.regs.e as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xA4 => {
                    //     // INSTRUCTION: ANA H
                    //     let result = (self.regs.a as u16) & (self.regs.h as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xA5 => {
                    //     // INSTRUCTION: ANA L
                    //     let result = (self.regs.a as u16) & (self.regs.l as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xA6 => {
                    //     // INSTRUCTION: ANA M
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     let result = (self.regs.a as u16) & (self.memory[addr] as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xA7 => {
                    //     // INSTRUCTION: ANA A
                    //     let result = self.regs.a as u16;

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xA8 => {
                    //     // INSTRUCTION: XRA B
                    //     let result = (self.regs.a as u16) ^ (self.regs.b as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xA9 => {
                    //     // INSTRUCTION: XRA C
                    //     let result = (self.regs.a as u16) ^ (self.regs.c as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xAA => {
                    //     // INSTRUCTION: XRA D
                    //     let result = (self.regs.a as u16) ^ (self.regs.d as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xAB => {
                    //     // INSTRUCTION: XRA E
                    //     let result = (self.regs.a as u16) ^ (self.regs.e as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xAC => {
                    //     // INSTRUCTION: XRA H
                    //     let result = (self.regs.a as u16) ^ (self.regs.h as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xAD => {
                    //     // INSTRUCTION: XRA L
                    //     let result = (self.regs.a as u16) ^ (self.regs.l as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xAE => {
                    //     // INSTRUCTION: XRA M
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     let result = (self.regs.a as u16) ^ (self.memory[addr] as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xAF => {
                    //     // INSTRUCTION: XRA A
                    //     let result = (self.regs.a as u16) ^ (self.regs.a as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }


                    // 0xB0 => {
                    //     // INSTRUCTION: ORA B
                    //     let result = (self.regs.a as u16) | (self.regs.b as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xB1 => {
                    //     // INSTRUCTION: ORA C
                    //     let result = (self.regs.a as u16) | (self.regs.c as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xB2 => {
                    //     // INSTRUCTION: ORA D
                    //     let result = (self.regs.a as u16) | (self.regs.d as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xB3 => {
                    //     // INSTRUCTION: ORA E
                    //     let result = (self.regs.a as u16) | (self.regs.e as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xB4 => {
                    //     // INSTRUCTION: ORA H
                    //     let result = (self.regs.a as u16) | (self.regs.h as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xB5 => {
                    //     // INSTRUCTION: ORA L
                    //     let result = (self.regs.a as u16) | (self.regs.l as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xB6 => {
                    //     // INSTRUCTION: ORA C
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     let result = (self.regs.a as u16) | (self.memory[addr] as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xB7 => {
                    //     // INSTRUCTION: ORA A
                    //     let result = (self.regs.a as u16) | (self.regs.a as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;

                    //     self.pc += 1;
                    // }
                    // 0xB8 => {
                    //     // INSTRUCTION: CMP B
                    //     let result = (self.regs.a as u16) - (self.regs.b as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.pc += 1;
                    // }
                    // 0xB9 => {
                    //     // INSTRUCTION: CMP C
                    //     let result = (self.regs.a as u16) - (self.regs.c as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.pc += 1;
                    // }
                    // 0xBA => {
                    //     // INSTRUCTION: CMP D
                    //     let result = (self.regs.a as u16) - (self.regs.d as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.pc += 1;
                    // }
                    // 0xBB => {
                    //     // INSTRUCTION: CMP E
                    //     let result = (self.regs.a as u16) - (self.regs.e as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.pc += 1;
                    // }
                    // 0xBC => {
                    //     // INSTRUCTION: CMP H
                    //     let result = (self.regs.a as u16) - (self.regs.h as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.pc += 1;
                    // }
                    // 0xBD => {
                    //     // INSTRUCTION: CMP L
                    //     let result = (self.regs.a as u16) - (self.regs.l as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.pc += 1;
                    // }
                    // 0xBE => {
                    //     // INSTRUCTION: CMP M
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     let result = (self.regs.a as u16) - (self.memory[addr] as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.pc += 1;
                    // }
                    // 0xBF => {
                    //     // INSTRUCTION: CMP A
                    //     let result = (self.regs.a as u16) - (self.regs.a as u16);

                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.pc += 1;
                    // }


                    // 0xC0 => {
                    //     // INSTRUCTION: RNZ
                    //     if self.flags.zero == 0 {
                    //         let lsb = self.memory[self.sp];
                    //         let msb = self.memory[self.sp + 1];

                    //         let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                    //         self.pc = addr;
                    //         self.sp += 2;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xC1 => {
                    //     // INSTRUCTION: POP B
                    //     self.regs.c = self.memory[self.sp];
                    //     self.regs.b = self.memory[self.sp + 1];

                    //     self.sp += 2;
                    // }
                    // 0xC2 => {
                    //     // INSTRUCTION: JNZ
                    //     if self.flags.zero == 0 {
                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xC3 => {
                    //     // INSTRUCTION: JMP
                    //     let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                 (self.memory[self.pc + 1] as u16)) as usize;

                    //     self.pc = addr;
                    // }
                    // 0xC4 => {
                    //     // INSTRUCTION: CNZ
                    //     if self.flags.zero == 0 {
                    //         let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //         let lsb = (self.pc & 0x00ff) as u8;

                    //         self.memory[self.sp] = lsb; self.sp += 1;
                    //         self.memory[self.sp] = msb; self.sp += 1;

                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr

                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xC5 => {
                    //     // INSTRUCTION: PUSH B
                    //     self.memory[self.sp - 1] = self.regs.b;
                    //     self.memory[self.sp - 2] = self.regs.c;

                    //     self.sp -= 2;

                    //     self.pc += 1;
                    // }
                    // 0xC6 => {
                    //     // INSTRUCTION: ADI
                    //     let result = (self.regs.a as u16) + (self.memory[self.pc + 1] as u16);
                        
                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;
                    //     self.pc += 2;
                    // }
                    // 0xC7 => {
                    //     // INSTRUCTION: RST 0
                    //     let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //     let lsb = (self.pc & 0x00ff) as u8;

                    //     self.memory[self.sp] = lsb; self.sp += 1;
                    //     self.memory[self.sp] = msb;

                    //     self.pc = 0;
                    // }
                    // 0xC8 => {
                    //     // INSTRUCTION: RZ
                    //     if self.flags.zero == 1 {
                    //         let lsb = self.memory[self.sp];
                    //         let msb = self.memory[self.sp + 1];

                    //         let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                    //         self.pc = addr;
                    //         self.sp += 2;

                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xC9 => {
                    //     // INSTRUCTION: RET
                    //     let lsb = self.memory[self.sp];
                    //     let msb = self.memory[self.sp + 1];

                    //     let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                    //     self.pc = addr;
                    //     self.sp += 2;
                    // }
                    // 0xCA => {
                    //     // INSTRUCTION: JZ
                    //     if self.flags.zero == 1 {
                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xCB => { 
                    //     // INSTRUCTION: NOP
                    //     self.pc += 1; 
                    // }
                    // 0xCC => {
                    //     // INSTRUCTION: CZ
                    //     if self.flags.zero == 1 {
                    //         let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //         let lsb = (self.pc & 0x00ff) as u8;

                    //         self.memory[self.sp] = lsb; self.sp += 1;
                    //         self.memory[self.sp] = msb; self.sp += 1;

                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xCD => {
                    //     // INSTRUCTION: CALL
                    //     let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //     let lsb = (self.pc & 0x00ff) as u8;

                    //     self.memory[self.sp] = lsb; self.sp += 1;
                    //     self.memory[self.sp] = msb; self.sp += 1;

                    //     let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                 (self.memory[self.pc + 1] as u16)) as usize;

                    //     self.pc = addr
                    // }
                    // 0xCE => {
                    //     // INSTRUCTION: ACI
                    //     let result = (self.regs.a as u16) + (self.memory[self.pc + 1] as u16 + 
                    //                                          self.flags.carry as u16);
                        
                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;
                    //     self.pc += 2;
                    // }
                    // 0xCF => {
                    //     // INSTRUCTION: RST 1
                    //     let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //     let lsb = (self.pc & 0x00ff) as u8;

                    //     self.memory[self.sp] = lsb; self.sp += 1;
                    //     self.memory[self.sp] = msb; self.sp += 1;

                    //     self.pc = (1_u8 << 3) as usize;
                    // }


                    // 0xD0 => {
                    //     // INSTRUCTION: RNC
                    //     if self.flags.carry == 0 {
                    //         let lsb = self.memory[self.sp];
                    //         let msb = self.memory[self.sp + 1];

                    //         let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                    //         self.pc = addr;
                    //         self.sp += 2;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xD1 => {
                    //     // INSTRUCTION: POP D
                    //     self.regs.e = self.memory[self.sp];
                    //     self.regs.d = self.memory[self.sp + 1];

                    //     self.sp += 2;
                    // }
                    // 0xD2 => {
                    //     // INSTRUCTION: JNC
                    //     if self.flags.carry == 0 {
                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xD3 => {
                    //     // INSTRUCTION: OUT
                    //     self.pc += 1;
                    // }
                    // 0xD4 => {
                    //     // INSTRUCTION: CNC
                    //     if self.flags.carry == 0 {
                    //         let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //         let lsb = (self.pc & 0x00ff) as u8;

                    //         self.memory[self.sp] = lsb; self.sp += 1;
                    //         self.memory[self.sp] = msb; self.sp += 1;

                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr

                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xD5 => {
                    //     // INSTRUCTION: PUSH D
                    //     self.memory[self.sp - 1] = self.regs.d;
                    //     self.memory[self.sp - 2] = self.regs.e;

                    //     self.sp -= 2;

                    //     self.pc += 1;
                    // }
                    // 0xD6 => {
                    //     // INSTRUCTION: SUI
                    //     let result = (self.regs.a as u16) - (self.memory[self.pc + 1] as u16);
                        
                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;
                    //     self.pc += 2;
                    // }
                    // 0xD7 => {
                    //      // INSTRUCTION: RST 2
                    //     let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //     let lsb = (self.pc & 0x00ff) as u8;

                    //     self.memory[self.sp] = lsb; self.sp += 1;
                    //     self.memory[self.sp] = msb; self.sp += 1;

                    //     self.pc = (2_u8 << 3) as usize;
                    // }
                    // 0xD8 => {
                    //     // INSTRUCTION: RC
                    //     if self.flags.zero == 1 {
                    //         let lsb = self.memory[self.sp];
                    //         let msb = self.memory[self.sp + 1];

                    //         let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                    //         self.pc = addr;
                    //         self.sp += 2;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xD9 => {
                    //     // INSTRUCTION: NOP
                    //     self.pc += 1;
                    // }
                    // 0xDA => {
                    //     // INSTRUCTION: JC
                    //     if self.flags.carry == 1 {
                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xDB => {
                    //     // INSTRUCTION: IN
                    //     self.pc += 1;
                    // }
                    // 0xDC => {
                    //     // INSTRUCTION: CC
                    //     if self.flags.carry == 1 {
                    //         let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //         let lsb = (self.pc & 0x00ff) as u8;

                    //         self.memory[self.sp] = lsb; self.sp += 1;
                    //         self.memory[self.sp] = msb; self.sp += 1;

                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xDD => {
                    //     // INSTRUCTION: NOP
                    //     self.pc += 1;
                    // }
                    // 0xDE => {
                    //     // INSTRUCTION: SBI
                    //     let result = (self.regs.a as u16) + (self.memory[self.pc + 1] as u16 + 
                    //                                          self.flags.carry as u16);
                        
                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;
                    //     self.pc += 2;
                    // }
                    // 0xDF => {
                    //      // INSTRUCTION: RST 3
                    //     let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //     let lsb = (self.pc & 0x00ff) as u8;

                    //     self.memory[self.sp] = lsb; self.sp += 1;
                    //     self.memory[self.sp] = msb; self.sp += 1;

                    //     self.pc = (3_u8 << 3) as usize;
                    // }


                    // 0xE0 => {
                    //     // INSTRUCTION: RPO
                    //     if self.flags.parity == 0 {
                    //         let lsb = self.memory[self.sp];
                    //         let msb = self.memory[self.sp + 1];

                    //         let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                    //         self.pc = addr;
                    //         self.sp += 2;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xE1 => {
                    //     // INSTRUCTION: POP H
                    //     self.regs.l = self.memory[self.sp];
                    //     self.regs.h = self.memory[self.sp + 1];

                    //     self.sp += 2;
                    // }
                    // 0xE2 => {
                    //     // INSTRUCTION: JPO
                    //     if self.flags.parity == 0 {
                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xE3 => {
                    //     // INSTRUCTION: XTHL
                    //     let lsb = self.memory[self.sp];
                    //     let msb = self.memory[self.sp + 1];

                    //     self.memory[self.sp] = self.regs.l;
                    //     self.memory[self.sp + 1] = self.regs.h;

                    //     self.regs.l = lsb;
                    //     self.regs.h = msb;

                    //     self.pc += 1;
                    // }
                    // 0xE4 => {
                    //     // INSTRUCTION: CPO
                    //     if self.flags.parity == 0 {
                    //         let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //         let lsb = (self.pc & 0x00ff) as u8;

                    //         self.memory[self.sp] = lsb; self.sp += 1;
                    //         self.memory[self.sp] = msb; self.sp += 1;

                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr

                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xE5 => {
                    //     // INSTRUCTION: PUSH H
                    //     self.memory[self.sp - 1] = self.regs.h;
                    //     self.memory[self.sp - 2] = self.regs.l;

                    //     self.sp -= 2;

                    //     self.pc += 1;
                    // }
                    // 0xE6 => {
                    //     // INSTRUCTION: ANI
                    //     let result = (self.regs.a as u16) & (self.memory[self.pc + 1] as u16);
                        
                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;
                    //     self.pc += 2;
                    // }
                    // 0xE7 => {
                    //      // INSTRUCTION: RST 4
                    //     let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //     let lsb = (self.pc & 0x00ff) as u8;

                    //     self.memory[self.sp] = lsb; self.sp += 1;
                    //     self.memory[self.sp] = msb; self.sp += 1;

                    //     self.pc = (4_u8 << 3) as usize;
                    // }
                    // 0xE8 => {
                    //     // INSTRUCTION: RPE
                    //     if self.flags.parity == 1 {
                    //         let lsb = self.memory[self.sp];
                    //         let msb = self.memory[self.sp + 1];

                    //         let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                    //         self.pc = addr;
                    //         self.sp += 2;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xE9 => {
                    //     // INSTRUCTION: PCHL
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     self.pc = addr;
                    // }
                    // 0xEA => {
                    //     // INSTRUCTION: JPE
                    //     if self.flags.parity == 1 {
                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xEB => {
                    //     // INSTRUCTION: XCHG
                    //     let (d, e) = (self.regs.d, self.regs.d);

                    //     self.regs.d = self.regs.h;
                    //     self.regs.e = self.regs.l;

                    //     self.regs.h = d;
                    //     self.regs.l = e;

                    //     self.pc += 1;
                    // }
                    // 0xEC => {
                    //     // INSTRUCTION: CPE
                    //     if self.flags.parity == 1 {
                    //         let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //         let lsb = (self.pc & 0x00ff) as u8;

                    //         self.memory[self.sp] = lsb; self.sp += 1;
                    //         self.memory[self.sp] = msb; self.sp += 1;

                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr

                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xED => {
                    //     // INSTRUCTION: NOP
                    //     self.pc += 1;
                    // }
                    // 0xEE => {
                    //     // INSTRUCTION: XRI
                    //     let result = (self.regs.a as u16) ^ (self.memory[self.pc + 1] as u16);
                        
                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;
                    //     self.pc += 2;
                    // }
                    // 0xEF => {
                    //      // INSTRUCTION: RST 5
                    //     let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //     let lsb = (self.pc & 0x00ff) as u8;

                    //     self.memory[self.sp] = lsb; self.sp += 1;
                    //     self.memory[self.sp] = msb; self.sp += 1;

                    //     self.pc = (5_u8 << 3) as usize;
                    // }


                    // 0xF0 => {
                    //     if self.flags.sign == 0 {
                    //         let lsb = self.memory[self.sp];
                    //         let msb = self.memory[self.sp + 1];

                    //         let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                    //         self.pc = addr;
                    //         self.sp += 2;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xF1 => {
                    //     // INSTRUCTION: POP PSW
                    //     self.regs.a = self.memory[self.sp + 1];

                    //     // get the content of the memory location specified by the stack pointer
                    //     let result = self.memory[self.sp];

                    //     // restore the condition flags with this value
                    //     self.flags.carry = (0x05 == (result & 0x08)) as u8;
                    //     self.flags.zero = ((result & 0xff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x80) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..8 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.sp += 2;

                    //     self.pc += 1;
                    // }
                    // 0xF2 => {
                    //     // INSTRUCTION: JP
                    //     if self.flags.sign == 1 {
                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xF3 => {
                    //     // INSTRUCTION: DI

                    //     // disable interrupts
                    //     self.int_enable = 0;
                    //     self.pc += 1;
                    // }
                    // 0xF4 => {
                    //     // INSTRUCTION: CP
                    //     if self.flags.sign == 0 {
                    //         let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //         let lsb = (self.pc & 0x00ff) as u8;

                    //         self.memory[self.sp] = lsb; self.sp += 1;
                    //         self.memory[self.sp] = msb; self.sp += 1;

                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr

                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }   
                    // 0xF5 => {
                    //     // INSTRUCTION: PUSH PSW
                    //     self.memory[self.sp - 1] = self.regs.a;

                    //     let psw = self.flags.zero        |
					// 		      self.flags.sign << 1   |
					// 		      self.flags.parity << 2 |
					// 		      self.flags.carry << 3  |
					// 		      self.flags.aux_carry << 4 ;

			        //     self.memory[self.sp - 2] = psw;
			        //     self.sp -= 2;

                    //     self.pc += 1;
                    // }
                    // 0xF6 => {
                    //     // INSTRUCTION: ORI
                    //     let result = (self.regs.a as u16) | (self.memory[self.pc + 1] as u16);
                        
                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;
                    //     self.pc += 2;
                    // }
                    // 0xF7 => {
                    //      // INSTRUCTION: RST 6
                    //     let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //     let lsb = (self.pc & 0x00ff) as u8;

                    //     self.memory[self.sp] = lsb; self.sp += 1;
                    //     self.memory[self.sp] = msb; self.sp += 1;

                    //     self.pc = (6_u8 << 3) as usize;
                    // }
                    // 0xF8 => {
                    //     // INSTRUCTION: RM
                    //     if self.flags.sign == 1 {
                    //         let lsb = self.memory[self.sp];
                    //         let msb = self.memory[self.sp + 1];

                    //         let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                    //         self.pc = addr;
                    //         self.sp += 2;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xF9 => {
                    //     // INSTRUCTION: SPHL
                    //     let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    //     self.sp = addr;

                    //     self.pc += 1;
                    // }
                    // 0xFA => {
                    //     // INSTRUCTION: JM
                    //     if self.flags.sign == 1 {
                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr;
                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xFB => {
                    //     // INSTRUCTION: EI

                    //     // enable interrupts
                    //     self.int_enable = 1;

                    //     self.pc += 1;
                    // }
                    // 0xFC => {
                    //     // INSTRUCTION: CM
                    //     if self.flags.sign == 1 {
                    //         let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //         let lsb = (self.pc & 0x00ff) as u8;

                    //         self.memory[self.sp] = lsb; self.sp += 1;
                    //         self.memory[self.sp] = msb; self.sp += 1;

                    //         let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                    //                     (self.memory[self.pc + 1] as u16)) as usize;

                    //         self.pc = addr

                    //     } else {
                    //         self.pc += 1;
                    //     }
                    // }
                    // 0xFD => {
                    //     // INSTRUCTION: NOP
                    //     self.pc += 1;
                    // }
                    // 0xFE => {
                    //     // INSTRUCTION: CPI
                    //     let result = (self.regs.a as u16) - (self.memory[self.pc + 1] as u16);
                        
                    //     self.flags.carry = (result > 0xff) as u8;
                    //     self.flags.zero = ((result & 0xffff) == 0) as u8;
                    //     self.flags.sign = ((result & 0x8000) != 0) as u8;
                    //     self.flags.parity = {
                    //         let mut counter = 0;
                    //         let mut r = result;
                    //         for _ in 0..16 {
                    //             if (r & 0x01) == 1 { counter += 1; }
                    //             r >>= 1;
                    //         }
                            
                    //         ((counter & 0x01) == 0) as u8
                    //     };

                    //     self.regs.a = result as u8;
                    //     self.pc += 2;
                    // }
                    // 0xFF => {
                    //      // INSTRUCTION: RST 7
                    //     let msb = ((self.pc & 0xff00) >> 8) as u8;
                    //     let lsb = (self.pc & 0x00ff) as u8;

                    //     self.memory[self.sp] = lsb; self.sp += 1;
                    //     self.memory[self.sp] = msb; self.sp += 1;

                    //     self.pc = (7_u8 << 3) as usize;
                    // }
                }
            }
        }
    }
}