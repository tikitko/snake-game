use super::world;

use std::hash::Hash;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

pub struct Config {
    pub game_controller: Rc<RefCell<dyn GameController>>,
}

impl Config {
    fn game_controller(&self) -> RefMut<dyn GameController> {
        self.game_controller.as_ref().borrow_mut()
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
    pub fn try_create(config: Config) -> Result<Self, CreateError> {
        Ok(Game {
            config
        })
    }
    pub fn start(&mut self) {
        self.start_game_loop()
    }
    fn start_game_loop(&mut self) {
        'game_loop: loop {
            let game_action = self.config.game_controller().game_action();
            match game_action {
                ActionType::Start => {
                    let world_config = self.config.game_controller().game_start();
                    match world::World::try_create(world_config) {
                        Ok(mut world) => {
                            self.start_tick_loop(&mut world);
                            self.config.game_controller().game_end(Ok(()));
                        }
                        Err(err) => self.config.game_controller().game_end(Err(err)),
                    }
                }
                ActionType::Exit => break 'game_loop,
            }
        }
    }
    fn start_tick_loop(&mut self, world: &mut world::World) {
        let mut last_world_view: Option<world::WorldView> = None;
        'tick_loop: loop {
            let tick_type = self.config.game_controller().game_will_tick(&last_world_view);
            let world_view = match tick_type {
                TickType::Initial => world.tick(true),
                TickType::Common => world.tick(false),
                TickType::Break => break 'tick_loop,
            };
            self.config.game_controller().game_did_tick(&world_view);
            last_world_view = Some(world_view);
        }
    }
}