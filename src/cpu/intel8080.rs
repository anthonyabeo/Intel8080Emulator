use std::fs::File;
use std::path::Path;
use std::io::Read;

use crate::cpu::{ConditionFlags, Register};
use crate::cpu::utils::*;
use crate::cpu::instructions::*;


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
            memory: vec![0_u8; 0x10000] // 65 KB of Memory
        }
    }

    pub fn load_program(&mut self, file_name: &str) {
        let mut f = match File::open(Path::new(file_name)) {
            Ok(file) => file,
            Err(e) => panic!("Could not open file - {}", e)
        };

        f.read(&mut self.memory).unwrap();
    }
    
    pub fn run(&mut self) {
        while self.memory[self.pc] != 0x76 { // while opcode != HLT (0x76)
            match self.memory[self.pc] {
                0x00 => { self.pc += 1; } // NOP
                0x01 => { lxi(self, 'B'); self.pc += 3; }
                0x02 => { stax(self, 'B'); self.pc += 1; }
                0x03 => { inx(self, 'B'); self.pc += 1; }
                0x04 => { inr(self, 'B'); self.pc += 1; }
                0x05 => { dcr(self, 'B'); self.pc += 1; }
                0x06 => { mvi(self, 'B'); self.pc += 2; }
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
                0x09 => { dad(self, 'B'); self.pc += 1; }
                0x0A => { ldax(self, 'B'); self.pc += 1; }
                0x0B => { dcx(self, 'B'); self.pc += 1; }
                0x0C => { inr(self, 'C'); self.pc += 1; }
                0x0D => { dcr(self, 'C'); self.pc += 1; }
                0x0E => { mvi(self, 'C'); self.pc += 2; }
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
                0x11 => { lxi(self, 'D'); self.pc += 3; }
                0x12 => { stax(self, 'D');  self.pc += 1; }
                0x13 => { inx(self, 'D'); self.pc += 1; }
                0x14 => { inr(self, 'D'); self.pc += 1; }
                0x15 => { dcr(self, 'D'); self.pc += 1; }
                0x16 => { mvi(self, 'D'); self.pc += 2; }
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
                0x19 => { dad(self, 'D'); self.pc += 1; }
                0x1A => { ldax(self, 'D'); self.pc += 1; }
                0x1B => { dcx(self, 'D'); self.pc += 1; }
                0x1C => { inr(self, 'E'); self.pc += 1; }
                0x1D => { dcr(self, 'E'); self.pc += 1; }
                0x1E => { mvi(self, 'E'); self.pc += 2; }
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
                0x21 => { lxi(self, 'H'); self.pc += 3; }
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
                0x23 => { inx(self, 'H'); self.pc += 1; }
                0x24 => { inr(self, 'H'); self.pc += 1; }
                0x25 => { dcr(self, 'H'); self.pc += 1; }
                0x26 => { mvi(self, 'H'); self.pc += 2; }
                0x27 => {
                    // INSTRUCTION: DAA
                    // DESCRIPTION:
                    //      The DAA intruction adjusts the eight-bit value in the accumulator 
                    //      to form two four-bit binary coded decimal digits.

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
                    self.flags.parity = parity(self.regs.a as u16);

                    self.pc += 1;
                }
                0x28 => { self.pc += 1; }
                0x29 => { dad(self, 'H'); self.pc += 1; }
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
                0x2B => { dcx(self, 'H'); self.pc += 1; }
                0x2C => { inr(self, 'L'); self.pc += 1; }
                0x2D => { dcr(self, 'L'); self.pc += 1; }
                0x2E => { mvi(self, 'L'); self.pc += 2; }
                0x2F => {
                    // INSTRUCTION: CMA
                    // DESCRIPTION: 
                    //      Each bit of the contents of the accumulator is complemented 
                    //      (producing the one's complement). 

                    self.regs.a = !self.regs.a;

                    self.pc += 1;
                }


                0x30 => { self.pc += 1; }
                0x31 => { lxi(self, 'S'); self.pc += 3; }
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
                0x33 => { self.sp += 1; self.pc += 1; }
                0x34 => { inr(self, 'M');  self.pc += 1; }
                0x35 => { dcr(self, 'M'); self.pc += 1; }
                0x36 => { mvi(self, 'M'); self.pc += 2; }
                0x37 => { self.flags.carry = 1; self.pc += 1; }
                0x38 => { self.pc += 1; }
                0x39 => { dad(self, 'S'); self.pc += 1;}
                0x3A => {
                    // INSTRUCTION: LDA
                    // DESCRIPTION: 
                    //      LDA load~ the accumulator with a copy of the byte at the location 
                    //      specified In bytes two and three of the LDA instruction.
                    let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                (self.memory[self.pc + 1] as u16)) as usize;

                    println!("{:04x}: LDA {:04x}", self.pc, addr);
                    self.regs.a = self.memory[addr];

                    self.pc += 3;
                }
                0x3B => { self.sp -= 1; self.pc += 1; }
                0x3C => { inr(self, 'A'); self.pc += 1; }
                0x3D => { dcr(self, 'A'); self.pc += 1; }
                0x3E => { mvi(self, 'A'); self.pc += 2; }
                0x3F => { self.flags.carry = !self.flags.carry; self.pc += 1; }


                0x40 => { self.pc += 1; }
                0x41 => { self.regs.b = self.regs.c; self.pc += 1; }
                0x42 => { self.regs.b = self.regs.d; self.pc += 1; }
                0x43 => { self.regs.b = self.regs.e; self.pc += 1; }
                0x44 => { self.regs.b = self.regs.h; self.pc += 1; }
                0x45 => { self.regs.b = self.regs.l; self.pc += 1; }
                0x46 => {
                    // INSTRUCTION: MOV B, M
                    // DESCRIPTION: move from memory into B
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    self.regs.b = self.memory[addr];

                    self.pc += 1;
                }
                0x47 => { self.regs.b = self.regs.a; self.pc += 1; }
                0x48 => { self.regs.c = self.regs.b; self.pc += 1; }
                0x49 => { self.pc += 1; }
                0x4A => { self.regs.c = self.regs.d; self.pc += 1; }
                0x4B => { self.regs.c = self.regs.e; self.pc += 1; }
                0x4C => { self.regs.c = self.regs.h; self.pc += 1; }
                0x4D => { self.regs.c = self.regs.l; self.pc += 1; }
                0x4E => {
                    // INSTRUCTION: MOV C, M
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    self.regs.c = self.memory[addr];

                    self.pc += 1;
                }
                0x4F => { self.regs.c = self.regs.a; self.pc += 1; }


                0x50 => { self.regs.d = self.regs.b; self.pc += 1; }
                0x51 => { self.regs.d = self.regs.c; self.pc += 1; }
                0x52 => { self.pc += 1; }
                0x53 => { self.regs.d = self.regs.e; self.pc += 1; }
                0x54 => { self.regs.d = self.regs.h; self.pc += 1; }
                0x55 => { self.regs.d = self.regs.l; self.pc += 1; }
                0x56 => {
                    // INSTRUCTION: MOV D, M
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    self.regs.d = self.memory[addr];

                    self.pc += 1;
                }
                0x57 => { self.regs.d = self.regs.a; self.pc += 1; }
                0x58 => { self.regs.e = self.regs.b; self.pc += 1; }
                0x59 => { self.regs.e = self.regs.c; self.pc += 1; }
                0x5A => { self.regs.e = self.regs.d; self.pc += 1; }
                0x5B => { self.pc += 1; }
                0x5C => { self.regs.e = self.regs.h; self.pc += 1; }
                0x5D => { self.regs.e = self.regs.l; self.pc += 1; }
                0x5E => {
                    // INSTRUCTION: MOV E, M
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    self.regs.e = self.memory[addr];

                    self.pc += 1;
                }
                0x5F => { self.regs.e = self.regs.a; self.pc += 1; }

                
                0x60 => { self.regs.h = self.regs.b; self.pc += 1; }
                0x61 => { self.regs.h = self.regs.c; self.pc += 1; }
                0x62 => { self.regs.h = self.regs.d; self.pc += 1; }
                0x63 => { self.regs.h = self.regs.e; self.pc += 1; }
                0x64 => { self.pc += 1; }
                0x65 => { self.regs.h = self.regs.l; self.pc += 1; }
                0x66 => {
                    // INSTRUCTION: MOV H, M
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    self.regs.h = self.memory[addr];

                    self.pc += 1;
                }
                0x67 => { self.regs.h = self.regs.a; self.pc += 1; }
                0x68 => { self.regs.l = self.regs.b; self.pc += 1; }
                0x69 => { self.regs.l = self.regs.c; self.pc += 1; }
                0x6A => { self.regs.l = self.regs.d; self.pc += 1; }
                0x6B => { self.regs.l = self.regs.e; self.pc += 1; }
                0x6C => { self.regs.l = self.regs.h; self.pc += 1; }
                0x6D => { self.pc += 1; }
                0x6E => {
                    // INSTRUCTION: MOV L, M
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    self.regs.l = self.memory[addr];

                    self.pc += 1;
                }
                0x6F => { self.regs.l = self.regs.a; self.pc += 1; }


                0x70 => { mov_m(self, 'B'); self.pc += 1; }
                0x71 => { mov_m(self, 'C'); self.pc += 1; }
                0x72 => { mov_m(self, 'D'); self.pc += 1; }
                0x73 => { mov_m(self, 'E'); self.pc += 1; }
                0x74 => { mov_m(self, 'H'); self.pc += 1; }
                0x75 => { mov_m(self, 'L'); self.pc += 1; }
                0x76 => {}
                0x77 => { mov_m(self, 'A'); self.pc += 1; }
                0x78 => { self.regs.a = self.regs.b; self.pc += 1; }
                0x79 => { self.regs.a = self.regs.c; self.pc += 1; }
                0x7A => { self.regs.a = self.regs.d; self.pc += 1; }
                0x7B => { self.regs.a = self.regs.e; self.pc += 1; }
                0x7C => { self.regs.a = self.regs.h; self.pc += 1; }
                0x7D => { self.regs.a = self.regs.l; self.pc += 1; }
                0x7E => {
                    // INSTRUCTION: MOV A, M
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    self.regs.a = self.memory[addr];

                    self.pc += 1;
                }
                0x7F => { self.pc += 1; }
            

                0x80 => { add_to_accu(self, self.regs.b); self.pc += 1; }
                0x81 => { add_to_accu(self, self.regs.c); self.pc += 1; }
                0x82 => { add_to_accu(self, self.regs.d); self.pc += 1; }
                0x83 => { add_to_accu(self, self.regs.e); self.pc += 1; }
                0x84 => { add_to_accu(self, self.regs.h); self.pc += 1; }
                0x85 => { add_to_accu(self, self.regs.l); self.pc += 1; }
                0x86 => {
                    // INSTRUCTION: ADD M
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    add_to_accu(self, self.memory[addr]);

                    self.pc += 1;
                }
                0x87 => { add_to_accu(self, self.regs.a); self.pc += 1; }
                0x88 => { adc(self, self.regs.b); self.pc += 1; }
                0x89 => { adc(self, self.regs.c); self.pc += 1; }
                0x8A => { adc(self, self.regs.d); self.pc += 1; }
                0x8B => { adc(self, self.regs.e); self.pc += 1; }
                0x8C => { adc(self, self.regs.h); self.pc += 1; }
                0x8D => { adc(self, self.regs.l); self.pc += 1; }
                0x8E => {
                    // INSTRUCTION: ADC M
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    adc(self, self.memory[addr]);

                    self.pc += 1;
                }
                0x8F => { adc(self, self.regs.a); self.pc += 1; }
                

                0x90 => { sub_accu(self, self.regs.b); self.pc += 1; }
                0x91 => { sub_accu(self, self.regs.c); self.pc += 1; }
                0x92 => { sub_accu(self, self.regs.d); self.pc += 1; }
                0x93 => { sub_accu(self, self.regs.e); self.pc += 1; }
                0x94 => { sub_accu(self, self.regs.h); self.pc += 1; }
                0x95 => { sub_accu(self, self.regs.l); self.pc += 1; }
                0x96 => {
                    // INSTRUCTION: SUB M
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    sub_accu(self, self.memory[addr]); 

                    self.pc += 1;
                }
                0x97 => { sub_accu(self, self.regs.a); self.pc += 1; }
                0x98 => { sbb(self, self.regs.b); self.pc += 1; }
                0x99 => { sbb(self, self.regs.c); self.pc += 1; }
                0x9A => { sbb(self, self.regs.d); self.pc += 1; }
                0x9B => { sbb(self, self.regs.e); self.pc += 1; }
                0x9C => { sbb(self, self.regs.h); self.pc += 1; }
                0x9D => { sbb(self, self.regs.l); self.pc += 1; }
                0x9E => {
                    // INSTRUCTION: SBB M
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    sbb(self, self.memory[addr]);

                    self.pc += 1;
                }
                0x9F => { sbb(self, self.regs.a); self.pc += 1; }


                0xA0 => { ana(self, self.regs.b); self.pc += 1; }
                0xA1 => { ana(self, self.regs.c); self.pc += 1; }
                0xA2 => { ana(self, self.regs.d); self.pc += 1; }
                0xA3 => { ana(self, self.regs.e); self.pc += 1; }
                0xA4 => { ana(self, self.regs.h); self.pc += 1; }
                0xA5 => { ana(self, self.regs.l); self.pc += 1; }
                0xA6 => {
                    // INSTRUCTION: ANA M
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    ana(self, self.memory[addr]);

                    self.pc += 1;
                }
                0xA7 => { println!("{:04x}: ANA A", self.pc); ana(self, self.regs.a); self.pc += 1; }
                0xA8 => { xra(self, self.regs.b); self.pc += 1; }
                0xA9 => { xra(self, self.regs.c); self.pc += 1; }
                0xAA => { xra(self, self.regs.d); self.pc += 1; }
                0xAB => { xra(self, self.regs.e); self.pc += 1; }
                0xAC => { xra(self, self.regs.h); self.pc += 1; }
                0xAD => { xra(self, self.regs.l); self.pc += 1; }
                0xAE => {
                    // INSTRUCTION: XRA M
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    xra(self, self.memory[addr]);

                    self.pc += 1;
                }
                0xAF => { xra(self, self.regs.a); self.pc += 1; }

                0xB0 => { ora(self, self.regs.b); self.pc += 1; }
                0xB1 => { ora(self, self.regs.c); self.pc += 1; }
                0xB2 => { ora(self, self.regs.d); self.pc += 1; }
                0xB3 => { ora(self, self.regs.e); self.pc += 1; }
                0xB4 => { ora(self, self.regs.h); self.pc += 1; }
                0xB5 => { ora(self, self.regs.l); self.pc += 1; }
                0xB6 => {
                    // INSTRUCTION: ORA C
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    ora(self, self.memory[addr]);

                    self.pc += 1;
                }
                0xB7 => { ora(self, self.regs.a); self.pc += 1; }
                0xB8 => { cmp(self, self.regs.b); self.pc += 1; }
                0xB9 => { cmp(self, self.regs.c); self.pc += 1; }
                0xBA => { cmp(self, self.regs.d); self.pc += 1; }
                0xBB => { cmp(self, self.regs.e); self.pc += 1; }
                0xBC => { cmp(self, self.regs.h); self.pc += 1; }
                0xBD => { cmp(self, self.regs.l); self.pc += 1; }
                0xBE => {
                    // INSTRUCTION: CMP M
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    cmp(self, self.memory[addr]);

                    self.pc += 1;
                }
                0xBF => { cmp(self, self.regs.a); self.pc += 1; }


                0xC0 => {
                    // INSTRUCTION: RNZ
                    if self.flags.zero == 0 {
                        let lsb = self.memory[self.sp];
                        let msb = self.memory[self.sp + 1];

                        let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                        self.pc = addr;
                        self.sp += 2;
                    } else {
                        self.pc += 1;
                    }
                }
                0xC1 => { pop(self, 'B'); self.pc += 1; }
                0xC2 => {
                    // INSTRUCTION: JNZ
                    if self.flags.zero == 0 {
                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        println!("{:04x}: JNZ {:04x}", self.pc, addr);
                        self.pc = addr;
                    } else {
                        self.pc += 3;
                    }
                }
                0xC3 => {
                    // INSTRUCTION: JMP
                    let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                (self.memory[self.pc + 1] as u16)) as usize;

                    self.pc = addr;
                }
                0xC4 => {
                    // INSTRUCTION: CNZ
                    if self.flags.zero == 0 {
                        let next_instr_addr = self.pc + 3; // Address of the next instruction
                        let msb = ((next_instr_addr & 0xff00) >> 8) as u8;
                        let lsb = (next_instr_addr & 0x00ff) as u8;

                        self.memory[self.sp - 1] = msb; 
                        self.memory[self.sp - 2] = lsb;

                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                        self.sp -= 2;
                    } else { self.pc += 3; }
                }
                0xC5 => { push(self, 'B'); self.pc += 1; }
                0xC6 => {
                    // INSTRUCTION: ADI
                    let result = (self.regs.a as u16) + (self.memory[self.pc + 1] as u16);
                    
                    self.flags.carry = (result > 0xff) as u8;
                    self.flags.zero = (((result as u8) & 0xff) == 0) as u8;
                    self.flags.sign = (((result as u8) & 0x80) != 0) as u8;
                    self.flags.parity = parity(result);

                    self.regs.a = result as u8;
                    self.pc += 2;
                }
                0xC7 => { rst(self, 0); }
                0xC8 => {
                    // INSTRUCTION: RZ
                    if self.flags.zero == 1 {
                        let lsb = self.memory[self.sp];
                        let msb = self.memory[self.sp + 1];

                        let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                        self.pc = addr;
                        self.sp += 2;

                    } else {
                        self.pc += 1;
                    }
                }
                0xC9 => {
                    // INSTRUCTION: RET
                    let lsb = self.memory[self.sp];
                    let msb = self.memory[self.sp + 1];

                    let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                    self.pc = addr;
                    self.sp += 2;
                }
                0xCA => {
                    // INSTRUCTION: JZ
                    if self.flags.zero == 1 {
                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                    } else {
                        self.pc += 3;
                    }
                }
                0xCB => { self.pc += 1; }
                0xCC => {
                    // INSTRUCTION: CZ
                    if self.flags.zero == 1 {
                        let next_instr_addr = self.pc + 3;
                        let msb = ((next_instr_addr & 0xff00) >> 8) as u8;
                        let lsb = (next_instr_addr & 0x00ff) as u8;

                        self.memory[self.sp - 1] = msb; 
                        self.memory[self.sp - 2] = lsb;

                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                        self.sp -= 2;
                    } else { self.pc += 3; }
                }
                0xCD => {
                    // INSTRUCTION: CALL
                    let next_instr_addr = self.pc + 3;
                    let msb = ((next_instr_addr & 0xff00) >> 8) as u8;
                    let lsb = (next_instr_addr & 0x00ff) as u8;

                    self.memory[self.sp - 1] = msb; 
                    self.memory[self.sp - 2] = lsb;

                    let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                (self.memory[self.pc + 1] as u16)) as usize;

                    self.pc = addr;
                    self.sp -= 2;
                }
                0xCE => {
                    // INSTRUCTION: ACI
                    let result = (self.regs.a as u16) + (self.memory[self.pc + 1] as u16 + 
                                                            self.flags.carry as u16);
                    
                    self.flags.carry = (result > 0xff) as u8;
                    self.flags.zero = (((result as u8) & 0xff) == 0) as u8;
                    self.flags.sign = (((result as u8) & 0x80) != 0) as u8;
                    self.flags.parity = parity(result);

                    self.regs.a = result as u8;
                    self.pc += 2;
                }
                0xCF => { rst(self, 1); }


                0xD0 => {
                    // INSTRUCTION: RNC
                    if self.flags.carry == 0 {
                        let lsb = self.memory[self.sp];
                        let msb = self.memory[self.sp + 1];

                        let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                        self.pc = addr;
                        self.sp += 2;
                    } else {
                        self.pc += 1;
                    }
                }
                0xD1 => { pop(self, 'D'); self.pc += 1; }
                0xD2 => {
                    // INSTRUCTION: JNC
                    if self.flags.carry == 0 {
                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                    } else {
                        self.pc += 3;
                    }
                }
                0xD3 => { self.pc += 1; }
                0xD4 => {
                    // INSTRUCTION: CNC
                    if self.flags.carry == 0 {
                        let next_instr_addr = self.pc + 3;
                        let msb = ((next_instr_addr & 0xff00) >> 8) as u8;
                        let lsb = (next_instr_addr & 0x00ff) as u8;

                        self.memory[self.sp - 1] = msb; 
                        self.memory[self.sp - 2] = lsb;

                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                        self.sp -= 2;
                    } else {
                        self.pc += 3;
                    }
                }
                0xD5 => { push(self, 'D'); self.pc += 1; }
                0xD6 => {
                    // INSTRUCTION: SUI
                    let result = (self.regs.a as u16) - (self.memory[self.pc + 1] as u16);
                    
                    self.flags.carry = (result > 0xff) as u8;
                    self.flags.zero = (((result as u8) & 0xff) == 0) as u8;
                    self.flags.sign = (((result as u8) & 0x80) != 0) as u8;
                    self.flags.parity = parity(result);

                    self.regs.a = result as u8;
                    self.pc += 2;
                }
                0xD7 => { rst(self, 2); }
                0xD8 => {
                    // INSTRUCTION: RC
                    if self.flags.carry == 1 {
                        let lsb = self.memory[self.sp];
                        let msb = self.memory[self.sp + 1];

                        let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                        self.pc = addr;
                        self.sp += 2;
                    } else { self.pc += 1; }
                }
                0xD9 => { self.pc += 1; }
                0xDA => {
                    // INSTRUCTION: JC
                    if self.flags.carry == 1 {
                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                    } else {
                        self.pc += 3;
                    }
                }
                0xDB => { self.pc += 1; }
                0xDC => {
                    // INSTRUCTION: CC
                    if self.flags.carry == 1 {
                        let next_instr_addr = self.pc + 3;
                        let msb = ((next_instr_addr & 0xff00) >> 8) as u8;
                        let lsb = (next_instr_addr & 0x00ff) as u8;

                        self.memory[self.sp - 1] = msb; 
                        self.memory[self.sp - 2] = lsb;

                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                        self.sp -= 2;
                    } else {
                        self.pc += 3;
                    }
                }
                0xDD => { self.pc += 1; }
                0xDE => {
                    // INSTRUCTION: SBI
                    let result = (self.regs.a as u16) - (self.memory[self.pc + 1] as u16 + 
                                                            self.flags.carry as u16);
                    
                    self.flags.carry = (result > 0xff) as u8;
                    self.flags.zero = (((result as u8) & 0xff) == 0) as u8;
                    self.flags.sign = (((result as u8) & 0x80) != 0) as u8;
                    self.flags.parity = parity(result);

                    self.regs.a = result as u8;
                    self.pc += 2;
                }
                0xDF => { rst(self, 3); }


                0xE0 => {
                    // INSTRUCTION: RPO
                    if self.flags.parity == 0 {
                        let lsb = self.memory[self.sp];
                        let msb = self.memory[self.sp + 1];

                        let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                        self.pc = addr;
                        self.sp += 2;
                    } else {
                        self.pc += 1;
                    }
                }
                0xE1 => { pop(self, 'H'); self.pc += 1; }
                0xE2 => {
                    // INSTRUCTION: JPO
                    if self.flags.parity == 0 {
                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                    } else {
                        self.pc += 3;
                    }
                }
                0xE3 => {
                    // INSTRUCTION: XTHL
                    let lsb = self.memory[self.sp];
                    let msb = self.memory[self.sp + 1];
                    self.sp += 2;

                    self.memory[self.sp - 1] = self.regs.l;
                    self.memory[self.sp - 2] = self.regs.h;
                    self.sp -= 2;

                    self.regs.l = lsb;
                    self.regs.h = msb;

                    self.pc += 1;
                }
                0xE4 => {
                    // INSTRUCTION: CPO
                    if self.flags.parity == 0 {
                        self.pc += 3; // Address of the next instruction
                        let msb = ((self.pc & 0xff00) >> 8) as u8;
                        let lsb = (self.pc & 0x00ff) as u8;

                        self.memory[self.sp - 1] = lsb; 
                        self.memory[self.sp - 2] = msb;

                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                        self.sp -= 2;
                    } else {
                        self.pc += 3;
                    }
                }
                0xE5 => { push(self, 'H'); self.pc += 1; }
                0xE6 => {
                    // INSTRUCTION: ANI
                    let result = (self.regs.a as u16) & (self.memory[self.pc + 1] as u16);
                    
                    self.flags.carry = (result > 0xff) as u8;
                    self.flags.zero = (((result as u8) & 0xff) == 0) as u8;
                    self.flags.sign = (((result as u8) & 0x80) != 0) as u8;
                    self.flags.parity = parity(result);

                    self.regs.a = result as u8;
                    self.pc += 2;
                }
                0xE7 => { rst(self, 4); }
                0xE8 => {
                    // INSTRUCTION: RPE
                    if self.flags.parity == 1 {
                        let lsb = self.memory[self.sp];
                        let msb = self.memory[self.sp + 1];

                        let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                        self.pc = addr;
                        self.sp += 2;
                    } else {
                        self.pc += 1;
                    }
                }
                0xE9 => {
                    // INSTRUCTION: PCHL
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    self.pc = addr;
                }
                0xEA => {
                    // INSTRUCTION: JPE
                    if self.flags.parity == 1 {
                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                    } else {
                        self.pc += 3;
                    }
                }
                0xEB => {
                    // INSTRUCTION: XCHG
                    let (d, e) = (self.regs.d, self.regs.d);

                    self.regs.d = self.regs.h;
                    self.regs.e = self.regs.l;

                    self.regs.h = d;
                    self.regs.l = e;

                    self.pc += 1;
                }
                0xEC => {
                    // INSTRUCTION: CPE
                    if self.flags.parity == 1 {
                        self.pc += 3; // Address of the next instruction
                        let msb = ((self.pc & 0xff00) >> 8) as u8;
                        let lsb = (self.pc & 0x00ff) as u8;

                        self.memory[self.sp - 1] = lsb; 
                        self.memory[self.sp - 2] = msb;

                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                        self.sp -= 2;
                    } else {
                        self.pc += 3;
                    }
                }
                0xED => { self.pc += 1; }
                0xEE => {
                    // INSTRUCTION: XRI
                    let result = (self.regs.a as u16) ^ (self.memory[self.pc + 1] as u16);
                    
                    self.flags.carry = (result > 0xff) as u8;
                    self.flags.zero = (((result as u8) & 0xff) == 0) as u8;
                    self.flags.sign = (((result as u8) & 0x80) != 0) as u8;
                    self.flags.parity = parity(result);

                    self.regs.a = result as u8;
                    self.pc += 2;
                }
                0xEF => { rst(self, 5); }


                0xF0 => {
                    // INSTRUCTION: RP
                    if self.flags.sign == 0 {
                        let lsb = self.memory[self.sp];
                        let msb = self.memory[self.sp + 1];

                        let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                        self.pc = addr;
                        self.sp += 2;
                    } else {
                        self.pc += 1;
                    }
                }
                0xF1 => { pop(self, 'P'); self.pc += 1; }
                0xF2 => {
                    // INSTRUCTION: JP
                    if self.flags.sign == 1 {
                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                    } else {
                        self.pc += 3;
                    }
                }
                0xF3 => {
                    // INSTRUCTION: DI

                    // disable interrupts
                    self.int_enable = 0;
                    self.pc += 1;
                }
                0xF4 => {
                    // INSTRUCTION: CP
                    if self.flags.sign == 0 {
                        self.pc += 3; // Address of the next instruction
                        let msb = ((self.pc & 0xff00) >> 8) as u8;
                        let lsb = (self.pc & 0x00ff) as u8;

                        self.memory[self.sp - 1] = lsb;
                        self.memory[self.sp - 2] = msb;

                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                        self.sp += 2;
                    } else {
                        self.pc += 3;
                    }
                }   
                0xF5 => { push(self, 'P'); self.pc += 1; }
                0xF6 => {
                    // INSTRUCTION: ORI
                    let result = (self.regs.a as u16) | (self.memory[self.pc + 1] as u16);
                    
                    self.flags.carry = (result > 0xff) as u8;
                    self.flags.zero = (((result as u8) & 0xff) == 0) as u8;
                    self.flags.sign = (((result as u8) & 0x80) != 0) as u8;
                    self.flags.parity = parity(result);

                    self.regs.a = result as u8;
                    self.pc += 2;
                }
                0xF7 => { rst(self, 6); }
                0xF8 => {
                    // INSTRUCTION: RM
                    if self.flags.sign == 1 {
                        let lsb = self.memory[self.sp];
                        let msb = self.memory[self.sp + 1];

                        let addr = (((msb as u16) << 8) | (lsb as u16)) as usize;
                        self.pc = addr;
                        self.sp += 2;
                    } else {
                        self.pc += 1;
                    }
                }
                0xF9 => {
                    // INSTRUCTION: SPHL
                    let addr = (((self.regs.h as u16) << 8) | (self.regs.l as u16)) as usize;
                    self.sp = addr;

                    self.pc += 1;
                }
                0xFA => {
                    // INSTRUCTION: JM
                    if self.flags.sign == 1 {
                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                    } else {
                        self.pc += 3;
                    }
                }
                0xFB => {
                    // INSTRUCTION: EI

                    // enable interrupts
                    self.int_enable = 1;

                    self.pc += 1;
                }
                0xFC => {
                    // INSTRUCTION: CM
                    if self.flags.sign == 1 {
                        self.pc += 3; // Address of the next instruction
                        let msb = ((self.pc & 0xff00) >> 8) as u8;
                        let lsb = (self.pc & 0x00ff) as u8;

                        self.memory[self.sp - 1] = lsb;
                        self.memory[self.sp - 2] = msb;

                        let addr = (((self.memory[self.pc + 2] as u16) << 8) | 
                                    (self.memory[self.pc + 1] as u16)) as usize;

                        self.pc = addr;
                        self.sp += 2;
                    } else {
                        self.pc += 3;
                    }
                }
                0xFD => { self.pc += 1; }
                0xFE => {
                    // INSTRUCTION: CPI
                    let result = (self.regs.a as i16) - (self.memory[self.pc + 1] as i16);
                    
                    self.flags.carry = (self.regs.a < self.memory[self.pc + 1]) as u8;
                    self.flags.zero = (result == 0) as u8;
                    self.flags.sign = (((result as u8) & 0x80) != 0) as u8;
                    self.flags.parity = parity(result as u16);

                    self.pc += 2;
                }
                0xFF => { rst(self, 7); }
            }
        }
    }
}