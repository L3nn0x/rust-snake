use super::game::*;
use super::World;

use super::piston_window::*;

use super::state_machine::*;

enum Selection {
    NewGame,
    Quit
}

pub struct Menu {
    selection: Selection
}

impl Menu {
    pub fn new() -> Menu {
        Menu {
            selection: Selection::NewGame
        }
    }
}

impl State for Menu {
    fn render(&self, world: &mut World, c: &Context, g: &mut G2d) {
        let GREEN = [0.0, 1.0, 0.0, 1.0];
        let BLACK = [0.0, 0.0, 0.0, 1.0];

        clear(GREEN, g);
        let transform = c.transform.trans(10.0, 12.0);
        let size = Text::new(12);
        size.draw("Snake game", &mut world.font, &c.draw_state, transform, g);
        let transform = c.transform.trans(14.0, 50.0);
        size.draw("New game", &mut world.font, &c.draw_state, transform, g);
        let transform = c.transform.trans(14.0, 70.0);
        size.draw("Quit", &mut world.font, &c.draw_state, transform, g);
        let transform = match self.selection {
            Selection::NewGame => c.transform.trans(5.0, 50.0),
            Selection::Quit => c.transform.trans(5.0, 70.0)
        };
        size.draw(">", &mut world.font, &c.draw_state, transform, g);
    }

    fn update(&mut self, args: &UpdateArgs) -> Transition {
        Transition::None
    }

    fn handle_event(&mut self, btn: &Button) -> Transition {
        match *btn {
            Button::Keyboard(Key::Return) => match self.selection {
                Selection::NewGame => Transition::Push(Box::new(Game::new())),
                Selection::Quit => Transition::Quit
            },
            Button::Keyboard(Key::Up) => {
                self.selection = match self.selection {
                    Selection::NewGame => Selection::Quit,
                    Selection::Quit => Selection::NewGame
                };
                Transition::None
            },
            Button::Keyboard(Key::Down) => {
                self.selection = match self.selection {
                    Selection::NewGame => Selection::Quit,
                    Selection::Quit => Selection::NewGame
                };
                Transition::None
            },
            _ => Transition::None
        }
    }
}