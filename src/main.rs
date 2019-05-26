use std::env;
use std::process;

use emulator_intel8080::intel8080::Intel8080;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} - Executable file not provided.", &args[0]);
        process::exit(1);
    }

    let mut machine = Intel8080::new();
    
    machine.load_program(&args[1]); 

    machine.run();
}
