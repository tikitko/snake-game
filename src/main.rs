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

use crate::snake_game::{SnakeGame, SnakeGameCreateError, SnakeGameConfig, SnakeGameGameController, SnakeGameTickType, SnakeGameActionType};
use crate::snake_world::{SnakeWorld, SnakeWorldCreateError, SnakeWorldConfig, SnakeWorldSnakeController, SnakeWorldWorldView, SnakeWorldObjectType, SnakeWorldSnakeInfo};
use crate::terminal::{ErrorKind, TerminalPixel, Terminal, KeyCode};
use crate::direction::Direction;
use std::time::Duration;
use std::thread;
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use crate::point::Point;
use std::cell::RefCell;
use std::rc::Rc;

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

struct TerminalSnakeController {
    terminal: Rc<RefCell<Terminal>>
}
impl SnakeWorldSnakeController for TerminalSnakeController {
    fn snake_will_burn(&mut self, world_view: &SnakeWorldWorldView) {}

    fn snake_did_burn(&mut self, self_info: &SnakeWorldSnakeInfo, world_view: &SnakeWorldWorldView) {}

    fn snake_will_move(&mut self, self_info: &SnakeWorldSnakeInfo, world_view: &SnakeWorldWorldView) -> Direction {
        Direction::Right
    }

    fn snake_did_move(&mut self, self_info: &SnakeWorldSnakeInfo, world_view: &SnakeWorldWorldView) {}

    fn snake_will_died(&mut self, self_info: &SnakeWorldSnakeInfo, world_view: &SnakeWorldWorldView) {}

    fn snake_did_died(&mut self, world_view: &SnakeWorldWorldView) {}
}
struct TerminalSnakeGameController {
    is_inited: bool,
    terminal: Rc<RefCell<Terminal>>
}
impl SnakeGameGameController for TerminalSnakeGameController {
    fn game_action(&mut self) -> SnakeGameActionType {
        SnakeGameActionType::Start
    }

    fn game_start(&mut self) -> SnakeWorldConfig {
        let mut controllers: HashMap<usize, Rc<RefCell<dyn SnakeWorldSnakeController>>> = HashMap::new();
        for i in 0..2 {
            controllers.insert(i, Rc::new(RefCell::new(TerminalSnakeController { terminal: self.terminal.clone() })));
        }
        SnakeWorldConfig {
            world_size: (100, 30),
            eat_count: 3,
            snakes_controllers: controllers
        }
    }

    fn game_map_update(&mut self, map: HashMap<Point<u16>, SnakeWorldObjectType>) {
        self.terminal.borrow_mut().render_points(&map);
    }

    fn game_will_tick(&mut self, world_view: SnakeWorldWorldView) -> SnakeGameTickType {
        thread::sleep(Duration::from_millis(100));
        if self.is_inited {
            SnakeGameTickType::Common
        } else {
            self.is_inited = true;
            SnakeGameTickType::Initial
        }
    }

    fn game_did_tick(&mut self, world_view: SnakeWorldWorldView) {}

    fn game_end(&mut self, state: Result<(), SnakeWorldCreateError>) {}
}

fn main() {
    let terminal = Rc::new(RefCell::new(Terminal::new()));
    let controller = TerminalSnakeGameController { is_inited: false, terminal: terminal.clone() };
    let config = SnakeGameConfig { game_controller: Rc::new(RefCell::new(controller)) };

    let mut game = SnakeGame::try_create(config).unwrap();
    game.start();
    /*let snake_world_config = SnakeWorldConfig {
        world_size: (100, 30),
        eat_count: 3,
        snakes_count: 1,
        snakes_controllers: Default::default()
    };
    let snake_world = SnakeWorld::try_create(snake_world_config)
        .map_err(|err| GameError::SnakeWorldCreateError(err))?;
    let snake_game_config = SnakeGameConfig {};
    let mut snake_world = SnakeGame::try_create(snake_game_config, snake_world)
        .map_err(|err| GameError::SnakeGameCreateError(err))?;
    snake_world.game_start(controller);*/
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
