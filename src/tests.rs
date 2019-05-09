use crate::intel8080::Intel8080;

#[test]
fn emulate_opcodes_1_and_2 () {
    let mut machine = Intel8080::new();
    machine.regs.a = 255;
    machine.memory = vec![
        1, // LXI B
        10, 0, 
        2, // STAX B
        0, 0,
        0x76, // HLT
        0, 0, 0, 0, 0
    ];

    machine.run();

    assert_eq!(machine.regs.b, 0);
    assert_eq!(machine.regs.c, 10);
    assert_eq!(machine.memory[10], 255);
}

#[test]
fn emulate_opcodes_3() {
    let mut machine = Intel8080::new();
    machine.memory = vec![
        1, // LXI B
        0xff, 0x38, 
        3, // INX B
        0,
        0x76, // HLT
        0, 0, 0, 0, 0
    ];

    machine.run();

    assert_eq!(machine.regs.b, 0x39);
    assert_eq!(machine.regs.c, 0x00);
}

#[test]
fn emulate_opcodes_4_0x0c_5_0x0d_6_0x0e() {
    // opcode 4
    let mut machine = Intel8080::new();
    machine.regs.b = 0x99;
    machine.memory = vec![
        4,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.b, 0x9A);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.sign, 1);
    assert_eq!(machine.flags.parity, 1);

    // opcode 0x0c
    let mut machine = Intel8080::new();
    machine.regs.c = 0x99;
    machine.memory = vec![
        0x0c,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.c, 0x9A);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.sign, 1);
    assert_eq!(machine.flags.parity, 1);

    // opcode 5
    let mut machine = Intel8080::new();
    machine.regs.b = 0x08;
    machine.memory = vec![
        5,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.b, 0x07);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.sign, 0);
    assert_eq!(machine.flags.parity, 0);

    // opcode 0x0d
    let mut machine = Intel8080::new();
    machine.regs.c = 0x08;
    machine.memory = vec![
        0x0d,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.c, 0x07);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.sign, 0);
    assert_eq!(machine.flags.parity, 0);

    // opcode 6
    let mut machine = Intel8080::new();
    machine.memory = vec![
        6, // opcode
        45, // data
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.b, 45);

    // opcode 0x0e
    let mut machine = Intel8080::new();
    machine.memory = vec![
        0x0e, // opcode
        45, // data
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.c, 45);
}

#[test]
fn emulate_opcode_7_0x0f() {
    let mut machine = Intel8080::new();
    machine.regs.a = 0xf2;
    machine.memory = vec![
        7, // opcode
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0xe5);
    assert_eq!(machine.flags.carry, 1);

    // opcode 0x0f
    let mut machine = Intel8080::new();
    machine.regs.a = 0xf2;
    machine.memory = vec![
        0x0f, // opcode
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0x79);
    assert_eq!(machine.flags.carry, 0);

    let mut machine = Intel8080::new();
    machine.regs.a = 0xf3;
    machine.memory = vec![
        0x0f, // opcode
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0xf9);
    assert_eq!(machine.flags.carry, 1);
}

#[test]
fn emulate_9() {
    let mut machine = Intel8080::new();
    machine.regs.b = 0x33;
    machine.regs.c = 0x9f;
    machine.regs.h = 0xa1;
    machine.regs.l = 0x7b;

    machine.memory = vec![
        9,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.h, 0xd5);
    assert_eq!(machine.regs.l, 0x1a);
    assert_eq!(machine.flags.carry, 0);

    let mut machine = Intel8080::new();
    machine.regs.b = 0xff;
    machine.regs.c = 0xff;
    machine.regs.h = 0x00;
    machine.regs.l = 0x01;

    machine.memory = vec![
        9,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.h, 0x00);
    assert_eq!(machine.regs.l, 0x00);
    assert_eq!(machine.flags.carry, 1);
}


#[test]
fn emulate_opcode_0a_and_0b() {
    let mut machine = Intel8080::new();
    machine.regs.b = 0x00;
    machine.regs.c = 0x05;

    machine.memory = vec![
        0x0a,
        0, 0, 0,
        0x76,
        0x34
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0x34);

    let mut machine = Intel8080::new();
    machine.regs.b = 0x00;
    machine.regs.c = 0x05;

    machine.memory = vec![
        0x0b,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.b, 0);
    assert_eq!(machine.regs.c, 4);
}

