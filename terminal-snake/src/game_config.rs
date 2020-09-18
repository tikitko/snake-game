use super::terminal::{
    TerminalSize,
    Terminal,
    KeyCode,
    current_key_code,
    enable_raw_mode,
    disable_raw_mode,
    size
};
use super::snake::{
    Direction,
    Point
};
use super::snake::world::{
    SnakeController,
    ObjectType as WorldObjectType,
    Config as WorldConfig,
    CreateError as WorldCreateError,
    WorldView,
    SnakeInfo
};
use super::snake::game::{
    GameController,
    ActionType as GameActionType,
    TickType as GameTickType,
    Config as GameConfig
};

use std::time::{Duration, SystemTime};
use std::thread;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

pub fn new() -> GameConfig {
    GameConfig {
        game_controller: Rc::new(RefCell::new(TerminalGameController::new())),
    }
}

struct TerminalGameController {
    terminal: Terminal,
    last_tick_start: Option<SystemTime>,
    first_snake: Rc<RefCell<DirectionSnakeController>>,
    second_snake: Rc<RefCell<DirectionSnakeController>>,
}

impl TerminalGameController {
    fn new() -> Self {
        Self {
            terminal: Terminal::new(),
            last_tick_start: None,
            first_snake: Rc::new(RefCell::new(DirectionSnakeController {
                next_direction: Direction::Right,
            })),
            second_snake: Rc::new(RefCell::new(DirectionSnakeController {
                next_direction: Direction::Right,
            })),
        }
    }
    fn delay_if_needed(&mut self) {
        const MINIMUM_DELAY_MILLIS: u64 = 150;
        match self.last_tick_start.and_then(|v| v.elapsed().ok()) {
            Some(difference) => {
                let after_time = difference.as_millis() as u64;
                if after_time < MINIMUM_DELAY_MILLIS {
                    let delay_time = MINIMUM_DELAY_MILLIS - after_time;
                    thread::sleep(Duration::from_millis(delay_time));
                }
            },
            None => thread::sleep(Duration::from_millis(MINIMUM_DELAY_MILLIS)),
        }
        self.last_tick_start = Some(SystemTime::now());
    }
}

impl GameController for TerminalGameController {
    fn game_action(&mut self) -> GameActionType {
        if let Ok(mut first_snake) = self.first_snake.try_borrow_mut() {
            first_snake.next_direction = Direction::Right;
        }
        if let Ok(mut second_snake) = self.second_snake.try_borrow_mut() {
            second_snake.next_direction = Direction::Right;
        }
        let last_tick_start = self.last_tick_start;
        self.last_tick_start = None;
        match last_tick_start {
            Some(_) => GameActionType::Exit,
            None => GameActionType::Start,
        }
    }
    fn game_start(&mut self) -> WorldConfig {
        let _ = enable_raw_mode();
        let _ = self.terminal.clear();
        let mut controllers = HashMap::<usize, Rc<RefCell<dyn SnakeController>>>::new();
        controllers.insert(0, self.first_snake.clone());
        controllers.insert(1, self.second_snake.clone());
        WorldConfig {
            world_size: size().unwrap_or((50, 50)),
            eat_count: 3,
            cut_tails: true,
            base_snake_tail_size: 3,
            snakes_controllers: controllers,
        }
    }
    fn game_will_tick(&mut self, previous_world_view: &Option<WorldView>) -> GameTickType {
        self.delay_if_needed();
        match previous_world_view {
            Some(world_view) => {
                if world_view.get_snakes_info().len() == 0 {
                    return GameTickType::Break;
                }
                let current_key_code = current_key_code(Duration::from_millis(0));
                match current_key_code.ok() {
                    Some(key_code) => {
                        if key_code == KeyCode::Esc {
                            return GameTickType::Break;
                        }
                        match self.first_snake.try_borrow_mut() {
                            Ok(mut first_snake) => {
                                first_snake.next_direction = match key_code {
                                    KeyCode::Char('d') => Direction::Right,
                                    KeyCode::Char('a') => Direction::Left,
                                    KeyCode::Char('w') => Direction::Up,
                                    KeyCode::Char('s') => Direction::Down,
                                    _ => first_snake.next_direction,
                                };
                            },
                            Err(_) => return GameTickType::Break,
                        }
                        match self.second_snake.try_borrow_mut() {
                            Ok(mut second_snake) => {
                                second_snake.next_direction = match key_code {
                                    KeyCode::Right => Direction::Right,
                                    KeyCode::Left => Direction::Left,
                                    KeyCode::Up => Direction::Up,
                                    KeyCode::Down => Direction::Down,
                                    _ => second_snake.next_direction,
                                };
                            },
                            Err(_) => return GameTickType::Break,
                        }
                        GameTickType::Common
                    },
                    None => GameTickType::Common,
                }
            },
            None => GameTickType::Initial,
        }
    }
    fn game_did_tick(&mut self, world_view: &WorldView) {
        let points_mapper = |point: &Point<TerminalSize>| (point.x(), point.y());
        let objects_mapper = |object: &WorldObjectType| match object {
            WorldObjectType::Border => '#',
            WorldObjectType::Snake(number) => match number {
                0 => 'o',
                1 => 'x',
                _ => unreachable!(),
            },
            WorldObjectType::Eat => '@',
        };
        let map = world_view.get_world_mask().generate_map(points_mapper, objects_mapper);
        let _ = self.terminal.render(&map);
    }
    fn game_end(&mut self, _: Result<(), WorldCreateError>) {
        let _ = self.terminal.clear();
        let _ = disable_raw_mode();
    }
}

struct DirectionSnakeController {
    next_direction: Direction,
}

impl SnakeController for DirectionSnakeController {
    fn snake_will_burn(&mut self, _: &WorldView) {}
    fn snake_did_burn(&mut self, _: &SnakeInfo, _: &WorldView) {}
    fn snake_will_move(&mut self, _: &SnakeInfo, _: &WorldView) -> Direction {
        self.next_direction
    }
    fn snake_did_move(&mut self, _: &SnakeInfo, _: &WorldView) {}
    fn snake_will_eat(&mut self, _: bool, _: &SnakeInfo, _: &WorldView) {}
    fn snake_did_eat(&mut self, _: bool, _: &SnakeInfo, _: &WorldView) {}
    fn snake_will_died(&mut self, _: &SnakeInfo, _: &WorldView) {}
    fn snake_did_died(&mut self, _: &WorldView) {}
}