use super::snake;
use super::terminal;
use super::point;
use super::world;
use snake::{Snake, MoveDirection};
use terminal::{Terminal, TerminalPixel, KeyCode, ErrorKind};
use point::Point;
use world::World;

use rand::Rng;
use rand::rngs::ThreadRng;
use std::collections::{HashSet, HashMap};
use std::time::Duration;
use std::thread;
use std::iter::FromIterator;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum ObjectType {
    Border,
    Snake(usize),
    Eat,
}

impl TerminalPixel for ObjectType {
    fn char(&self) -> char {
        match self {
            ObjectType::Border => '#',
            ObjectType::Snake(_) => 'o',
            ObjectType::Eat => '@',
        }
    }
}

pub struct SnakeGameConfig {
    pub two_players: bool,
    pub world_size: (u16, u16),
    pub eat_count: u16,
}

pub enum SnakeGameCreateError {
    WorldSmall,
    WorldLarge,
    FoodExcess,
    FoodLack,
}

struct SnakeInfo {
    snake: Snake<u16>,
    direction: Option<MoveDirection>,
}

pub struct SnakeGame {
    world: World<ObjectType, u16>,
    snakes_info: HashMap<usize, SnakeInfo>,
    border_points: HashSet<Point<u16>>,
    eat_points: HashSet<Point<u16>>,
    config: SnakeGameConfig,
    terminal: Terminal,
    rng: ThreadRng,
}

