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