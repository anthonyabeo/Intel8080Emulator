use emulator_intel8080::intel8080::Intel8080;


fn main() {
    // let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     println!("Usage: {} - Game ROM files not provided", &args[0]);
    //     process::exit(1);
    // }

    let mut machine = Intel8080::new();
    
    // load invaders.h into memory range 0x0000 - 0x07ff
    // load invaders.g into memory range 0x0800 - 0x0fff
    // load invaders.f into memory range 0x1000 - 0x17ff
    // load invaders.e into memory range 0x1800 - 0x1fff
    machine.load_game_rom("/Users/anabila/Downloads/invaders/invaders"); 

    // println!("{:?}", &machine.memory[0x0000..=0x17ff]);
    machine.emulate();
}
