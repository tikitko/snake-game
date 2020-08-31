#![allow(dead_code)]

extern crate crossterm;

mod base;
mod snake;
mod terminal;

use base::direction::Direction;
use snake::{game, world};

use terminal::{TerminalPixel, Terminal, KeyCode};
use std::time::{Duration, SystemTime};
use std::thread;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

impl TerminalPixel for world::ObjectType {
    fn char(&self) -> char {
        match self {
            world::ObjectType::Border => '#',
            world::ObjectType::Snake(_) => 'o',
            world::ObjectType::Eat => '@',
        }
    }
}

struct TerminalSnakeController {
    control_number: usize,
    terminal: Rc<RefCell<Terminal>>,
    current_key_code: Rc<RefCell<Option<KeyCode>>>,
}

impl world::SnakeController for TerminalSnakeController {
    fn snake_will_burn(&mut self, world_view: &world::WorldView) {}
    fn snake_did_burn(&mut self, self_info: &world::SnakeInfo, world_view: &world::WorldView) {}
    fn snake_will_move(&mut self, self_info: &world::SnakeInfo, world_view: &world::WorldView) -> Direction {
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
    fn snake_did_move(&mut self, self_info: &world::SnakeInfo, world_view: &world::WorldView) {}
    fn snake_will_eat(&mut self, good_eat: bool, self_info: &world::SnakeInfo, world_view: &world::WorldView) {}
    fn snake_did_eat(&mut self, good_eat: bool, self_info: &world::SnakeInfo, world_view: &world::WorldView) {}
    fn snake_will_died(&mut self, self_info: &world::SnakeInfo, world_view: &world::WorldView) {}
    fn snake_did_died(&mut self, world_view: &world::WorldView) {}
}

struct TerminalSnakeGameController {
    last_tick_start: Option<SystemTime>,
    terminal: Rc<RefCell<Terminal>>,
    current_key_code: Rc<RefCell<Option<KeyCode>>>,
}

impl game::GameController for TerminalSnakeGameController {
    fn game_action(&mut self) -> game::ActionType {
        game::ActionType::Start
    }
    fn game_start(&mut self) -> world::Config {
        Terminal::enable_raw_mode();

        let mut controllers = HashMap::<usize, Rc<RefCell<dyn world::SnakeController>>>::new();
        for i in 0..2 {
            controllers.insert(i, Rc::new(RefCell::new(TerminalSnakeController {
                control_number: i,
                terminal: self.terminal.clone(),
                current_key_code:
                self.current_key_code.clone()
            })));
        }
        world::Config {
            world_size: (100, 30),
            eat_count: 3,
            cut_tails: true,
            base_snake_tail_size: 3,
            snakes_controllers: controllers,
        }
    }
    fn game_will_tick(&mut self, previous_world_view: &Option<world::WorldView>) -> game::TickType {
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
                game::TickType::Common
            }
            None => {
                thread::sleep(Duration::from_millis(minimum_delay_millis));
                self.last_tick_start = Some(SystemTime::now());
                game::TickType::Initial
            }
        }
    }
    fn game_did_tick(&mut self, world_view: &world::WorldView) {
        let map = world_view.generate_map();
        self.terminal.borrow_mut().render_points(&map);
    }
    fn game_end(&mut self, state: Result<(), world::CreateError>) {
        Terminal::disable_raw_mode();
    }
}

fn main() {
    let game_controller = TerminalSnakeGameController {
        last_tick_start: None,
        terminal: Rc::new(RefCell::new(Terminal::new())),
        current_key_code: Rc::new(RefCell::new(None)),
    };
    let game_config = game::Config {
        game_controller: Rc::new(RefCell::new(game_controller))
    };
    let mut game = game::Game::try_create(game_config).unwrap();
    game.start();
}