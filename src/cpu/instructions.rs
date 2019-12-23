use crate::cpu::intel8080::Intel8080;
use crate::cpu::utils::*;


pub fn add_to_accu(state: &mut Intel8080, byte: u8) {
    // INSTRUCTION: ADD byte
    // DESCRIPTION: 
    //      The ADD In,truction adds one byte of data to the contents of the 
    //      accumulatoL The result is stored in the accumulator Notice that the 
    //      ADD instruction excludes the carry flag from the addition but sets the 
    //      flag to indicate the Jutcome of the operation.

    let result = (state.regs.a as u16) + (byte as u16);

    state.flags.carry = (result > 0xff) as u8;
    state.flags.zero = (((result as u8) & 0xff) == 0) as u8;
    state.flags.sign = (((result as u8) & 0x80) != 0) as u8;
    state.flags.parity = parity(result);

    state.regs.a = result as u8;
}

pub fn adc(state: &mut Intel8080, byte: u8) {
    // INSTRUCTION: ADC byte
    // DESCRIPTION: 
    //      The ADC inst ruction adds one byte of data plus the setting of the 
    //      carry flag to the contents of the accumulator. The result istored 
    //      in the accumulator ADC then updates the setting of the carry flag 
    //      to indicate the outcome of the operaton.

    let result = (state.regs.a as u16) + (byte as u16) + (state.flags.carry as u16);

    state.flags.carry = (result > 0xff) as u8;
    state.flags.zero = (((result as u8) & 0xff) == 0) as u8;
    state.flags.sign = (((result as u8) & 0x80) != 0) as u8;
    state.flags.parity = parity(result);

    state.regs.a = result as u8;
}

pub fn lxi(state: &mut Intel8080, byte: char) {
    // INSTRUCTION: LXI byte
    // DESCRIPTION:
    //      LXI is a three-byte instruction; its second and third bytes contain the source 
    //      data to be loaded into a register pair. LXI loads a register pair by copying its 
    //      second and third bytes into the specified destination register pair.

    match byte {
        'B' => {
            // load bytes into register B and C
            state.regs.b = state.memory[state.pc + 2];
            state.regs.c = state.memory[state.pc + 1];
        }
        'D' => {
            // load bytes into register D and E
            state.regs.d = state.memory[state.pc + 2];
            state.regs.e = state.memory[state.pc + 1];
        }
        'H' => {
            // load bytes into register H and L
            state.regs.h = state.memory[state.pc + 2];
            state.regs.l = state.memory[state.pc + 1];
        }
        'S' => {
            // load bytes into th stack pointer (SP)
            state.sp = (((state.memory[state.pc + 2] as u16) << 8) | 
                        (state.memory[state.pc + 1] as u16)) as usize;
        }
        _ => {}
    }
}

pub fn stax(state: &mut Intel8080, byte: char) {
    // INSTRUCTION: STAX byte
    // DESCRIPTION: 
    //      The STAX ins :ruction stores a copy of the contents of the accumulator into the memory location addressed
    //      by register pai B or register pair D.

    let mut addr = 0;
    match byte {
        'B' => {
            // get the content of register pair B and C
            // format them into an address in LE format.
            addr = (((state.regs.b as u16) << 8) | 
                    (state.regs.c as u16)) as usize;
        }
        'D' => {
            // get the content of register pair B and C
            // format them into an address in LE format.
            addr = (((state.regs.d as u16) << 8) | 
                    (state.regs.e as u16)) as usize;
        }
        _ => {}
    }
    
    // get the value in the A register and store this
    // value at the address created in the previous step.
    state.memory[addr] = state.regs.a;
}

pub fn mvi(state: &mut Intel8080, byte: char) {
    // INSTRUCTION: MVI byte
    // DESCRIPTION:
    //      the immediate data byte is stored in register specified. 
    //      No condition flags are affected. 

    match byte {
        'B' => { state.regs.b = state.memory[state.pc + 1]; }
        'C' => { state.regs.c = state.memory[state.pc + 1]; }
        'D' => { state.regs.d = state.memory[state.pc + 1]; }
        'E' => { state.regs.e = state.memory[state.pc + 1]; }
        'H' => { state.regs.h = state.memory[state.pc + 1]; }
        'L' => { state.regs.l = state.memory[state.pc + 1]; }
        'A' => { state.regs.a = state.memory[state.pc + 1]; }
        'M' => { 
            let addr = (((state.regs.h as u16) << 8) | (state.regs.l as u16)) as usize;
            state.memory[addr] = state.memory[state.pc + 1]; 
        }
        _ => {}
    }   
}

