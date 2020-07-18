use super::snake;
use super::terminal;
use super::point;
use snake::Snake;
use terminal::{Terminal, TerminalPixel};
use point::Point;

use crossterm::event::KeyCode;

use rand::Rng;
use rand::rngs::ThreadRng;

enum PointType {
    Border,
    Body,
    Eat,
    Air,
}

impl Copy for PointType {}

impl Clone for PointType {
    fn clone(&self) -> Self {
        *self
    }
}

impl TerminalPixel for PointType {
    fn char(&self) -> char {
        match self {
            PointType::Border => '#',
            PointType::Body => 'o',
            PointType::Eat => '@',
            PointType::Air => ' ',
        }
    }
}

type WorldMatrix = Vec<Vec<PointType>>;

pub struct SnakeGame {
    snake: Snake<u16>,
    default_world: WorldMatrix,
    terminal: Terminal,
    last_move_direction: Option<snake::MoveDirection>,
    border_points: Vec<Point<u16>>,
    eat_point: Option<Point<u16>>,
    rng: ThreadRng
}

impl SnakeGame {
    pub fn new(map_size: (u16, u16)) -> Self {
        let border_points = {
            let mut border_points: Vec<Point<u16>> = Vec::new();
            for i in 0..map_size.0 {
                for j in 0..map_size.1 {
                    if i == 0 || j == 0 || i == map_size.0 - 1 || j == map_size.1 - 1 {
                        border_points.push(Point::new(i, j));
                    }
                }
            }
            border_points
        };
        SnakeGame {
            snake: Snake::make_on(Point::new(5, 5)),
            default_world: vec![vec![PointType::Air; map_size.1 as usize]; map_size.0 as usize],
            terminal: Terminal::new(),
            last_move_direction: None,
            border_points,
            eat_point: None,
            rng: rand::thread_rng()
        }
    }
    pub fn tick(&mut self, event_key_code: KeyCode) {
        match event_key_code {
            KeyCode::Char('d') => self.last_move_direction = Some(snake::MoveDirection::Right),
            KeyCode::Char('a') => self.last_move_direction = Some(snake::MoveDirection::Left),
            KeyCode::Char('w') => self.last_move_direction = Some(snake::MoveDirection::Top),
            KeyCode::Char('s') => self.last_move_direction = Some(snake::MoveDirection::Bottom),
            _ => {}
        }
        if let Some(last_move_direction) = &self.last_move_direction {
            self.snake.move_to(last_move_direction.clone());
        }
        for (i, snake_point_in) in self.snake.body_parts_points().iter().enumerate() {
            for (j, snake_point_out) in self.snake.body_parts_points().iter().enumerate() {
                if i != j && snake_point_in == snake_point_out {
                    self.recreate_game();
                    return
                }
            }
        }
        for border_point in &self.border_points {
            for snake_point in self.snake.body_parts_points() {
                if *border_point == snake_point {
                    self.recreate_game();
                    return
                }
            }
        }
        let snake_head_points = self.snake.head_point();
        match &self.eat_point {
            Some(eat_point) => if snake_head_points == *eat_point {
                self.snake.fill_stomach_if_empty();
                self.eat_point = None
            },
            None => {
                let x: u16 = self.rng.gen_range(1, self.default_world.len() as u16 - 1);
                let y: u16 = self.rng.gen_range(1, self.default_world[0].len() as u16 - 1);
                self.eat_point = Some(Point::new(x, y));
            }
        }
        self.render_map()
    }
    fn recreate_game(&mut self) {
        self.snake = Snake::make_on(Point::new(5, 5));
        self.last_move_direction = None;
    }
    fn render_map(&mut self) {
        let mut tick_world_map = self.default_world.clone();
        for border_point in &self.border_points {
            let x_index = border_point.x() as usize;
            let y_index = border_point.y() as usize;
            tick_world_map[x_index][y_index] = PointType::Border;
        }
        if let Some(eat_point) = &self.eat_point {
            let x_index = eat_point.x() as usize;
            let y_index = eat_point.y() as usize;
            tick_world_map[x_index][y_index] = PointType::Eat;
        }
        for snake_point in self.snake.body_parts_points() {
            let x_index = snake_point.x() as usize;
            let y_index = snake_point.y() as usize;
            tick_world_map[x_index][y_index] = PointType::Body;
        }
        let _ = self.terminal.render_matrix(&tick_world_map);
    }
}