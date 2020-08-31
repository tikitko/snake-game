use super::snake_world;
use snake_world::{SnakeWorld, SnakeWorldCreateError};

use std::hash::Hash;
use crate::snake_world::{SnakeWorldConfig, SnakeWorldWorldView};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

type Config = SnakeGameConfig;
type CreateError = SnakeGameCreateError;
type ActionType = SnakeGameActionType;
type TickType = SnakeGameTickType;
type GameController = dyn SnakeGameGameController;

pub struct SnakeGameConfig {
    pub game_controller: Rc<RefCell<GameController>>,
}

impl SnakeGameConfig {
    fn game_controller(&self) -> RefMut<GameController> {
        self.game_controller.as_ref().borrow_mut()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SnakeGameCreateError {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SnakeGameActionType {
    Start,
    Exit,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SnakeGameTickType {
    Initial,
    Common,
    Break,
}

pub trait SnakeGameGameController {
    fn game_action(&mut self) -> ActionType;
    fn game_start(&mut self) -> SnakeWorldConfig;
    fn game_will_tick(&mut self, previous_world_view: &Option<SnakeWorldWorldView>) -> TickType;
    fn game_did_tick(&mut self, world_view: &SnakeWorldWorldView);
    fn game_end(&mut self, state: Result<(), SnakeWorldCreateError>);
}

pub struct SnakeGame {
    config: Config,
}

impl SnakeGame {
    pub fn try_create(config: Config) -> Result<Self, CreateError> {
        Ok(SnakeGame {
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
                SnakeGameActionType::Start => {
                    let world_config = self.config.game_controller().game_start();
                    match SnakeWorld::try_create(world_config) {
                        Ok(mut world) => {
                            self.start_tick_loop(&mut world);
                            self.config.game_controller().game_end(Ok(()));
                        }
                        Err(err) => self.config.game_controller().game_end(Err(err)),
                    }
                }
                SnakeGameActionType::Exit => break 'game_loop,
            }
        }
    }
    fn start_tick_loop(&mut self, world: &mut SnakeWorld) {
        let mut last_world_view: Option<SnakeWorldWorldView> = None;
        'tick_loop: loop {
            let tick_type = self.config.game_controller().game_will_tick(&last_world_view);
            let world_view = match tick_type {
                SnakeGameTickType::Initial => world.tick(true),
                SnakeGameTickType::Common => world.tick(false),
                SnakeGameTickType::Break => break 'tick_loop,
            };
            self.config.game_controller().game_did_tick(&world_view);
            last_world_view = Some(world_view);
        }
    }
}