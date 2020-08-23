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
use std::cell::RefCell;
use std::rc::Rc;

type Config = SnakeGameConfig;
type CreateError = SnakeGameCreateError;
type ActionType = SnakeGameActionType;
type TickType = SnakeGameTickType;
type GameController = dyn SnakeGameGameController;

pub struct SnakeGameConfig {
    pub game_controller: Box<RefCell<GameController>>
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
    Common,
    EndGame,
}

pub trait SnakeGameGameController {
    fn game_action(&mut self) -> ActionType;
    fn game_world_config(&mut self) -> SnakeWorldConfig;
    fn game_world_create_state(&mut self, state: Result<(), SnakeWorldCreateError>);
    fn game_start(&mut self);
    fn game_tick(&mut self, world_view: SnakeWorldWorldView) -> TickType;
    fn game_map_update(&mut self, map: HashMap<Point<u16>, SnakeWorldObjectType>);
    fn game_end(&mut self);
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
        loop {
            match self.config.game_controller.borrow_mut().game_action() {
                SnakeGameActionType::Start => {},
                SnakeGameActionType::Exit => break,
            }

            let world_config = self.config.game_controller.borrow_mut().game_world_config();
            match SnakeWorld::try_create(world_config) {
                Ok(mut world) => {
                    self.config.game_controller.borrow_mut().game_world_create_state(Ok(()));

                    self.config.game_controller.borrow_mut().game_start();
                    loop {
                        self.config.game_controller.borrow_mut().game_tick(world.get_world_view());
                        world.tick();
                        self.config.game_controller.borrow_mut().game_map_update(world.generate_map());
                    }
                    self.config.game_controller.borrow_mut().game_end();
                }
                Err(err) => {
                    self.config.game_controller.borrow_mut().game_world_create_state(Err(err));
                }
            }
        }
    }
}