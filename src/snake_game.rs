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
use crate::snake_world::{SnakeWorldConfig, SnakeWorldView, SnakeWorldSnakeController, SnakeWorldObjectType};
use std::cell::RefCell;

type Config = SnakeGameConfig;
type CreateError = SnakeGameCreateError;
type TickType = SnakeGameTickType;
type Controller = dyn SnakeGameController;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SnakeGameConfig {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SnakeGameCreateError {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SnakeGameTickType {
    Common,
    EndGame,
}

pub trait SnakeGameController {
    fn game_start(&mut self) -> HashMap<usize, RefCell<&mut dyn SnakeWorldSnakeController>>;
    fn game_tick(&mut self, world_view: &SnakeWorldView) -> TickType;
    fn game_map_update(&mut self, map: HashMap<Point<u16>, SnakeWorldObjectType>);
    fn game_end(&mut self);
}

pub struct SnakeGame {
    config: Config,
    world: SnakeWorld,
}

impl SnakeGame {
    pub fn try_create(config: Config, world: SnakeWorld) -> Result<Self, CreateError> {
        Ok(SnakeGame {
            config,
            world,
        })
    }
    pub fn game_start(&mut self, controller: Box<&mut Controller>) {
        let snake_controllers = controller.game_start();
        loop {
            controller.game_tick(&self.world.get_view());
            self.world.tick(snake_controllers);
            controller.game_map_update(self.world.generate_map());
        }
        controller.game_end();
    }
}