#[test]
fn emulate_opcode_0x17() {
    let mut machine = Intel8080::new();
    machine.regs.a = 0xb5;
    machine.memory = vec![
        0x17, // opcode
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0x6a);
    assert_eq!(machine.flags.carry, 1);

    let mut machine = Intel8080::new();
    machine.regs.a = 0xb5;
    machine.flags.carry = 1;
    machine.memory = vec![
        0x17, // opcode
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0xea);
    assert_eq!(machine.flags.carry, 1);
}

#[test]
fn emulate_0x1f() {
    let mut machine = Intel8080::new();
    machine.regs.a = 0x6a;
    machine.flags.carry = 1;
    machine.memory = vec![
        0x1f, // opcode
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0xb5);
    assert_eq!(machine.flags.carry, 0);
}

#[test]
fn emulate_shld() {
    // opcode SHLD
    let mut machine = Intel8080::new();
    machine.regs.h = 0xae;
    machine.regs.l = 0x29;

    machine.memory = vec![
        0x22,
        4,
        0,
        0x76,
        0,
        0
    ];

    machine.run();

    assert_eq!(machine.memory[4], 0x29);
    assert_eq!(machine.memory[5], 0xae);

    // opcode LHLD
    let mut machine = Intel8080::new();
    machine.regs.h = 0xae;
    machine.regs.l = 0x29;

    machine.memory = vec![
        0x22,
        4,
        0,
        0x76,
        0x29,
        0xae
    ];

    machine.run();

    assert_eq!(machine.regs.l, 0x29);
    assert_eq!(machine.regs.h, 0xae);
}

#[test]
fn emulate_cma() {
    let mut machine = Intel8080::new();
    machine.regs.a = 0x51;

    machine.memory = vec![
        0x2f,
        0x76
    ];

    machine.run();
    
    assert_eq!(machine.regs.a, 0xae);
}

#[test]
fn emulate_daa() {
    let mut machine = Intel8080::new();
    machine.regs.a = 0x9b;

    machine.memory = vec![
        0x27,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 1);
    assert_eq!(machine.flags.carry, 1);
    assert_eq!(machine.flags.aux_carry, 1);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.sign, 0);
    assert_eq!(machine.flags.parity, 0);
}

#[test]
fn emulate_sta() {
    let mut machine = Intel8080::new();
    machine.regs.a = 0x9b;

    machine.memory = vec![
        0x32,
        4, 0,
        0x76,
        0
    ];

    machine.run();

    assert_eq!(machine.memory[4], machine.regs.a);
}

#[test]
fn emulate_add() {
    let mut machine = Intel8080::new();
    machine.regs.a = 0x6c;
    machine.regs.b = 0x2e;

    machine.memory = vec![
        0x80,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0x9a);
    assert_eq!(machine.flags.sign, 1);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.parity, 1);
    assert_eq!(machine.flags.carry, 0);
}

#[test]
fn emulate_adc() {
    let mut machine = Intel8080::new();
    machine.flags.carry = 0;
    machine.regs.a = 0x42;
    machine.regs.b = 0x3d;

    machine.memory = vec![
        0x88,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0x7f);
    assert_eq!(machine.flags.sign, 0);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.parity, 0);
    assert_eq!(machine.flags.carry, 0);

    let mut machine = Intel8080::new();
    machine.regs.a = 0x42;
    machine.regs.b = 0x3d;
    machine.flags.carry = 1;

    machine.memory = vec![
        0x88,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0x80);
    assert_eq!(machine.flags.sign, 1);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.parity, 0);
    assert_eq!(machine.flags.carry, 0);
}

#[test]
fn emulate_sub() {
    let mut machine = Intel8080::new();
    machine.regs.a = 0x3e;
    machine.regs.b = 0x3e;

    machine.memory = vec![
        0x97,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0x00);
    assert_eq!(machine.flags.sign, 0);
    assert_eq!(machine.flags.zero, 1);
    assert_eq!(machine.flags.parity, 1);
    assert_eq!(machine.flags.carry, 0);
}

#[test]
fn emulate_sbb() {
    let mut machine = Intel8080::new();
    machine.regs.a = 0x04;
    machine.regs.b = 0x02;
    machine.flags.carry = 1;

    machine.memory = vec![
        0x98,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0x01);
    assert_eq!(machine.flags.sign, 0);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.parity, 0);
    assert_eq!(machine.flags.carry, 0);
}

