extern crate sdl2; 

use std::env;
use std::process;

use emulator_intel8080::cpu::intel8080::Intel8080;
use emulator_intel8080::space_invaders::SpaceInvadersMachine;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} - Executable file not provided.", &args[0]);
        process::exit(1);
    }

    let mut state = Intel8080::new();
    state.load_program(&args[1]);

    let _space_invaders = SpaceInvadersMachine::new();
}
