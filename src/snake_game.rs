use super::snake;
use super::point;
use super::world;
use super::direction;
use super::snake_world;
use snake::Snake;
use point::Point;
use world::World;
use direction::Direction;
use snake_world::{SnakeWorld, SnakeWorldCreateError};

use rand::Rng;
use rand::rngs::ThreadRng;
use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;
use std::hash::Hash;
use crate::snake_world::{SnakeWorldConfig, SnakeWorldWorldView, SnakeWorldSnakeController, SnakeWorldObjectType};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::borrow::{BorrowMut, Borrow};

type Config = SnakeGameConfig;
type CreateError = SnakeGameCreateError;
type ActionType = SnakeGameActionType;
type TickType = SnakeGameTickType;
type GameController = dyn SnakeGameGameController;

pub struct SnakeGameConfig {
    pub game_controller: Rc<RefCell<GameController>>
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
    fn game_will_tick(&mut self, world_view: SnakeWorldWorldView) -> TickType;
    fn game_did_tick(&mut self, world_view: SnakeWorldWorldView);
    fn game_map_update(&mut self, map: HashMap<Point<u16>, SnakeWorldObjectType>);
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
        'game_loop: loop {
            match self.config.game_controller().game_action() {
                SnakeGameActionType::Start => {},
                SnakeGameActionType::Exit => break 'game_loop,
            }
            let world_config = self.config.game_controller().game_start();
            match SnakeWorld::try_create(world_config) {
                Ok(mut world) => {
                    'tick_loop: loop {
                        let world_view = world.get_world_view();
                        let tick_type = self.config.game_controller().game_will_tick(world_view);
                        match tick_type {
                            SnakeGameTickType::Initial => world.tick(true),
                            SnakeGameTickType::Common => world.tick(false),
                            SnakeGameTickType::Break => break 'tick_loop,
                        };
                        let world_view = world.get_world_view();
                        self.config.game_controller().game_did_tick(world_view);
                        self.config.game_controller().game_map_update(world.generate_map());
                    }
                    self.config.game_controller().game_end(Ok(()));
                }
                Err(err) => {
                    self.config.game_controller().game_end(Err(err));
                }
            }
        }
    }
}