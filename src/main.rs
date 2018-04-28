extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};

use std::collections::LinkedList;
use std::iter::FromIterator;

#[derive(Clone, PartialEq)]
enum Direction {
    Right,
    Left,
    Up,
    Down
}

struct Game {
    gl: GlGraphics,
    snake: Snake,
    apple: (i32, i32)
}

impl Game {
    fn generate_apple(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        self.apple = (rng.gen_range(0, 600 / 20), rng.gen_range(0, 400 / 20));
    }
}

impl State for Game {
    fn render(&mut self, arg: &RenderArgs) {
        use graphics;

        let GREEN = [0.0, 1.0, 0.0, 1.0];
        let ORANGE = [1.0, 1.0, 0.0, 1.0];

        self.gl.draw(arg.viewport(), |_c, gl| {
            graphics::clear(GREEN, gl);
        });

        self.snake.render(&mut self.gl, arg);

        let apple = self.apple.clone();

        self.gl.draw(arg.viewport(), |c, gl| {
            let transform = c.transform;

            let square = graphics::rectangle::square(
                (apple.0 * 20) as f64,
                (apple.1 * 20) as f64, 20_f64
            );
            graphics::rectangle(ORANGE, square, transform, gl);
        });
    }

    fn update(&mut self, u: &UpdateArgs) -> Transition {
        if self.snake.update(&self.apple) {
            self.generate_apple();
        }
        Transition::None
    }

    fn handle_event(&mut self, btn: &Button) -> Transition {
        let last_dir = self.snake.dir.clone();

        self.snake.dir = match *btn {
            Button::Keyboard(Key::Up) if last_dir != Direction::Down => Direction::Up,
            Button::Keyboard(Key::Down) if last_dir != Direction::Up => Direction::Down,
            Button::Keyboard(Key::Left) if last_dir != Direction::Right => Direction::Left,
            Button::Keyboard(Key::Right) if last_dir != Direction::Left => Direction::Right,
            _ => last_dir,
        };
        Transition::None
    }
}

struct Snake {
    body: LinkedList<(i32, i32)>,
    dir: Direction
}

impl Snake {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics;

        let RED = [1.0, 0.0, 0.0, 1.0];

        let squares: Vec<graphics::types::Rectangle> = self.body.iter().map(|&(x, y)| {
            graphics::rectangle::square(
                    (x * 20) as f64, 
                    (y * 20) as f64, 20_f64)
        }).collect();

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            squares.into_iter().for_each(|square| graphics::rectangle(RED, square, transform, gl));
        });
    }

    fn update(&mut self, apple: &(i32, i32)) -> bool {
        let mut new_head = (*self.body.front().expect("Snake has no body")).clone();
        match self.dir {
            Direction::Left => new_head.0 -= 1,
            Direction::Right => new_head.0 += 1,
            Direction::Up => new_head.1 -= 1,
            Direction::Down => new_head.1 += 1,
        }

        if new_head.0 < 0 {
            new_head.0 += 600 / 20;
        }
        if new_head.1 < 0 {
            new_head.1 += 400 / 20;
        }

        new_head.0 %= 600 / 20;
        new_head.1 %= 400 / 20;

        if self.body.contains(&new_head) {
            panic!("You loose");
        }

        self.body.push_front(new_head);
        if new_head != *apple {
            self.body.pop_back().unwrap();
            false
        } else {
            true
        }
    }
}

enum Transition {
    None,
    Pop,
    Push(Box<State>),
    Switch(Box<State>),
    Quit
}

trait State {
    fn render(&mut self, args: &RenderArgs);
    fn update(&mut self, args: &UpdateArgs) -> Transition;
    fn handle_event(&mut self, btn: &Button) -> Transition;
    fn on_start(&mut self) {}
    fn on_stop(&mut self) {}
    fn on_pause(&mut self) {}
    fn on_resume(&mut self) {}
}

struct StateMachine {
    running: bool,
    states: Vec<Box<State>>
}

impl StateMachine {
    fn new(state: Box<State>) -> StateMachine {
        StateMachine {
            running: false,
            states: vec![state]
        }
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn render(&mut self, args: &RenderArgs) {
        if self.running {
            let state = self.states.last_mut().unwrap();
            state.render(args);
        }
    }

    fn start(&mut self) {
        if !self.running {
            let state = self.states.last_mut().unwrap();
            state.on_start();
            self.running = true;
        }
    }

    fn handle_event(&mut self, btn: &Button) {
        if self.running {
            let trans = match self.states.last_mut() {
                Some(ref mut state) => state.handle_event(btn),
                None => Transition::None
            };
            self.transition(trans);
        }
    }

    fn update(&mut self, u: &UpdateArgs) {
        if self.running {
            let trans = match self.states.last_mut() {
                Some(ref mut state) => state.update(u),
                None => Transition::None
            };
            self.transition(trans);
        }
    }

    fn transition(&mut self, trans: Transition) {
        if self.running {
            match trans {
                Transition::None => {},
                Transition::Pop => self.pop(),
                Transition::Push(state) => self.push(state),
                Transition::Switch(state) => self.switch(state),
                Transition::Quit => self.stop()
            }
        }
    }

    fn switch(&mut self, state: Box<State>) {
        if self.running {
            if let Some(mut state) = self.states.pop() {
                state.on_stop();
            }
            self.states.push(state);
            let state = self.states.last_mut().unwrap();
            state.on_start();
        }
    }

    fn push(&mut self, state: Box<State>) {
        if self.running {
            if let Some(ref mut state) = self.states.last_mut() {
                state.on_pause();
            }
            self.states.push(state);
            let state = self.states.last_mut().unwrap();
            state.on_start();
        }
    }

    fn pop(&mut self) {
        if self.running {
            if let Some(mut state) = self.states.pop() {
                state.on_stop();
            }
            if let Some(ref mut state) = self.states.last_mut() {
                state.on_resume();
            } else {
                self.running = false;
            }
        }
    }

    fn stop(&mut self) {
        if self.running {
            while let Some(mut state) = self.states.pop() {
                state.on_stop();
            }
            self.running = false;
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: GlutinWindow = WindowSettings::new("Snake Game", [600, 400])
                                    .opengl(opengl).exit_on_esc(true).build().unwrap();
    
    let game = Game {
        gl: GlGraphics::new(opengl),
        snake: Snake { 
            body: LinkedList::from_iter((vec![(0,0), (0,1)]).into_iter()),
            dir: Direction::Right },
        apple: (3, 3)
    };


    let mut state_machine = StateMachine::new(Box::new(game));
    state_machine.start();

    let mut events = Events::new(EventSettings::new()).ups(8);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            state_machine.render(&r);
        }

        if let Some(u) = e.update_args() {
            state_machine.update(&u);
        }

        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                state_machine.handle_event(&k.button);
            }
        }
    }
}
