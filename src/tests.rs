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
fn emulate_opcodes_4_5_6() {
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

    // opcode 6
    let mut machine = Intel8080::new();
    machine.memory = vec![
        6, // opcode
        45, // data
        0x76
    ];

    machine.emulate();

    assert_eq!(machine.regs.b, 45);
}

#[test]
fn emulate_opcode_7() {
    let mut machine = Intel8080::new();
    machine.regs.a = 0xf2;
    machine.memory = vec![
        7, // opcode
        0x76
    ];

    machine.emulate();

    assert_eq!(machine.regs.a, 0xe5);
    assert_eq!(machine.flags.carry, 1);
}