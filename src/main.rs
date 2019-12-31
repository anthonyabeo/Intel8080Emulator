extern crate sfml;

use std::env;
use std::process;

use emulator_intel8080::cpu::intel8080::Intel8080;

use sfml::graphics::*;
use sfml::window::*;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} - Executable file not provided.", &args[0]);
        process::exit(1);
    }

    let mut window = RenderWindow::new((800, 600), "SPACE INVADERS", Style::CLOSE, &Default::default());
    window.set_vertical_sync_enabled(true);

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => {
                    window.close();
                    return;
                }
                _ => {
                    window.clear(&Color::BLACK);
                    window.display();
                }
            }
        }
    }

    let mut machine = Intel8080::new();
    machine.load_program(&args[1]); 
    machine.run();
}