pub fn inx(state: &mut Intel8080, byte: char) {
    // INSTRUCTION: INX byte
    // DESCRIPTION: 
    //      INX adds one to the contents of the specified register pair.

    match byte {
        'B' => {
            let value = (((state.regs.b as u16) << 8) | (state.regs.c as u16)) + 1;

            state.regs.b = ((value & 0xff00) >> 8) as u8;
            state.regs.c = (value & 0x00ff) as u8;
        }
        'D' => {
            let value = (((state.regs.d as u16) << 8) | (state.regs.e as u16)) + 1;

            state.regs.d = ((value & 0xff00) >> 8) as u8;
            state.regs.e = (value & 0x00ff) as u8;
        }
        'H' => {
            let value = (((state.regs.h as u16) << 8) | (state.regs.l as u16)) + 1;

            state.regs.h = ((value & 0xff00) >> 8) as u8;
            state.regs.l = (value & 0x00ff) as u8;
        }
        _ => {}
    }

}

pub fn inr(state: &mut Intel8080, byte: char) {
    // INSTRUCTION: INR byte
    // DESCRIPTION: 
    //      Increment the specified register by 1;

    let mut result = 0;
    match byte {
        'B' => { result = (state.regs.b as u16) + 1; state.regs.b = result as u8; }
        'C' => { result = (state.regs.c as u16) + 1; state.regs.c = result as u8; }
        'D' => { result = (state.regs.d as u16) + 1; state.regs.d = result as u8; }
        'E' => { result = (state.regs.e as u16) + 1; state.regs.e = result as u8; }
        'H' => { result = (state.regs.h as u16) + 1; state.regs.h = result as u8; }
        'L' => { result = (state.regs.l as u16) + 1; state.regs.l = result as u8; }
        'A' => { result = (state.regs.a as u16) + 1; state.regs.a = result as u8; }
        'M' => {
            let addr = (((state.regs.h as u16) << 8) | (state.regs.l as u16)) as usize;
            let result = (state.memory[addr] as u16) + 1;
            
            state.memory[addr] = result as u8;
        }   
        _ => {}
    }
    
    state.flags.zero = (((result as u8) & 0xff) == 0) as u8;
    state.flags.sign = (((result as u8) & 0x80) != 0) as u8;
    state.flags.parity = parity(result);
}

pub fn dcr(state: &mut Intel8080, byte: char) {
    // INSTRUCTION: DCR byte
    // DESCRIPTION:
    //      The value in the specified register is decremented by 1;

    let mut result = 0;
    match byte {
        'B' => { result = (state.regs.b as i16) - 1; state.regs.b = result as u8; }
        'C' => { result = (state.regs.c as i16) - 1; state.regs.c = result as u8; }
        'D' => { result = (state.regs.d as i16) - 1; state.regs.d = result as u8; }
        'E' => { result = (state.regs.e as i16) - 1; state.regs.e = result as u8; }
        'H' => { result = (state.regs.h as i16) - 1; state.regs.h = result as u8; }
        'L' => { result = (state.regs.l as i16) - 1; state.regs.l = result as u8; }
        'A' => { result = (state.regs.a as i16) - 1; state.regs.a = result as u8; }
        'M' => {
                let addr = (((state.regs.h as u16) << 8) | (state.regs.l as u16)) as usize;
                let result = (state.memory[addr] as u16) - 1;

                state.memory[addr] = result as u8;
        }
        _ => {}
    }

    state.flags.zero = (((result as u8) & 0xff) == 0) as u8;
    state.flags.sign = (((result as u8) & 0x80) != 0) as u8;
    state.flags.parity = parity(result as u16);
}

