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
        pub memory: Vec<u8>
    }

    impl Intel8080 {
        pub fn new() -> Self {
            Intel8080 {
                regs: Register::new(),
                flags: ConditionFlags::new(),
                pc: 0_usize,
                sp: 0_usize,
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
                    0x33 => {}
                    0x34 => {}
                    0x35 => {}
                    0x36 => {}
                    0x37 => {}
                    0x38 => { self.pc += 1; }
                    0x39 => {}
                    0x3A => {}
                    0x3B => {}
                    0x3C => {}
                    0x3D => {}
                    0x3E => {}
                    0x3F => {}


                    0x40 => {}
                    0x41 => {}
                    0x42 => {}
                    0x43 => {}
                    0x44 => {}
                    0x45 => {}
                    0x46 => {}
                    0x47 => {}
                    0x48 => {}
                    0x49 => {}
                    0x4A => {}
                    0x4B => {}
                    0x4C => {}
                    0x4D => {}
                    0x4E => {}
                    0x4F => {}


                    0x50 => {}
                    0x51 => {}
                    0x52 => {}
                    0x53 => {}
                    0x54 => {}
                    0x55 => {}
                    0x56 => {}
                    0x57 => {}
                    0x58 => {}
                    0x59 => {}
                    0x5A => {}
                    0x5B => {}
                    0x5C => {}
                    0x5D => {}
                    0x5E => {}
                    0x5F => {}


                    0x60 => {}
                    0x61 => {}
                    0x62 => {}
                    0x63 => {}
                    0x64 => {}
                    0x65 => {}
                    0x66 => {}
                    0x67 => {}
                    0x68 => {}
                    0x69 => {}
                    0x6A => {}
                    0x6B => {}
                    0x6C => {}
                    0x6D => {}
                    0x6E => {}
                    0x6F => {}


                    0x70 => {}
                    0x71 => {}
                    0x72 => {}
                    0x73 => {}
                    0x74 => {}
                    0x75 => {}
                    0x76 => {}
                    0x77 => {}
                    0x78 => {}
                    0x79 => {}
                    0x7A => {}
                    0x7B => {}
                    0x7C => {}
                    0x7D => {}
                    0x7E => {}
                    0x7F => {}


                    0x80 => {}
                    0x81 => {}
                    0x82 => {}
                    0x83 => {}
                    0x84 => {}
                    0x85 => {}
                    0x86 => {}
                    0x87 => {}
                    0x88 => {}
                    0x89 => {}
                    0x8A => {}
                    0x8B => {}
                    0x8C => {}
                    0x8D => {}
                    0x8E => {}
                    0x8F => {}


                    0x90 => {}
                    0x91 => {}
                    0x92 => {}
                    0x93 => {}
                    0x94 => {}
                    0x95 => {}
                    0x96 => {}
                    0x97 => {}
                    0x98 => {}
                    0x99 => {}
                    0x9A => {}
                    0x9B => {}
                    0x9C => {}
                    0x9D => {}
                    0x9E => {}
                    0x9F => {}


                    0xA0 => {}
                    0xA1 => {}
                    0xA2 => {}
                    0xA3 => {}
                    0xA4 => {}
                    0xA5 => {}
                    0xA6 => {}
                    0xA7 => {}
                    0xA8 => {}
                    0xA9 => {}
                    0xAA => {}
                    0xAB => {}
                    0xAC => {}
                    0xAD => {}
                    0xAE => {}
                    0xAF => {}


                    0xB0 => {}
                    0xB1 => {}
                    0xB2 => {}
                    0xB3 => {}
                    0xB4 => {}
                    0xB5 => {}
                    0xB6 => {}
                    0xB7 => {}
                    0xB8 => {}
                    0xB9 => {}
                    0xBA => {}
                    0xBB => {}
                    0xBC => {}
                    0xBD => {}
                    0xBE => {}
                    0xBF => {}


                    0xC0 => {}
                    0xC1 => {}
                    0xC2 => {}
                    0xC3 => {}
                    0xC4 => {}
                    0xC5 => {}
                    0xC6 => {}
                    0xC7 => {}
                    0xC8 => {}
                    0xC9 => {}
                    0xCA => {}
                    0xCB => {}
                    0xCC => {}
                    0xCD => {}
                    0xCE => {}
                    0xCF => {}


                    0xD0 => {}
                    0xD1 => {}
                    0xD2 => {}
                    0xD3 => {}
                    0xD4 => {}
                    0xD5 => {}
                    0xD6 => {}
                    0xD7 => {}
                    0xD8 => {}
                    0xD9 => {}
                    0xDA => {}
                    0xDB => {}
                    0xDC => {}
                    0xDD => {}
                    0xDE => {}
                    0xDF => {}


                    0xE0 => {}
                    0xE1 => {}
                    0xE2 => {}
                    0xE3 => {}
                    0xE4 => {}
                    0xE5 => {}
                    0xE6 => {}
                    0xE7 => {}
                    0xE8 => {}
                    0xE9 => {}
                    0xEA => {}
                    0xEB => {}
                    0xEC => {}
                    0xED => {}
                    0xEE => {}
                    0xEF => {}


                    0xF0 => {}
                    0xF1 => {}
                    0xF2 => {}
                    0xF3 => {}
                    0xF4 => {}
                    0xF5 => {}
                    0xF6 => {}
                    0xF7 => {}
                    0xF8 => {}
                    0xF9 => {}
                    0xFA => {}
                    0xFB => {}
                    0xFC => {}
                    0xFD => {}
                    0xFE => {}
                    0xFF => {}
                }
            }
        }
    }
}