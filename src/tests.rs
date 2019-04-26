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

    machine.emulate();

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

    machine.emulate();

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

    machine.emulate();

    assert_eq!(machine.regs.b, 0x9A);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.sign, 0);
    assert_eq!(machine.flags.parity, 1);

    // opcode 0x0c
    let mut machine = Intel8080::new();
    machine.regs.c = 0x99;
    machine.memory = vec![
        0x0c,
        0x76
    ];

    machine.emulate();

    assert_eq!(machine.regs.c, 0x9A);
    assert_eq!(machine.flags.zero, 0);
    assert_eq!(machine.flags.sign, 0);
    assert_eq!(machine.flags.parity, 1);

    // opcode 5
    let mut machine = Intel8080::new();
    machine.regs.b = 0x08;
    machine.memory = vec![
        5,
        0x76
    ];

    machine.emulate();

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

    machine.emulate();

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

    machine.emulate();

    assert_eq!(machine.regs.b, 45);

    // opcode 0x0e
    let mut machine = Intel8080::new();
    machine.memory = vec![
        0x0e, // opcode
        45, // data
        0x76
    ];

    machine.emulate();

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

    machine.emulate();

    assert_eq!(machine.regs.a, 0xe5);
    assert_eq!(machine.flags.carry, 1);

    // opcode 0x0f
    let mut machine = Intel8080::new();
    machine.regs.a = 0xf2;
    machine.memory = vec![
        0x0f, // opcode
        0x76
    ];

    machine.emulate();

    assert_eq!(machine.regs.a, 0x79);
    assert_eq!(machine.flags.carry, 0);

    let mut machine = Intel8080::new();
    machine.regs.a = 0xf3;
    machine.memory = vec![
        0x0f, // opcode
        0x76
    ];

    machine.emulate();

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

    machine.emulate();

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

    machine.emulate();

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

    machine.emulate();

    assert_eq!(machine.regs.a, 0x34);

    let mut machine = Intel8080::new();
    machine.regs.b = 0x00;
    machine.regs.c = 0x05;

    machine.memory = vec![
        0x0b,
        0x76
    ];

    machine.emulate();

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

    machine.emulate();

    assert_eq!(machine.regs.a, 0x6a);
    assert_eq!(machine.flags.carry, 1);

    let mut machine = Intel8080::new();
    machine.regs.a = 0xb5;
    machine.flags.carry = 1;
    machine.memory = vec![
        0x17, // opcode
        0x76
    ];

    machine.emulate();

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

    machine.emulate();

    assert_eq!(machine.regs.a, 0xb5);
    assert_eq!(machine.flags.carry, 0);
}