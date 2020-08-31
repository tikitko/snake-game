use super::snake::Snake;
use super::base::point::Point;
use super::base::world::World as GenericWorld;
use super::base::direction::Direction;
use super::AreaSize;

use rand::Rng;
use rand::rngs::ThreadRng;
use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;
use std::hash::Hash;
use std::cell::{RefCell, RefMut};
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
        let controller = self.snakes_controllers.get(id);
        match controller {
            Some(controller) => Some(controller.as_ref().borrow_mut()),
            None => None,
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
    world: &'a GenericWorld<ObjectType, AreaSize>
}

impl<'a> WorldView<'a> {
    pub fn generate_map(&self) -> HashMap<Point<AreaSize>, ObjectType> {
        self.world.generate_map()
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
    world: GenericWorld<ObjectType, AreaSize>,
    snakes_info: HashMap<usize, SnakeInfo>,
    border_points: HashSet<Point<AreaSize>>,
    eat_points: HashSet<Point<AreaSize>>,
    config: Config,
    rng: ThreadRng,
}

impl World {
    pub fn try_create(config: Config) -> Result<Self, CreateError> {
        if config.world_size.0 < 10 || config.world_size.1 < 10 {
            return Err(CreateError::WorldSmall);
        }
        if config.world_size.0 > 100 || config.world_size.1 > 100 {
            return Err(CreateError::WorldLarge);
        }
        if config.eat_count < 1 {
            return Err(CreateError::FoodLack);
        }
        if config.eat_count > 10 {
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
        Ok(World {
            world: GenericWorld::new(),
            snakes_info: HashMap::new(),
            border_points: HashSet::new(),
            eat_points: HashSet::new(),
            config,
            rng: rand::thread_rng(),
        })
    }
    pub fn tick(&mut self, reset: bool) -> WorldView {
        // Border
        if reset {
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
            self.world.set_layer(ObjectType::Border, self.border_points.clone());
        }
        // Snakes
        if reset {
            let snakes: HashMap<usize, Snake<AreaSize>> = {
                let mut snakes = HashMap::new();
                for snake_number in 0..self.config.snakes_controllers.len() as AreaSize {
                    let real_snake_number = snake_number + 1;
                    let snake_number = snake_number as usize;
                    let mut snake = Snake::make_on(Point::new(3, real_snake_number * 3));
                    for _ in 0..self.config.base_snake_tail_size {
                        snake.fill_stomach_if_empty();
                        snake.move_to(Direction::Right)
                    }
                    snakes.insert(snake_number, snake);
                }
                snakes
            };
            // Snakes: snakes_spawn
            for (snake_number, snake) in snakes {
                if let Some(mut controller) = self.config.snake_controller(&snake_number) {
                    let world_view = WorldView { world: &self.world };
                    controller.snake_will_burn(&world_view);
                }
                let points = HashSet::from_iter(snake.body_parts_points(true).clone());
                let snake_info = SnakeInfo {
                    snake,
                    direction: None,
                };
                self.snakes_info.insert(snake_number, snake_info);
                self.world.set_layer(ObjectType::Snake(snake_number.clone()), points);
                if let Some(snake_info) = self.snakes_info.get(&snake_number) {
                    if let Some(mut controller) = self.config.snake_controller(&snake_number) {
                        let world_view = WorldView { world: &self.world };
                        controller.snake_did_burn(snake_info, &world_view);
                    }
                }
            }
        }
        let mut points_move_vectors = HashMap::<Point<AreaSize>, HashSet<Direction>>::new();
        // Snakes: snakes_move
        for (snake_number, snake_info) in &mut self.snakes_info {
            if let Some(mut controller) = self.config.snake_controller(&snake_number) {
                let world_view = WorldView { world: &self.world };
                let controller_direction = controller.snake_will_move(snake_info, &world_view);
                if let Some(snake_direction) = snake_info.direction {
                    let have_tail = snake_info.snake.body_parts_points(false).len() > 0;
                    if controller_direction.reverse() != snake_direction || !have_tail {
                        snake_info.direction = Some(controller_direction)
                    }
                } else {
                    snake_info.direction = Some(controller_direction);
                }
            }
            let direction = snake_info.direction;
            let head_point = snake_info.snake.head_point();
            if let Some(direction) = direction {
                if let Some(vector_directions) = points_move_vectors.get_mut(&head_point) {
                    vector_directions.insert(direction);
                } else {
                    let mut vector_directions = HashSet::new();
                    vector_directions.insert(direction);
                    points_move_vectors.insert(head_point.clone(), vector_directions);
                }
            }
            if let Some(direction) = direction {
                snake_info.snake.move_to(direction);
            }
            let points = HashSet::from_iter(snake_info.snake.body_parts_points(true).clone());
            self.world.set_layer(ObjectType::Snake(snake_number.clone()), points);
            if let Some(mut controller) = self.config.snake_controller(&snake_number) {
                let world_view = WorldView { world: &self.world };
                controller.snake_did_move(snake_info, &world_view);
            }
        }
        let mut snakes_to_remove = HashSet::<usize>::new();
        let mut snakes_that_ate_food = HashMap::<usize, Point<AreaSize>>::new();
        let mut snakes_that_bit_tail = HashMap::<usize, (usize, Point<AreaSize>)>::new();
        // Snakes: snakes_interactions_detect
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
                for object in self.world.point_occurrences(&body_point) {
                    match object {
                        ObjectType::Snake(number) => if number != *snake_number {
                            if self.config.cut_tails {
                                if let Some(other_snake_info) = self.snakes_info.get(&number) {
                                    if other_snake_info.snake.head_point() == head_point {
                                        snakes_to_remove.insert(snake_number.clone());
                                        continue;
                                    }
                                }
                                if body_point == head_point {
                                    let tail_info = (number.clone(), body_point.clone());
                                    snakes_that_bit_tail.insert(snake_number.clone(), tail_info);
                                }
                            } else {
                                snakes_to_remove.insert(snake_number.clone());
                            }
                        },
                        ObjectType::Eat => if body_point == head_point {
                            snakes_that_ate_food.insert(snake_number.clone(), body_point.clone());
                        } else {
                            snakes_to_remove.insert(snake_number.clone());
                        },
                        ObjectType::Border => {
                            snakes_to_remove.insert(snake_number.clone());
                        }
                    }
                }
            }
        }
        // Snakes: snakes_remove
        for snake_remove_number in snakes_to_remove {
            if let Some(to_remove_snake_info) = self.snakes_info.get(&snake_remove_number) {
                if let Some(mut controller) = self.config.snake_controller(&snake_remove_number) {
                    let world_view = WorldView { world: &self.world };
                    controller.snake_will_died(&to_remove_snake_info, &world_view);
                }
            }
            self.snakes_info.remove(&snake_remove_number);
            self.world.remove_layer(&ObjectType::Snake(snake_remove_number));
            if let Some(mut controller) = self.config.snake_controller(&snake_remove_number) {
                let world_view = WorldView { world: &self.world };
                controller.snake_did_died(&world_view);
            }
        }
        // Snakes: snakes_bit_tail
        for (snake, (cut_snake, body_point)) in snakes_that_bit_tail {
            if let Some(snake_info) = self.snakes_info.get_mut(&snake) {
                if let Some(mut controller) = self.config.snake_controller(&snake) {
                    let world_view = WorldView { world: &self.world };
                    controller.snake_will_eat(false, &snake_info, &world_view);
                }
                snake_info.snake.fill_stomach_if_empty();
            }
            if let Some(cut_snake_info) = self.snakes_info.get_mut(&cut_snake) {
                cut_snake_info.snake.remove_tail(|p| p == body_point);
                let body_points = cut_snake_info.snake.body_parts_points(true).clone();
                let points = HashSet::from_iter(body_points);
                self.world.set_layer(ObjectType::Snake(cut_snake.clone()), points);
            }
            if let Some(snake_info) = self.snakes_info.get(&snake) {
                if let Some(mut controller) = self.config.snake_controller(&snake) {
                    let world_view = WorldView { world: &self.world };
                    controller.snake_did_eat(false, &snake_info, &world_view);
                }
            }
        }
        // Snakes: snakes_feeding
        for (snakes_feeding, eat_point) in snakes_that_ate_food {
            if let Some(snake_info) = self.snakes_info.get_mut(&snakes_feeding) {
                if let Some(mut controller) = self.config.snake_controller(&snakes_feeding) {
                    let world_view = WorldView { world: &self.world };
                    controller.snake_will_eat(true, &snake_info, &world_view);
                }
                snake_info.snake.fill_stomach_if_empty();
                if self.eat_points.remove(&eat_point) {
                    self.world.set_layer(ObjectType::Eat, self.eat_points.clone());
                }
                if let Some(mut controller) = self.config.snake_controller(&snakes_feeding) {
                    let world_view = WorldView { world: &self.world };
                    controller.snake_did_eat(true, &snake_info, &world_view);
                }
            }
        }
        // Eat
        let eat_to_spawn = self.config.eat_count - self.eat_points.len() as AreaSize;
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
        self.world.set_layer(ObjectType::Eat, self.eat_points.clone());
        WorldView { world: &self.world }
    }
}