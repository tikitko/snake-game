use super::world;

use std::cell::{RefCell, RefMut};
use std::hash::Hash;
use std::rc::Rc;

pub struct Config {
    pub game_controller: Rc<RefCell<dyn GameController>>,
}

impl Config {
    fn game_controller(&self) -> Option<RefMut<dyn GameController>> {
        match self.game_controller.try_borrow_mut() {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum CreateError {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ActionType {
    Start,
    Exit,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TickType {
    Initial,
    Common,
    Break,
}

pub trait GameController {
    fn game_action(&mut self) -> ActionType;
    fn game_start(&mut self) -> world::Config;
    fn game_will_tick(&mut self, previous_world_view: &Option<world::WorldView>) -> TickType;
    fn game_did_tick(&mut self, world_view: &world::WorldView);
    fn game_end(&mut self, state: Result<(), world::CreateError>);
}

pub struct Game {
    config: Config,
}

impl Game {
    pub fn new(config: Config) -> Result<Self, CreateError> {
        Ok(Self { config })
    }
    pub fn start(&mut self) {
        self.start_game_loop();
    }
    fn start_game_loop(&mut self) {
        loop {
            let game_action = match self.config.game_controller() {
                Some(mut controller) => controller.game_action(),
                None => break,
            };
            match game_action {
                ActionType::Start => {
                    let world_config = match self.config.game_controller() {
                        Some(mut controller) => controller.game_start(),
                        None => break,
                    };
                    let start_result = match world::World::new(world_config) {
                        Ok(mut world) => {
                            self.start_tick_loop(&mut world);
                            Ok(())
                        }
                        Err(err) => Err(err),
                    };
                    match self.config.game_controller() {
                        Some(mut controller) => controller.game_end(start_result),
                        None => break,
                    }
                }
                ActionType::Exit => break,
            }
        }
    }
    fn start_tick_loop(&mut self, world: &mut world::World) {
        let mut last_world_view: Option<world::WorldView> = None;
        loop {
            let tick_type = match self.config.game_controller() {
                Some(mut controller) => controller.game_will_tick(&last_world_view),
                None => break,
            };
            let world_view = match tick_type {
                TickType::Initial => world.tick(true),
                TickType::Common => world.tick(false),
                TickType::Break => break,
            };
            match self.config.game_controller() {
                Some(mut controller) => controller.game_did_tick(&world_view),
                None => break,
            };
            last_world_view = Some(world_view);
        }
    }
}
