#![allow(dead_code)]

mod world;
mod snake_game;
mod terminal;
mod snake;
mod node;
mod point;
mod direction;
mod snake_world;

extern crate crossterm;

use crate::snake_game::{SnakeGame, SnakeGameCreateError, SnakeGameConfig, SnakeGameController, SnakeGameTickType};
use crate::snake_world::{SnakeWorld, SnakeWorldCreateError, SnakeWorldConfig, SnakeWorldSnakeController, SnakeWorldView, SnakeWorldObjectType, SnakeWorldSnakeInfo};
use crate::terminal::{ErrorKind, TerminalPixel, Terminal, KeyCode, Result};
use crate::direction::Direction;
use std::time::Duration;
use std::thread;
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use crate::point::Point;

#[derive(Debug)]
enum GameError {
    SnakeGameCreateError(SnakeGameCreateError),
    SnakeWorldCreateError(SnakeWorldCreateError),
}

impl TerminalPixel for SnakeWorldObjectType {
    fn char(&self) -> char {
        match self {
            SnakeWorldObjectType::Border => '#',
            SnakeWorldObjectType::Snake(_) => 'o',
            SnakeWorldObjectType::Eat => '@',
        }
    }
}

struct MainSnakeController;
impl SnakeWorldSnakeController for MainSnakeController {
    fn snake_burn(&mut self, self_info: &SnakeWorldSnakeInfo, world_view: &SnakeWorldView) {}
    fn snake_move(&mut self, self_info: &SnakeWorldSnakeInfo, world_view: &SnakeWorldView) -> Direction {
        Direction::Right
    }
    fn snake_died(&mut self, self_info: &SnakeWorldSnakeInfo, world_view: &SnakeWorldView) {}
}
impl SnakeGameController for MainSnakeController {
    fn game_start(&mut self){
        unimplemented!()
    }

    fn game_tick(&mut self, world_view: &SnakeWorldView) -> SnakeGameTickType {
        unimplemented!()
    }

    fn game_snakes_controllers(&mut self) -> HashMap<usize, Box<&mut dyn SnakeWorldSnakeController>> {
        HashMap::new()
        /*let mut controllers = HashMap::<usize, &mut Box<dyn SnakeWorldSnakeController>>::new();
        let mut controller: Box<dyn SnakeWorldSnakeController> = Box::new(self);
        controllers.insert(0, &mut controller);
        controllers*/
    }

    fn game_map_update(&mut self, map: HashMap<Point<u16>, SnakeWorldObjectType>) {
        unimplemented!()
    }

    fn game_end(&mut self) {
        unimplemented!()
    }
}

fn main() -> core::result::Result<(), GameError> {
    let mut controller = MainSnakeController {};
    let mut controller: Box<&mut dyn SnakeGameController> = Box::new(&mut controller);

    let snake_world_config = SnakeWorldConfig {
        world_size: (100, 30),
        eat_count: 3,
        snakes_count: 1,
    };
    let snake_world = SnakeWorld::try_create(snake_world_config)
        .map_err(|err| GameError::SnakeWorldCreateError(err))?;
    let snake_game_config = SnakeGameConfig {};
    let mut snake_world = SnakeGame::try_create(snake_game_config, snake_world)
        .map_err(|err| GameError::SnakeGameCreateError(err))?;
    snake_world.game_start(controller);
    Ok(())
}

/*fn start_snake_game(snake_game: &mut SnakeGame, terminal: &mut Terminal) -> Result<()> {
    Terminal::enable_raw_mode()?;
    terminal.clear()?;
    loop {
        let mut controllers_directions = HashMap::new();
        match Terminal::current_key_code(Duration::from_millis(0))? {
            KeyCode::Char('d') => controllers_directions.insert(0, Some(Direction::Right)),
            KeyCode::Char('a') => controllers_directions.insert(0, Some(Direction::Left)),
            KeyCode::Char('w') => controllers_directions.insert(0, Some(Direction::Up)),
            KeyCode::Char('s') => controllers_directions.insert(0, Some(Direction::Down)),
            KeyCode::Right => controllers_directions.insert(1, Some(Direction::Right)),
            KeyCode::Left => controllers_directions.insert(1, Some(Direction::Left)),
            KeyCode::Up => controllers_directions.insert(1, Some(Direction::Up)),
            KeyCode::Down => controllers_directions.insert(1, Some(Direction::Down)),
            KeyCode::Char('q') => break,
            _ => None
        };
        snake_game.game_tick();
        let map = snake_game.generate_map();
        terminal.render_points(&map)?;

    }   thread::sleep(Duration::from_millis(100));
    Terminal::disable_raw_mode()?;
    Ok(())
}*/
