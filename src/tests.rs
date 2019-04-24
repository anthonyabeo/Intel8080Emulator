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
fn emulate_opcode_4() {
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
}