pub fn dad(state: &mut Intel8080, byte: char) {
    // INSTRUCTION: DAD byte
    // DESCRIPTION:
    //      The 16-bit number in the specified register pair is added to the 
    //      16-bit number held in the H and L registers using two's complement 
    //      arithmetic. The result replaces the contents in the H and L registers. 

    let mut result = 0;
    let hl = ((state.regs.h as u32) << 8) | (state.regs.l as u32);
    match byte {
        'B' => {
            let bc = ((state.regs.b as u32) << 8) | (state.regs.c as u32);
            
            result =  bc + hl;
            state.regs.h = ((result & 0x0000ff00) >> 8) as u8;
            state.regs.l = (result & 0x000000ff) as u8;
        }
        'D' => {
            let de = ((state.regs.d as u32) << 8) | (state.regs.e as u32);

            result =  de + hl;
            state.regs.h = ((result & 0x0000ff00) >> 8) as u8;
            state.regs.l = (result & 0x000000ff) as u8;
        }
        'H' => {
            let result =  hl << 1;
            state.regs.h = ((result & 0x0000ff00) >> 8) as u8;
            state.regs.l = (result & 0x000000ff) as u8;
        }
        'S' => {
            let result =  (state.sp as u32) + hl;
            state.regs.h = ((result & 0x0000ff00) >> 8) as u8;
            state.regs.l = (result & 0x000000ff) as u8;
        }
        _ => {}
    }

    // set the carry flag
    state.flags.carry = ((result & 0xffff0000) > 0) as u8;
}

pub fn ldax(state: &mut Intel8080, byte: char) {
    // INSTRUCTION: LDAX byte
    // DESCRIPTION: 
    //      The contents of the memory location addressed by the specified register
    //      pair replace the contents of the accumulator.
    
    let mut addr = 0;
    match byte {
        'B' => { addr = (((state.regs.b as u16) << 8) | ((state.regs.c) as u16)) as usize; }
        'D' => { addr = (((state.regs.d as u16) << 8) | ((state.regs.e) as u16)) as usize; }
        _ => {}
    }

    state.regs.a = state.memory[addr];
}

pub fn dcx(state: &mut Intel8080, byte: char) {
    // INSTRUCTION: DCX B
    // DESCRIPTION: 
    //      The 16-bit number held in the specified register pair is decremented by one.

    match byte {
        'B' => {
            let value = (((state.regs.b as u16) << 8) | (state.regs.c as u16)) - 1;

            state.regs.b = ((value & 0xff00) >> 8) as u8;
            state.regs.c = (value & 0x00ff) as u8;
        }
        'D' => {
            let value = (((state.regs.d as u16) << 8) | (state.regs.e as u16)) - 1;

            state.regs.d = ((value & 0xff00) >> 8) as u8;
            state.regs.e = (value & 0x00ff) as u8;
        }
        'H' => {
            let value = (((state.regs.h as u16) << 8) | (state.regs.l as u16)) - 1;

            state.regs.h = ((value & 0xff00) >> 8) as u8;
            state.regs.l = (value & 0x00ff) as u8;
        }
        _ => {}
    }
}

pub fn mov_m(state: &mut Intel8080, byte: char) {
    let addr = (((state.regs.h as u16) << 8) | (state.regs.l as u16)) as usize;
    match byte {
        'B' => { state.memory[addr] = state.regs.b; }
        'C' => { state.memory[addr] = state.regs.c; }
        'D' => { state.memory[addr] = state.regs.d; }
        'E' => { state.memory[addr] = state.regs.e; }
        'H' => { state.memory[addr] = state.regs.h; }
        'L' => { state.memory[addr] = state.regs.l; }
        'A' => { state.memory[addr] = state.regs.a; }
        _ => {}
    }
}

pub fn sub_accu(state: &mut Intel8080, byte: u8) {
    let result: u16 = (state.regs.a - byte) as u16;

    state.flags.carry = (result > 0xff) as u8;
    state.flags.zero = (((result as u8) & 0xff) == 0) as u8;
    state.flags.sign = (((result as u8) & 0x80) != 0) as u8;
    state.flags.parity = parity(result as u16);

    state.regs.a = result as u8;
}

