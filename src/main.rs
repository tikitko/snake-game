#![allow(dead_code)]

mod world;
mod snake_game;
mod terminal;
mod snake;
mod node;
mod point;
mod direction;

extern crate crossterm;

use crate::snake_game::{SnakeGameCreateError, SnakeGameObjectType, SnakeGame, SnakeGameTickData};
use crate::terminal::{ErrorKind, TerminalPixel, Terminal, KeyCode, Result};
use crate::direction::Direction;
use std::time::Duration;
use std::thread;
use std::collections::HashMap;

#[derive(Debug)]
enum GameError {
    SnakeGameCreateError(SnakeGameCreateError),
    TerminalError(ErrorKind),
}

impl TerminalPixel for SnakeGameObjectType {
    fn char(&self) -> char {
        match self {
            SnakeGameObjectType::Border => '#',
            SnakeGameObjectType::Snake(_) => 'o',
            SnakeGameObjectType::Eat => '@',
        }
    }
}

fn main() -> core::result::Result<(), GameError> {
    let snake_game_config = snake_game::SnakeGameConfig {
        players_count: 2,
        world_size: (100, 30),
        eat_count: 3,
    };
    let mut terminal = Terminal::new();
    let mut snake_game = SnakeGame::try_create(snake_game_config)
        .map_err(|err| GameError::SnakeGameCreateError(err))?;
    start_snake_game(&mut snake_game, &mut terminal)
        .map_err(|err| GameError::TerminalError(err))?;
    Ok(())
}

fn start_snake_game(snake_game: &mut SnakeGame, terminal: &mut Terminal) -> Result<()> {
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
        snake_game.game_tick(SnakeGameTickData {
            controllers_directions,
        });
        let map = snake_game.generate_map();
        terminal.render_points(&map)?;
        thread::sleep(Duration::from_millis(100));
    }
    Terminal::disable_raw_mode()?;
    Ok(())
}
