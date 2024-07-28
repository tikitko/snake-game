use super::components::direction::Direction;
use super::components::point::Point;
use super::components::world::World as GenericWorld;
use super::components::{get_rand_in_range, set_rand_current_time_seed};
use super::snake::Snake;
use super::AreaSize;

use std::cell::{RefCell, RefMut};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::FromIterator;
use std::rc::Rc;

pub struct Config {
    pub world_size: (AreaSize, AreaSize),
    pub eat_count: AreaSize,
    pub cut_tails: bool,
    pub base_snake_tail_size: usize,
    pub snakes_controllers: HashMap<usize, Rc<RefCell<dyn SnakeController>>>,
}

impl Config {
    fn snake_controller(&self, id: &usize) -> Option<RefMut<dyn SnakeController>> {
        let controller = self.snakes_controllers.get(id)?;
        match controller.try_borrow_mut() {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum CreateError {
    WorldSmall,
    WorldLarge,
    FoodLack,
    FoodExcess,
    TooFewControllers,
    TooManyControllers,
    TooLargeSnakeTail,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ObjectType {
    Border,
    Snake(usize),
    Eat,
}

pub struct SnakeInfo {
    snake: Snake<AreaSize>,
    direction: Option<Direction>,
}

impl SnakeInfo {
    pub fn get_snake(&self) -> &Snake<AreaSize> {
        &self.snake
    }
    pub fn get_direction(&self) -> &Option<Direction> {
        &self.direction
    }
}

pub struct WorldView<'a> {
    world_mask: &'a GenericWorld<ObjectType, AreaSize>,
    snakes_info: &'a HashMap<usize, SnakeInfo>,
    border_points: &'a HashSet<Point<AreaSize>>,
    eat_points: &'a HashSet<Point<AreaSize>>,
}

impl<'a> WorldView<'a> {
    fn new(world: &'a World) -> Self {
        Self {
            world_mask: &world.world_mask,
            snakes_info: &world.snakes_info,
            border_points: &world.border_points,
            eat_points: &world.eat_points,
        }
    }
    pub fn get_world_mask(&self) -> &'a GenericWorld<ObjectType, AreaSize> {
        self.world_mask
    }
    pub fn get_snakes_info(&self) -> &'a HashMap<usize, SnakeInfo> {
        self.snakes_info
    }
    pub fn get_border_points(&self) -> &'a HashSet<Point<AreaSize>> {
        self.border_points
    }
    pub fn get_eat_points(&self) -> &'a HashSet<Point<AreaSize>> {
        self.eat_points
    }
}

pub trait SnakeController {
    fn snake_will_burn(&mut self, world_view: &WorldView);
    fn snake_did_burn(&mut self, self_info: &SnakeInfo, world_view: &WorldView);
    fn snake_will_move(&mut self, self_info: &SnakeInfo, world_view: &WorldView) -> Direction;
    fn snake_did_move(&mut self, self_info: &SnakeInfo, world_view: &WorldView);
    fn snake_will_eat(&mut self, good_eat: bool, self_info: &SnakeInfo, world_view: &WorldView);
    fn snake_did_eat(&mut self, good_eat: bool, self_info: &SnakeInfo, world_view: &WorldView);
    fn snake_will_died(&mut self, self_info: &SnakeInfo, world_view: &WorldView);
    fn snake_did_died(&mut self, world_view: &WorldView);
}

pub struct World {
    world_mask: GenericWorld<ObjectType, AreaSize>,
    snakes_info: HashMap<usize, SnakeInfo>,
    border_points: HashSet<Point<AreaSize>>,
    eat_points: HashSet<Point<AreaSize>>,
    config: Config,
}

struct SnakesInteractionsDetectResult {
    snakes_to_remove: HashSet<usize>,
    snakes_that_ate_food: HashMap<usize, Point<AreaSize>>,
    snakes_that_bit_tail: HashMap<usize, (usize, Point<AreaSize>)>,
}

