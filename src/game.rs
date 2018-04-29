extern crate rand;

use std::collections::LinkedList;
use std::iter::FromIterator;

use super::World;
use super::piston_window::*;
use super::state_machine::*;

#[derive(Clone, PartialEq)]
enum Direction {
    Right,
    Left,
    Up,
    Down
}

pub struct Lost(i32);

impl State for Lost {
    fn render(&self, world: &mut World, c: &Context, g: &mut G2d) {
        let GREEN = [0.0, 1.0, 0.0, 1.0];
        clear(GREEN, g);
        let transform = c.transform.trans(10.0, 12.0);
        let size = Text::new(12);
        size.draw("You lost!", &mut world.font, &c.draw_state, transform, g);

        let transform = c.transform.trans(50.0, 50.0);
        size.draw(&format!("Your score: {}", self.0), &mut world.font, &c.draw_state, transform, g);
    }
    fn update(&mut self, args: &UpdateArgs) -> Transition {
        Transition::None
    }
    fn handle_event(&mut self, btn: &Button) -> Transition {
        Transition::Pop
    }
}

pub struct Game {
    snake: Snake,
    apple: (i32, i32)
}

impl Game {
    pub fn new() -> Game {
        Game {
            snake: Snake { 
                body: LinkedList::from_iter((vec![(1,0), (0,0)]).into_iter()),
                dir: Direction::Right,
                dt: 0_f64,
                last: (0, 0),
                update: 0.2
            },
            apple: (3, 3)
        }
    }

    fn generate_apple(&mut self) {
        use self::rand::Rng;
        let mut rng = rand::thread_rng();
        self.apple = (rng.gen_range(0, 600 / 20), rng.gen_range(0, 400 / 20));
    }
}

impl State for Game {
    fn render(&self, world: &mut World, c: &Context, g: &mut G2d) {
        let GREEN = [0.0, 1.0, 0.0, 1.0];
        let ORANGE = [1.0, 1.0, 0.0, 1.0];

        clear(GREEN, g);
        
        self.snake.render(world, c, g);
        let transform = c.transform;

        let square = rectangle::square(
            (self.apple.0 * 20) as f64,
            (self.apple.1 * 20) as f64, 20_f64
        );
        rectangle(ORANGE, square, transform, g);
    }

    fn update(&mut self, u: &UpdateArgs) -> Transition {
        if self.snake.update(u) {
            Transition::Switch(Box::new(Lost(self.snake.size())))
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
    fn render(&self, world: &mut World, c: &Context, g: &mut G2d) {
        let RED = [1.0, 0.0, 0.0, 1.0];

        let squares: Vec<types::Rectangle> = self.body.iter().map(|&(x, y)| {
            rectangle::square(
                    (x * 20) as f64, 
                    (y * 20) as f64, 20_f64)
        }).collect();

        let transform = c.transform;

        squares.into_iter().for_each(|square| rectangle(RED, square, transform, g));
    }

    fn size(&self) -> i32 {
        self.body.len() as i32
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