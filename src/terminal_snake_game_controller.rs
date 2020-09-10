use super::base::direction::Direction;
use super::snake::{game, world};
use super::terminal::{TerminalPixel, Terminal, KeyCode};

use std::time::{Duration, SystemTime};
use std::thread;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

impl TerminalPixel for world::ObjectType {
    fn char(&self) -> char {
        match self {
            world::ObjectType::Border => '#',
            world::ObjectType::Snake(number) => match number {
                0 => 'o',
                1 => 'x',
                _ => ' ',
            },
            world::ObjectType::Eat => '@',
        }
    }
}

pub struct GameController {
    terminal: Terminal,
    last_tick_start: Option<SystemTime>,
    first_snake: Rc<RefCell<SnakeController>>,
    second_snake: Rc<RefCell<SnakeController>>,
}

impl GameController {
    pub fn new() -> Self {
        Self {
            terminal: Terminal::new(),
            last_tick_start: None,
            first_snake: Rc::new(RefCell::new(SnakeController {
                next_direction: Direction::Right
            })),
            second_snake: Rc::new(RefCell::new(SnakeController {
                next_direction: Direction::Right
            })),
        }
    }
    fn delay_if_needed(&mut self) {
        let minimum_delay_millis = 150;
        match self.last_tick_start.and_then(|v| v.elapsed().ok()) {
            Some(difference) => {
                let after_time = difference.as_millis() as u64;
                if after_time < minimum_delay_millis {
                    let delay_time = minimum_delay_millis - after_time;
                    thread::sleep(Duration::from_millis(delay_time))
                }
            }
            None => thread::sleep(Duration::from_millis(minimum_delay_millis)),
        }
        self.last_tick_start = Some(SystemTime::now());
    }
}

impl game::GameController for GameController {
    fn game_action(&mut self) -> game::ActionType {
        self.first_snake.borrow_mut().next_direction = Direction::Right;
        self.second_snake.borrow_mut().next_direction = Direction::Right;
        let last_tick_start = self.last_tick_start;
        self.last_tick_start = None;
        match last_tick_start {
            Some(_) => game::ActionType::Exit,
            None => game::ActionType::Start,
        }
    }
    fn game_start(&mut self) -> world::Config {
        let _ = Terminal::enable_raw_mode();
        let _ = self.terminal.clear();
        let mut controllers = HashMap::<usize, Rc<RefCell<dyn world::SnakeController>>>::new();
        controllers.insert(0, self.first_snake.clone());
        controllers.insert(1, self.second_snake.clone());
        world::Config {
            world_size: Terminal::size().unwrap_or((50, 50)),
            eat_count: 3,
            cut_tails: true,
            base_snake_tail_size: 3,
            snakes_controllers: controllers,
        }
    }
    fn game_will_tick(&mut self, world_view: &Option<world::WorldView>) -> game::TickType {
        self.delay_if_needed();
        match world_view {
            Some(world_view) => {
                if world_view.get_snakes_info().len() == 0 {
                    return game::TickType::Break;
                }
                let current_key_code = Terminal::current_key_code(Duration::from_millis(0));
                match current_key_code.ok() {
                    Some(key_code) => {
                        if key_code == KeyCode::Esc {
                            return game::TickType::Break;
                        }
                        let mut first_snake = self.first_snake.borrow_mut();
                        first_snake.next_direction = match key_code {
                            KeyCode::Char('d') => Direction::Right,
                            KeyCode::Char('a') => Direction::Left,
                            KeyCode::Char('w') => Direction::Up,
                            KeyCode::Char('s') => Direction::Down,
                            _ => first_snake.next_direction,
                        };
                        let mut second_snake = self.second_snake.borrow_mut();
                        second_snake.next_direction = match key_code {
                            KeyCode::Right => Direction::Right,
                            KeyCode::Left => Direction::Left,
                            KeyCode::Up => Direction::Up,
                            KeyCode::Down => Direction::Down,
                            _ => second_snake.next_direction,
                        };
                        game::TickType::Common
                    }
                    None => game::TickType::Common,
                }
            }
            None => game::TickType::Initial,
        }
    }
    fn game_did_tick(&mut self, world_view: &world::WorldView) {
        let map = world_view.get_world_mask().generate_map();
        let _ = self.terminal.render_points(&map);
    }
    fn game_end(&mut self, _: Result<(), world::CreateError>) {
        let _ = self.terminal.clear();
        let _ = Terminal::disable_raw_mode();
    }
}

struct SnakeController {
    next_direction: Direction,
}

impl world::SnakeController for SnakeController {
    fn snake_will_burn(&mut self, _: &world::WorldView) {}
    fn snake_did_burn(&mut self, _: &world::SnakeInfo, _: &world::WorldView) {}
    fn snake_will_move(&mut self, _: &world::SnakeInfo, _: &world::WorldView) -> Direction {
        self.next_direction
    }
    fn snake_did_move(&mut self, _: &world::SnakeInfo, _: &world::WorldView) {}
    fn snake_will_eat(&mut self, _: bool, _: &world::SnakeInfo, _: &world::WorldView) {}
    fn snake_did_eat(&mut self, _: bool, _: &world::SnakeInfo, _: &world::WorldView) {}
    fn snake_will_died(&mut self, _: &world::SnakeInfo, _: &world::WorldView) {}
    fn snake_did_died(&mut self, _: &world::WorldView) {}
}