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

    let mut window = RenderWindow::new((1024, 768), "SPACE INVADERS", Style::DEFAULT, &Default::default());
    window.set_vertical_sync_enabled(true);

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => {window.close(); return; },
                Event::KeyPressed { code, .. } => {
                    match code {
                        Key::Space => println!("space key was pressed"),
                        Key::Left => println!("Left arrow key was pressed"),
                        Key::Right => println!("Right arrow key was pressed"),
                        _ => {}
                    }
                },
                Event::KeyReleased { code, .. } => {
                    match code {
                        Key::Space => println!("space key was released"),
                        Key::Left => println!("Left arrow key was released"),
                        Key::Right => println!("Right arrow key was released"),
                        _ => {}
                    }
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
