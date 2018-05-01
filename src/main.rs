extern crate piston_window;
extern crate find_folder;

use piston_window::*;

mod state_machine;
mod game;
mod menu;

use state_machine::*;
use menu::*;

pub struct World {
    font: Glyphs,
}

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Snake Game", [600, 400])
                                    .build().unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();

    let ref font = assets.join("arial.ttf");
    let factory = window.factory.clone();

    let font = Glyphs::new(font, factory, TextureSettings::new()).unwrap();

    let mut world = World { font: font};

    let mut state_machine = StateMachine::new(Box::new(Menu::new()));
    state_machine.start();

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, mut g| {
            if let Err(e) = state_machine.render(&mut world, &c, &mut g) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        });

        if let Some(u) = e.update_args() {
            state_machine.update(&u);
        }

        if let Some(k) = e.press_args() {
            state_machine.handle_event(&k);
        }
        if !state_machine.is_running() {
            break
        }
    }
}