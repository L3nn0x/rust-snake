use super::World;
use super::piston_window::*;

pub enum Transition {
    None,
    Pop,
    Push(Box<State>),
    Switch(Box<State>),
    Quit
}

pub trait State {
    fn render(&self, world: &mut World, c: &Context, g: &mut G2d);
    fn update(&mut self, args: &UpdateArgs) -> Transition;
    fn handle_event(&mut self, btn: &Button) -> Transition;
    fn on_start(&mut self) {}
    fn on_stop(&mut self) {}
    fn on_pause(&mut self) {}
    fn on_resume(&mut self) {}
}

pub struct StateMachine {
    running: bool,
    states: Vec<Box<State>>
}

impl StateMachine {
    pub fn new(state: Box<State>) -> StateMachine {
        StateMachine {
            running: false,
            states: vec![state]
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn render(&self, world: &mut World, c: &Context, g: &mut G2d) {
        if self.running {
            let state = self.states.last().unwrap();
            state.render(world, c, g);
        }
    }

    pub fn start(&mut self) {
        if !self.running {
            let state = self.states.last_mut().unwrap();
            state.on_start();
            self.running = true;
        }
    }

    pub fn handle_event(&mut self, btn: &Button) {
        if self.running {
            let trans = match self.states.last_mut() {
                Some(ref mut state) => state.handle_event(btn),
                None => Transition::None
            };
            self.transition(trans);
        }
    }

    pub fn update(&mut self, u: &UpdateArgs) {
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