impl SnakeGame {
    pub fn try_create(config: SnakeGameConfig) -> Result<Self, SnakeGameCreateError> {
        if config.world_size.0 < 10 && config.world_size.1 < 10 {
            Err(SnakeGameCreateError::WorldSmall)
        }
        if config.world_size.0 > 100 && config.world_size.1 > 100 {
            Err(SnakeGameCreateError::WorldLarge)
        }
        if config.eat_count < 1 {
            Err(SnakeGameCreateError::FoodLack)
        }
        if config.eat_count > 10 {
            Err(SnakeGameCreateError::FoodExcess)
        }
        Ok(SnakeGame {
            world: World::new(),
            snakes_info: HashMap::new(),
            border_points: HashSet::new(),
            eat_points: HashSet::new(),
            config,
            terminal: Terminal::new(),
            rng: rand::thread_rng(),
        })
    }
    pub fn start(&mut self) -> Result<(), ErrorKind> {
        Terminal::enable_raw_mode()?;
        self.terminal.clear()?;
        self.border_points = {
            let mut border_points = HashSet::new();
            for i in 0..self.config.world_size.0 {
                for j in 0..self.config.world_size.1 {
                    let max_i = self.config.world_size.0 - 1;
                    let max_j = self.config.world_size.1 - 1;
                    if i == 0 || j == 0 || i == max_i || j == max_j {
                        border_points.insert(Point::new(i, j));
                    }
                }
            }
            border_points
        };
        loop {
            if self.snakes_info.len() == 0 {
                self.snakes_info = {
                    let mut snakes = HashMap::new();
                    snakes.insert(0, SnakeInfo {
                        snake: Snake::make_on(Point::new(6, 3)),
                        direction: None,
                    });
                    if self.config.two_players {
                        snakes.insert(1, SnakeInfo {
                            snake: Snake::make_on(Point::new(6, 6)),
                            direction: None,
                        });
                    }
                    snakes
                };
            }
            let current_key_code = Terminal::current_key_code(Duration::from_millis(0))?;
            if let Some(snake_info) = self.snakes_info.get_mut(&0) {
                let direction = match &current_key_code {
                    KeyCode::Char('d') => Some(MoveDirection::Right),
                    KeyCode::Char('a') => Some(MoveDirection::Left),
                    KeyCode::Char('w') => Some(MoveDirection::Up),
                    KeyCode::Char('s') => Some(MoveDirection::Down),
                    _ => None
                };
                if let Some(direction) = direction {
                    snake_info.direction = Some(direction);
                }
            }
            if let Some(snake_info) = self.snakes_info.get_mut(&1) {
                let direction = match &current_key_code {
                    KeyCode::Right => Some(MoveDirection::Right),
                    KeyCode::Left => Some(MoveDirection::Left),
                    KeyCode::Up => Some(MoveDirection::Up),
                    KeyCode::Down => Some(MoveDirection::Down),
                    _ => None,
                };
                if let Some(direction) = direction {
                    snake_info.direction = Some(direction);
                }
            }
            match &current_key_code {
                KeyCode::Char('q') => break,
                _ => {}
            }
            self.game_tick();
            let map = self.world.generate_map();
            self.terminal.render_points(&map)?;
            thread::sleep(Duration::from_millis(100));
        }
        Terminal::disable_raw_mode()?;
        Ok(())
    }
    fn game_tick(&mut self) {
        self.world.set_layer(ObjectType::Border, self.border_points.clone());
        let mut snakes_move_vectors = HashMap::new();
        for (snake_number, snake_info) in &mut self.snakes_info {
            let direction = snake_info.direction;
            if let Some(direction) = direction {
                snake_info.snake.move_to(direction);
            }
            let head_point = snake_info.snake.head_point();
            snakes_move_vectors.insert(snake_number.clone(), (direction, head_point));
        }
        let mut snakes_numbers_to_remove = HashSet::new();
        'main: for (snake_number, snake_info) in &mut self.snakes_info {
            let body_points = snake_info.snake.body_parts_points(true);
            let head_point = snake_info.snake.head_point();
            for (_, (vector_direction, vector_point)) in &snakes_move_vectors {
                if *vector_point != head_point {
                    continue;
                }
                if let Some(vector_direction) = vector_direction {
                    let vector_reversed_direction = vector_direction.reverse();
                    if Some(vector_reversed_direction) == snake_info.direction {
                        snakes_numbers_to_remove.insert(snake_number.clone());
                        continue 'main;
                    }
                }
            }
            let mut head_points_catch = false;
            for body_point in body_points {
                if head_point == body_point {
                    if head_points_catch {
                        snakes_numbers_to_remove.insert(snake_number.clone());
                        continue 'main;
                    }
                    head_points_catch = true
                }
                for object in self.world.point_occurrences(&body_point) {
                    if object == ObjectType::Snake(*snake_number) {
                        continue;
                    }
                    match object {
                        ObjectType::Snake(number) => if number != *snake_number {
                            snakes_numbers_to_remove.insert(snake_number.clone());
                            continue 'main;
                        },
                        ObjectType::Eat => {
                            snake_info.snake.fill_stomach_if_empty();
                            self.eat_points.remove(&body_point);
                        }
                        ObjectType::Border => {
                            snakes_numbers_to_remove.insert(snake_number.clone());
                            continue 'main;
                        }
                    }
                }
            }
        }
        for snake_remove_number in snakes_numbers_to_remove {
            self.snakes_info.remove(&snake_remove_number);
            self.world.remove_layer(&ObjectType::Snake(snake_remove_number))
        }
        for (snake_number, snake_info) in &self.snakes_info {
            let points = HashSet::from_iter(snake_info.snake.body_parts_points(true).clone());
            self.world.set_layer(ObjectType::Snake(snake_number.clone()), points)
        }
        let eat_to_spawn = self.config.eat_count - self.eat_points.len();
        for _ in 0..eat_to_spawn {
            loop {
                let x = self.rng.gen_range(1, self.config.world_size.0 - 1);
                let y = self.rng.gen_range(1, self.config.world_size.1 - 1);
                let point = Point::new(x, y);
                if self.world.point_occurrences(&point).len() == 0 {
                    self.eat_points.insert(point);
                    break;
                }
            }
        }
        self.world.set_layer(ObjectType::Eat, self.eat_points.clone())
    }
}