pub fn sbb(state: &mut Intel8080, byte: u8) {
    let result = (state.regs.a as u16) - ((byte as u16) + (state.flags.carry as u16));

    state.flags.carry = (result > 0xff) as u8;
    state.flags.zero = ((result as u16 & 0xffff) == 0) as u8;
    state.flags.sign = ((result as u16 & 0x8000) != 0) as u8;
    state.flags.parity = parity(result);

    state.regs.a = result as u8;
}

pub fn ana(state: &mut Intel8080, byte: u8) {
    let result = (state.regs.a as u16) & (byte as u16);

    state.flags.carry = (result > 0xff) as u8;
    state.flags.zero = (((result as u8) & 0xff) == 0) as u8;
    state.flags.sign = (((result as u8) & 0x80) != 0) as u8;
    state.flags.parity = parity(result);

    state.regs.a = result as u8;
}

pub fn xra(state: &mut Intel8080, byte: u8) {
    let result = (state.regs.a as u16) ^ (byte as u16);

    state.flags.carry = (result > 0xff) as u8;
    state.flags.zero = (((result as u8) & 0xff) == 0) as u8;
    state.flags.sign = (((result as u8) & 0x80) != 0) as u8;
    state.flags.parity = parity(result);

    state.regs.a = result as u8;
}

pub fn ora(state: &mut Intel8080, byte: u8) {
    let result = (state.regs.a as u16) | (byte as u16);

    state.flags.carry = (result > 0xff) as u8;
    state.flags.zero = (((result as u8) & 0xff) == 0) as u8;
    state.flags.sign = (((result as u8) & 0x80) != 0) as u8;
    state.flags.parity = parity(result);

    state.regs.a = result as u8;
}

pub fn cmp(state: &mut Intel8080, byte: u8) {
    let result = (state.regs.a as u16) - (byte as u16);

    state.flags.carry = (result > 0xff) as u8;
    state.flags.zero = (((result as u8) & 0xff) == 0) as u8;
    state.flags.sign = (((result as u8) & 0x80) != 0) as u8;
    state.flags.parity = parity(result);

    state.regs.a = result as u8;
}

pub fn pop(state: &mut Intel8080, byte: char) {
    match byte {
        'B' => {
            state.regs.c = state.memory[state.sp];
            state.regs.b = state.memory[state.sp + 1];
        }
        'D' => {
            state.regs.e = state.memory[state.sp];
            state.regs.d = state.memory[state.sp + 1];
        }
        'H' => {
            state.regs.h = state.memory[state.sp];
            state.regs.l = state.memory[state.sp + 1];
        }
        'P' => {
            state.regs.a = state.memory[state.sp + 1];

            // get the content of the memory location specified by the stack pointer
            let result = state.memory[state.sp] as u16;

            state.flags.carry = (result > 0xff) as u8;
            state.flags.zero = (((result as u8) & 0xff) == 0) as u8;
            state.flags.sign = (((result as u8) & 0x80) != 0) as u8;
            state.flags.parity = parity(result);
        }
        _ => {}
    }

    state.sp += 2;
}

pub fn push(state: &mut Intel8080, byte: char) {
    match byte {
        'B' => {
            state.memory[state.sp - 1] = state.regs.b;
            state.memory[state.sp - 2] = state.regs.c;
        }
        'D' => {
            state.memory[state.sp - 1] = state.regs.d;
            state.memory[state.sp - 2] = state.regs.e;
        }
        'H' => {
            state.memory[state.sp - 1] = state.regs.h;
            state.memory[state.sp - 2] = state.regs.l;
        }
        'P' => {
            state.memory[state.sp - 1] = state.regs.a;

            let psw = state.flags.zero             |
                        state.flags.sign      << 1 |
                        state.flags.parity    << 2 |
                        state.flags.carry     << 3 |
                        state.flags.aux_carry << 4;

            state.memory[state.sp - 2] = psw;
        }
        _ => {}
    }

    state.sp -= 2;
}

pub fn rst(state: &mut Intel8080, code: u8) {
    let addr = state.pc + 1; // Address of the next instruction

    let msb = ((addr & 0xff00) >> 8) as u8;
    let lsb = (addr & 0x00ff) as u8;
    
    state.memory[state.sp - 1] = msb;
    state.memory[state.sp - 2] = lsb;

    state.pc = ((code as u16) << 3) as usize;

    state.sp -= 2;
}