impl World {
    pub fn new(config: Config) -> Result<Self, CreateError> {
        if config.world_size.0 < 10 || config.world_size.1 < 10 {
            return Err(CreateError::WorldSmall);
        }
        if config.world_size.0 > 1000 || config.world_size.1 > 1000 {
            return Err(CreateError::WorldLarge);
        }
        if config.eat_count < 1 {
            return Err(CreateError::FoodLack);
        }
        if config.eat_count > 100 {
            return Err(CreateError::FoodExcess);
        }
        if config.snakes_controllers.len() < 1 {
            return Err(CreateError::TooFewControllers);
        }
        if config.world_size.1 <= ((config.snakes_controllers.len() + 1) * 3) as AreaSize {
            return Err(CreateError::TooManyControllers);
        }
        if (4 + config.base_snake_tail_size + 1 + 4) as AreaSize > config.world_size.0 {
            return Err(CreateError::TooLargeSnakeTail);
        }
        Ok(Self {
            world_mask: GenericWorld::new(),
            snakes_info: HashMap::new(),
            border_points: HashSet::new(),
            eat_points: HashSet::new(),
            config,
        })
    }
    fn spawn_border(&mut self) {
        let border_points: HashSet<Point<AreaSize>> = {
            let mut border_points = HashSet::new();
            for x in 0..self.config.world_size.0 {
                for y in 0..self.config.world_size.1 {
                    let max_x = self.config.world_size.0 - 1;
                    let max_y = self.config.world_size.1 - 1;
                    if x == 0 || y == 0 || x == max_x || y == max_y {
                        border_points.insert(Point::new(x, y));
                    }
                }
            }
            border_points
        };
        self.border_points = border_points;
        self.world_mask
            .set_layer(ObjectType::Border, self.border_points.clone());
    }
    fn spawn_snakes(&mut self) {
        let snakes: HashMap<usize, Snake<AreaSize>> = {
            let mut snakes = HashMap::new();
            for snake_number in 0..self.config.snakes_controllers.len() as AreaSize {
                let real_snake_number = snake_number + 1;
                let snake_number = snake_number as usize;
                let mut snake = Snake::new(Point::new(3, real_snake_number * 3));
                for _ in 0..self.config.base_snake_tail_size {
                    snake.fill_stomach_if_empty();
                    snake.move_to(Direction::Right);
                }
                snakes.insert(snake_number, snake);
            }
            snakes
        };
        for (snake_number, snake) in snakes {
            if let Some(mut controller) = self.config.snake_controller(&snake_number) {
                let world_view = WorldView::new(self);
                controller.snake_will_burn(&world_view);
            }
            let points = HashSet::from_iter(snake.body_parts_points(true).clone());
            let snake_info = SnakeInfo {
                snake,
                direction: None,
            };
            self.snakes_info.insert(snake_number, snake_info);
            self.world_mask
                .set_layer(ObjectType::Snake(snake_number.clone()), points);
            if let Some(snake_info) = self.snakes_info.get(&snake_number) {
                if let Some(mut controller) = self.config.snake_controller(&snake_number) {
                    let world_view = WorldView::new(self);
                    controller.snake_did_burn(snake_info, &world_view);
                }
            }
        }
    }
    fn snakes_move(&mut self) -> HashMap<Point<AreaSize>, HashSet<Direction>> {
        let mut points_move_vectors = HashMap::<Point<AreaSize>, HashSet<Direction>>::new();
        let snakes_numbers = {
            let mut snakes_numbers = Vec::<usize>::new();
            for (key, snake_number) in self.snakes_info.keys().enumerate() {
                snakes_numbers.insert(key, snake_number.clone());
            }
            snakes_numbers
        };
        for snake_number in snakes_numbers {
            let mut new_direction: Option<Direction> = None;
            if let Some(snake_info) = self.snakes_info.get(&snake_number) {
                new_direction = snake_info.direction;
                if let Some(mut controller) = self.config.snake_controller(&snake_number) {
                    let world_view = WorldView::new(self);
                    let controller_direction = controller.snake_will_move(snake_info, &world_view);
                    if let Some(snake_direction) = snake_info.direction {
                        let have_tail = snake_info.snake.body_parts_points(false).len() > 0;
                        if controller_direction.reverse() != snake_direction || !have_tail {
                            new_direction = Some(controller_direction);
                        }
                    } else {
                        new_direction = Some(controller_direction);
                    }
                }
                if let Some(direction) = new_direction {
                    let head_point = snake_info.snake.head_point();
                    if let Some(vector_directions) = points_move_vectors.get_mut(&head_point) {
                        vector_directions.insert(direction);
                    } else {
                        let mut vector_directions = HashSet::new();
                        vector_directions.insert(direction);
                        points_move_vectors.insert(head_point.clone(), vector_directions);
                    }
                }
            }
            if let Some(snake_info) = self.snakes_info.get_mut(&snake_number) {
                snake_info.direction = new_direction;
                if let Some(direction) = new_direction {
                    snake_info.snake.move_to(direction);
                }
            }
            if let Some(snake_info) = self.snakes_info.get(&snake_number) {
                let points = HashSet::from_iter(snake_info.snake.body_parts_points(true).clone());
                self.world_mask
                    .set_layer(ObjectType::Snake(snake_number.clone()), points);
                if let Some(mut controller) = self.config.snake_controller(&snake_number) {
                    let world_view = WorldView::new(self);
                    controller.snake_did_move(snake_info, &world_view);
                }
            }
        }
        points_move_vectors
    }
    fn snakes_interactions_detect(
        &mut self,
        points_move_vectors: &HashMap<Point<AreaSize>, HashSet<Direction>>,
    ) -> SnakesInteractionsDetectResult {
        let mut snakes_to_remove = HashSet::<usize>::new();
        let mut snakes_that_ate_food = HashMap::<usize, Point<AreaSize>>::new();
        let mut snakes_that_bit_tail = HashMap::<usize, (usize, Point<AreaSize>)>::new();
        for (snake_number, snake_info) in &self.snakes_info {
            let body_points = snake_info.snake.body_parts_points(true);
            let head_point = snake_info.snake.head_point();
            if let Some(vector_directions) = points_move_vectors.get(&head_point) {
                for vector_direction in vector_directions {
                    let vector_reversed_direction = vector_direction.reverse();
                    if Some(vector_reversed_direction) == snake_info.direction {
                        snakes_to_remove.insert(snake_number.clone());
                    }
                }
            }
            let mut head_points_catch = false;
            for body_point in body_points {
                if head_point == body_point {
                    if head_points_catch {
                        if self.config.cut_tails {
                            let tail_info = (snake_number.clone(), body_point.clone());
                            snakes_that_bit_tail.insert(snake_number.clone(), tail_info);
                        } else {
                            snakes_to_remove.insert(snake_number.clone());
                        }
                    } else {
                        head_points_catch = true;
                    }
                }
                for object in self.world_mask.point_occurrences(&body_point) {
                    match object {
                        ObjectType::Snake(number) => {
                            if number != *snake_number {
                                if self.config.cut_tails {
                                    if let Some(other_snake_info) = self.snakes_info.get(&number) {
                                        if other_snake_info.snake.head_point() == head_point {
                                            snakes_to_remove.insert(snake_number.clone());
                                            continue;
                                        }
                                    }
                                    if body_point == head_point {
                                        let tail_info = (number.clone(), body_point.clone());
                                        snakes_that_bit_tail
                                            .insert(snake_number.clone(), tail_info);
                                    }
                                } else {
                                    snakes_to_remove.insert(snake_number.clone());
                                }
                            }
                        }
                        ObjectType::Eat => {
                            if body_point == head_point {
                                snakes_that_ate_food
                                    .insert(snake_number.clone(), body_point.clone());
                            } else {
                                snakes_to_remove.insert(snake_number.clone());
                            }
                        }
                        ObjectType::Border => {
                            snakes_to_remove.insert(snake_number.clone());
                        }
                    }
                }
            }
        }
        SnakesInteractionsDetectResult {
            snakes_to_remove,
            snakes_that_ate_food,
            snakes_that_bit_tail,
        }
    }
    fn handle_snakes_to_remove(&mut self, snakes_to_remove: HashSet<usize>) {
        for snake_remove_number in snakes_to_remove {
            if let Some(to_remove_snake_info) = self.snakes_info.get(&snake_remove_number) {
                if let Some(mut controller) = self.config.snake_controller(&snake_remove_number) {
                    let world_view = WorldView::new(self);
                    controller.snake_will_died(&to_remove_snake_info, &world_view);
                }
            }
            self.snakes_info.remove(&snake_remove_number);
            self.world_mask
                .remove_layer(&ObjectType::Snake(snake_remove_number));
            if let Some(mut controller) = self.config.snake_controller(&snake_remove_number) {
                let world_view = WorldView::new(self);
                controller.snake_did_died(&world_view);
            }
        }
    }
    fn handle_snakes_that_bit_tail(
        &mut self,
        snakes_that_bit_tail: HashMap<usize, (usize, Point<AreaSize>)>,
    ) {
        for (snake, (cut_snake, body_point)) in snakes_that_bit_tail {
            if let Some(snake_info) = self.snakes_info.get(&snake) {
                if let Some(mut controller) = self.config.snake_controller(&snake) {
                    let world_view = WorldView::new(self);
                    controller.snake_will_eat(false, &snake_info, &world_view);
                }
            }
            if let Some(snake_info) = self.snakes_info.get_mut(&snake) {
                snake_info.snake.fill_stomach_if_empty();
            }
            if let Some(cut_snake_info) = self.snakes_info.get_mut(&cut_snake) {
                cut_snake_info
                    .snake
                    .recursive_remove_tail(|p| p == body_point);
                let body_points = cut_snake_info.snake.body_parts_points(true).clone();
                let points = HashSet::from_iter(body_points);
                self.world_mask
                    .set_layer(ObjectType::Snake(cut_snake.clone()), points);
            }
            if let Some(snake_info) = self.snakes_info.get(&snake) {
                if let Some(mut controller) = self.config.snake_controller(&snake) {
                    let world_view = WorldView::new(self);
                    controller.snake_did_eat(false, &snake_info, &world_view);
                }
            }
        }
    }
    fn handle_snakes_that_ate_food(
        &mut self,
        snakes_that_ate_food: HashMap<usize, Point<AreaSize>>,
    ) {
        for (snakes_feeding, eat_point) in snakes_that_ate_food {
            if let Some(snake_info) = self.snakes_info.get(&snakes_feeding) {
                if let Some(mut controller) = self.config.snake_controller(&snakes_feeding) {
                    let world_view = WorldView::new(self);
                    controller.snake_will_eat(true, &snake_info, &world_view);
                }
            }
            if let Some(snake_info) = self.snakes_info.get_mut(&snakes_feeding) {
                snake_info.snake.fill_stomach_if_empty();
                if self.eat_points.remove(&eat_point) {
                    self.world_mask
                        .set_layer(ObjectType::Eat, self.eat_points.clone());
                }
            }
            if let Some(snake_info) = self.snakes_info.get(&snakes_feeding) {
                if let Some(mut controller) = self.config.snake_controller(&snakes_feeding) {
                    let world_view = WorldView::new(self);
                    controller.snake_did_eat(true, &snake_info, &world_view);
                }
            }
        }
    }
    fn spawn_eat(&mut self) {
        let eat_to_spawn = self.config.eat_count - self.eat_points.len() as AreaSize;
        unsafe {
            let _ = set_rand_current_time_seed();
        }
        for _ in 0..eat_to_spawn {
            loop {
                let point: Point<AreaSize> = unsafe {
                    let x = get_rand_in_range(1, (self.config.world_size.0 - 1) as i32);
                    let y = get_rand_in_range(1, (self.config.world_size.1 - 1) as i32);
                    Point::new(x as u16, y as u16)
                };
                if self.world_mask.point_occurrences(&point).len() == 0 {
                    self.eat_points.insert(point);
                    break;
                }
            }
        }
        self.world_mask
            .set_layer(ObjectType::Eat, self.eat_points.clone());
    }
    pub fn tick(&mut self, reset: bool) -> WorldView {
        if reset {
            self.spawn_border();
            self.spawn_snakes()
        }
        let points_move_vectors = self.snakes_move();
        let SnakesInteractionsDetectResult {
            snakes_to_remove,
            snakes_that_ate_food,
            snakes_that_bit_tail,
        } = self.snakes_interactions_detect(&points_move_vectors);
        self.handle_snakes_to_remove(snakes_to_remove);
        self.handle_snakes_that_bit_tail(snakes_that_bit_tail);
        self.handle_snakes_that_ate_food(snakes_that_ate_food);
        self.spawn_eat();
        WorldView::new(self)
    }
}