#[test]
fn emulate_ana() {
    let mut machine = Intel8080::new();
    machine.regs.a = 0xfc;
    machine.regs.b = 0x0f;

     machine.memory = vec![
        0xa0,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0x0c);
    assert_eq!(machine.flags.sign, 0);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.parity, 1);
    assert_eq!(machine.flags.carry, 0);
}

#[test]
fn emulate_xra() {
    let mut machine = Intel8080::new();
    machine.regs.a = 0x5c;
    machine.regs.b = 0x78;

     machine.memory = vec![
        0xa8,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0x24);
    assert_eq!(machine.flags.sign, 0);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.parity, 1);
    assert_eq!(machine.flags.carry, 0);
}

#[test]
fn emulate_ora() {
    let mut machine = Intel8080::new();
    machine.regs.a = 0x33;
    machine.regs.b = 0x0f;

     machine.memory = vec![
        0xb0,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0x3f);
    assert_eq!(machine.flags.sign, 0);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.parity, 1);
    assert_eq!(machine.flags.carry, 0);
}

#[test]
fn emulate_cmp() {
    let mut machine = Intel8080::new();
    machine.regs.a = 0x0a;
    machine.regs.b = 0x05;

     machine.memory = vec![
        0xb8,
        0x76
    ];

    machine.run();

    assert_eq!(machine.regs.a, 0x05);
    assert_eq!(machine.flags.sign, 0);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.parity, 1);
    assert_eq!(machine.flags.carry, 0);
}

#[test]
fn emulate_pop() {
    let mut machine = Intel8080::new();
    machine.sp = 0x05;

    machine.memory = vec![
        0xc1,
        0x76,
        0, 0, 0, 0xae, 0x2b 
    ];

    machine.run();

    assert_eq!(machine.regs.c, 0xae);
    assert_eq!(machine.regs.b, 0x2b);
    assert_eq!(machine.sp, 0x07);
}

#[test]
fn emulate_rnz() {
    let mut machine = Intel8080::new();
    machine.flags.zero = 0;
    machine.sp = 0x05;

    machine.memory = vec![
        0xc0,
        0, 0, 0, 
        0x76, 0x04, 0x00, 0
    ];

    machine.run();

    assert_eq!(machine.pc, 0x04);
    assert_eq!(machine.sp, 0x07);
}

#[test]
fn emulate_jnz() {
    let mut machine = Intel8080::new();
    machine.flags.zero = 0;
    machine.sp = 0x05;

    machine.memory = vec![
        0xc2,
        0x04, 0x00, 0, 
        0x76, 0x04, 0x00, 0
    ];

    machine.run();

    assert_eq!(machine.pc, 0x04);
}

#[test]
fn emulate_cnz() {
    let mut machine = Intel8080::new();
    machine.flags.zero = 0;
    machine.sp = 0x0B;

    machine.memory = vec![
        0, 0, 0, 0, 0,
        0xc4,
        0x08, 0x00, 0x76, 
        0, 0, 0, 0
    ];

    machine.run();

    assert_eq!(machine.pc, 0x08);
    assert_eq!(machine.memory[10], 0x00);
    assert_eq!(machine.memory[9], 0x08);
    assert_eq!(machine.sp, 0x09);
}

#[test]
fn emulate_push() {
    let mut machine = Intel8080::new();
    machine.regs.b = 0x7b;
    machine.regs.c = 0x8c;
    machine.sp = 0x05;

    machine.memory = vec![
        0xc5,
        0x76,
        0, 0, 0, 0
    ];

    machine.run();

    assert_eq!(machine.sp, 0x03);
    assert_eq!(machine.memory[4], 0x7b);
    assert_eq!(machine.memory[3], 0x8c);
}

#[test]
fn emulate_rst() {
    let mut machine = Intel8080::new();
    machine.pc = 0x03;
    machine.sp = 0x06;

    machine.memory = vec![
        0, 0, 0x76,
        0xc7,
        0, 0, 0,
    ];

    machine.run();

    assert_eq!(machine.memory[5], 0x00);
    assert_eq!(machine.memory[4], 0x04);
    assert_eq!(machine.pc, 0x02);
    assert_eq!(machine.sp, 0x04);
}