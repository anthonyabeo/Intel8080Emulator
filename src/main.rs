use std::env;
use std::process;

use emulator_intel8080::intel8080::Intel8080;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} - Game ROM file not provided", &args[0]);
        process::exit(1);
    }

    let machine = Intel8080::new();
    machine.emulate();
}
