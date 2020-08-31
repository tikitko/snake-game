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

use crate::snake_game::{SnakeGame, SnakeGameConfig, SnakeGameGameController, SnakeGameTickType, SnakeGameActionType};
use crate::snake_world::{SnakeWorldCreateError, SnakeWorldConfig, SnakeWorldSnakeController, SnakeWorldWorldView, SnakeWorldObjectType, SnakeWorldSnakeInfo};
use crate::terminal::{TerminalPixel, Terminal, KeyCode};
use crate::direction::Direction;
use std::time::{Duration, SystemTime};
use std::thread;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::panic;

impl TerminalPixel for SnakeWorldObjectType {
    fn char(&self) -> char {
        match self {
            SnakeWorldObjectType::Border => '#',
            SnakeWorldObjectType::Snake(n) => ((n.clone() as u8 + 1) as char)/*'o'*/,
            SnakeWorldObjectType::Eat => '@',
        }
    }
}

struct TerminalSnakeController {
    control_number: usize,
    terminal: Rc<RefCell<Terminal>>,
    current_key_code: Rc<RefCell<Option<KeyCode>>>,
}

impl SnakeWorldSnakeController for TerminalSnakeController {
    fn snake_will_burn(&mut self, world_view: &SnakeWorldWorldView) {}

    fn snake_did_burn(&mut self, self_info: &SnakeWorldSnakeInfo, world_view: &SnakeWorldWorldView) {}

    fn snake_will_move(&mut self, self_info: &SnakeWorldSnakeInfo, world_view: &SnakeWorldWorldView) -> Direction {
        match self.control_number {
            0 => match self.current_key_code.borrow().as_ref().unwrap_or(&KeyCode::Char('d')) {
                KeyCode::Char('d') => Direction::Right,
                KeyCode::Char('a') => Direction::Left,
                KeyCode::Char('w') => Direction::Up,
                KeyCode::Char('s') => Direction::Down,
                _ => {
                    match self_info.get_direction() {
                        Some(v) => v.clone(),
                        None => Direction::Right,
                    }
                }
            },
            1 => match self.current_key_code.borrow().as_ref().unwrap_or(&KeyCode::Right) {
                KeyCode::Right => Direction::Right,
                KeyCode::Left => Direction::Left,
                KeyCode::Up => Direction::Up,
                KeyCode::Down => Direction::Down,
                _ => {
                    match self_info.get_direction() {
                        Some(v) => v.clone(),
                        None => Direction::Right,
                    }
                }
            },
            _ => Direction::Right,
        }
    }

    fn snake_did_move(&mut self, self_info: &SnakeWorldSnakeInfo, world_view: &SnakeWorldWorldView) {}

    fn snake_will_eat(&mut self, good_eat: bool, self_info: &SnakeWorldSnakeInfo, world_view: &SnakeWorldWorldView) {}

    fn snake_did_eat(&mut self, good_eat: bool, self_info: &SnakeWorldSnakeInfo, world_view: &SnakeWorldWorldView) {}

    fn snake_will_died(&mut self, self_info: &SnakeWorldSnakeInfo, world_view: &SnakeWorldWorldView) {}

    fn snake_did_died(&mut self, world_view: &SnakeWorldWorldView) {}
}

struct TerminalSnakeGameController {
    last_tick_start: Option<SystemTime>,
    terminal: Rc<RefCell<Terminal>>,
    current_key_code: Rc<RefCell<Option<KeyCode>>>,
}

impl SnakeGameGameController for TerminalSnakeGameController {
    fn game_action(&mut self) -> SnakeGameActionType {
        SnakeGameActionType::Start
    }

    fn game_start(&mut self) -> SnakeWorldConfig {
        Terminal::enable_raw_mode();

        let mut controllers: HashMap<usize, Rc<RefCell<dyn SnakeWorldSnakeController>>> = HashMap::new();
        for i in 0..2 {
            controllers.insert(i, Rc::new(RefCell::new(TerminalSnakeController { control_number: i, terminal: self.terminal.clone(), current_key_code: self.current_key_code.clone() })));
        }
        SnakeWorldConfig {
            world_size: (100, 30),
            eat_count: 3,
            cut_tails: true,
            base_snake_tail_size: 3,
            snakes_controllers: controllers,
        }
    }

    fn game_will_tick(&mut self, previous_world_view: &Option<SnakeWorldWorldView>) -> SnakeGameTickType {
        let current_key_code = Terminal::current_key_code(Duration::from_millis(0));
        self.current_key_code.replace(match current_key_code {
            Ok(key_code) => Some(key_code),
            Err(_) => None,
        });

        let minimum_delay_millis = 150;
        match self.last_tick_start {
            Some(time) => {
                match time.elapsed() {
                    Ok(difference) => {
                        let after_time = difference.as_millis() as u64;
                        if after_time < minimum_delay_millis {
                            thread::sleep(Duration::from_millis(minimum_delay_millis - after_time))
                        }
                    }
                    Err(_) => thread::sleep(Duration::from_millis(minimum_delay_millis))
                }
                self.last_tick_start = Some(SystemTime::now());
                SnakeGameTickType::Common
            }
            None => {
                thread::sleep(Duration::from_millis(minimum_delay_millis));
                self.last_tick_start = Some(SystemTime::now());
                SnakeGameTickType::Initial
            }
        }
    }

    fn game_did_tick(&mut self, world_view: &SnakeWorldWorldView) {
        let map = world_view.generate_map();
        self.terminal.borrow_mut().render_points(&map);
    }

    fn game_end(&mut self, state: Result<(), SnakeWorldCreateError>) {
        Terminal::disable_raw_mode();
    }
}

fn main() {
    panic::set_hook(Box::new(|panic_info| {
        println!("panic occurred: {:?}", panic_info.location());
        thread::sleep(Duration::from_secs(5));
    }));

    let controller = TerminalSnakeGameController {
        last_tick_start: None,
        terminal: Rc::new(RefCell::new(Terminal::new())),
        current_key_code: Rc::new(RefCell::new(None)),
    };
    let config = SnakeGameConfig {
        game_controller: Rc::new(RefCell::new(controller))
    };

    SnakeGame::try_create(config).unwrap().start();
}