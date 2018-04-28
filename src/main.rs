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

struct World {
    gl: GlGraphics
}

struct Game {
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
    fn render(&self, world: &mut World, arg: &RenderArgs) {
        use graphics;

        let GREEN = [0.0, 1.0, 0.0, 1.0];
        let ORANGE = [1.0, 1.0, 0.0, 1.0];

        world.gl.draw(arg.viewport(), |_c, gl| {
            graphics::clear(GREEN, gl);
        });

        self.snake.render(world, arg);

        world.gl.draw(arg.viewport(), |c, gl| {
            let transform = c.transform;

            let square = graphics::rectangle::square(
                (self.apple.0 * 20) as f64,
                (self.apple.1 * 20) as f64, 20_f64
            );
            graphics::rectangle(ORANGE, square, transform, gl);
        });
    }

    fn update(&mut self, _world: &mut World, u: &UpdateArgs) -> Transition {
        if self.snake.update(u) {
            Transition::Pop
        } else {
            if *self.snake.get_head() == self.apple {
                self.snake.grow();
                while self.snake.is_body(&self.apple) {
                    self.generate_apple();
                }
            }
            Transition::None
        }
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
    dir: Direction,
    last: (i32, i32),
    dt: f64,
    update: f64
}

impl Snake {
    fn render(&self, world: &mut World, args: &RenderArgs) {
        use graphics;

        let RED = [1.0, 0.0, 0.0, 1.0];

        let squares: Vec<graphics::types::Rectangle> = self.body.iter().map(|&(x, y)| {
            graphics::rectangle::square(
                    (x * 20) as f64, 
                    (y * 20) as f64, 20_f64)
        }).collect();

        world.gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            squares.into_iter().for_each(|square| graphics::rectangle(RED, square, transform, gl));
        });
    }

    fn get_head(&self) -> &(i32, i32) {
        &self.body.front().expect("Snake has no body")
    }

    fn grow(&mut self) {
        self.body.push_back(self.last);
    }

    fn is_body(&self, pos: &(i32, i32)) -> bool {
        self.body.contains(pos)
    }

    fn update(&mut self, u: &UpdateArgs) -> bool {
        self.dt += u.dt;
        if self.dt < self.update {
            return false;
        }
        self.dt = 0_f64;
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

        if self.is_body(&new_head) {
            return true;
        }

        self.body.push_front(new_head);
        self.last = self.body.pop_back().unwrap();
        false
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
    fn render(&self, world: &mut World, args: &RenderArgs);
    fn update(&mut self, world: &mut World, args: &UpdateArgs) -> Transition;
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

    fn render(&self, world: &mut World, args: &RenderArgs) {
        if self.running {
            let state = self.states.last().unwrap();
            state.render(world, args);
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

    fn update(&mut self, world: &mut World, u: &UpdateArgs) {
        if self.running {
            let trans = match self.states.last_mut() {
                Some(ref mut state) => state.update(world, u),
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
        snake: Snake { 
            body: LinkedList::from_iter((vec![(1,0), (0,0)]).into_iter()),
            dir: Direction::Right,
            dt: 0_f64,
            last: (0, 0),
            update: 0.2
        },
        apple: (3, 3)
    };

    let mut world = World {
        gl: GlGraphics::new(opengl)
    };


    let mut state_machine = StateMachine::new(Box::new(game));
    state_machine.start();

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            state_machine.render(&mut world, &r);
        }

        if let Some(u) = e.update_args() {
            state_machine.update(&mut world, &u);
        }

        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                state_machine.handle_event(&k.button);
            }
        }
        if !state_machine.is_running() {
            break
        }
    }
}
