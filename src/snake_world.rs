use super::snake;
use super::point;
use super::world;
use super::direction;
use snake::Snake;
use point::Point;
use world::World;
use direction::Direction;

use rand::Rng;
use rand::rngs::ThreadRng;
use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;
use std::hash::Hash;
use std::cell::{RefCell, RefMut};
use std::borrow::{BorrowMut, Borrow};
use std::rc::Rc;
use rand::distributions::Open01;

type NumberSize = u16;
type Config = SnakeWorldConfig;
type CreateError = SnakeWorldCreateError;
type ObjectType = SnakeWorldObjectType;
type SnakeInfo = SnakeWorldSnakeInfo;
type WorldView<'a>  = SnakeWorldWorldView<'a> ;
type SnakeController = dyn SnakeWorldSnakeController;

pub struct SnakeWorldConfig {
    pub world_size: (NumberSize, NumberSize),
    pub eat_count: NumberSize,
    pub snakes_controllers: HashMap<usize, Rc<RefCell<SnakeController>>>
}
impl SnakeWorldConfig {
    fn snake_controller(&self, id: &usize) -> Option<RefMut<SnakeController>> {
        let controller = self.snakes_controllers.get(id);
        match controller {
            Some(v) => {
                let c = v.as_ref();
                Some(c.borrow_mut())
            },
            None => None
        }
        //controller.map(|v| (*v).as_ref().get_mut())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SnakeWorldCreateError {
    WorldSmall,
    WorldLarge,
    FoodLack,
    FoodExcess,
    TooFewControllers,
    TooManyControllers,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SnakeWorldObjectType {
    Border,
    Snake(usize),
    Eat,
}

pub struct SnakeWorldSnakeInfo {
    snake: Snake<NumberSize>,
    direction: Option<Direction>,
}

pub struct SnakeWorldWorldView<'a> {
    world: &'a World<ObjectType, NumberSize>
}

pub trait SnakeWorldSnakeController {
    fn snake_will_burn(&mut self, world_view: &WorldView);
    fn snake_did_burn(&mut self, self_info: &SnakeInfo, world_view: &WorldView);
    fn snake_will_move(&mut self, self_info: &SnakeInfo, world_view: &WorldView) -> Direction;
    fn snake_did_move(&mut self, self_info: &SnakeInfo, world_view: &WorldView);
    fn snake_will_died(&mut self, self_info: &SnakeInfo, world_view: &WorldView);
    fn snake_did_died(&mut self, world_view: &WorldView);
}

pub struct SnakeWorld {
    world: World<ObjectType, NumberSize>,
    snakes_info: HashMap<usize, SnakeInfo>,
    border_points: HashSet<Point<NumberSize>>,
    eat_points: HashSet<Point<NumberSize>>,
    config: Config,
    rng: ThreadRng,
}

impl SnakeWorld {
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
        if config.world_size.1 <= ((config.snakes_controllers.len() + 1) * 3) as NumberSize {
            return Err(CreateError::TooManyControllers);
        }
        Ok(SnakeWorld {
            world: World::new(),
            snakes_info: HashMap::new(),
            border_points: HashSet::new(),
            eat_points: HashSet::new(),
            config,
            rng: rand::thread_rng(),
        })
    }
    pub fn tick(&mut self, reset: bool) {
        // Border
        if reset {
            let border_points: HashSet<Point<NumberSize>> = {
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
            let snakes: HashMap<usize, Snake<NumberSize>> = {
                let mut snakes = HashMap::new();
                for snake_number in 0..self.config.snakes_controllers.len() as NumberSize {
                    let real_snake_number = snake_number + 1;
                    let snake_number = snake_number as usize;
                    let mut snake = Snake::make_on(Point::new(3, real_snake_number * 3));
                    for _ in 0..3 {
                        snake.fill_stomach_if_empty();
                        snake.move_to(Direction::Right)
                    }
                    snakes.insert(snake_number, snake);
                }
                snakes
            };
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
        let mut snakes_move_vectors = HashMap::new();
        for (snake_number, snake_info) in &mut self.snakes_info {
            if let Some(mut controller) = self.config.snake_controller(&snake_number) {
                let world_view = WorldView { world: &self.world };
                let controller_direction = controller.snake_will_move(snake_info, &world_view);
                if let Some(snake_direction) = snake_info.direction {
                    if controller_direction.reverse() != snake_direction {
                        snake_info.direction = Some(controller_direction)
                    }
                } else {
                    snake_info.direction = Some(controller_direction);
                }
            }
            let direction = snake_info.direction;
            let head_point = snake_info.snake.head_point();
            snakes_move_vectors.insert(snake_number.clone(), (direction, head_point));
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
        let mut snakes_numbers_to_remove = HashSet::new();
        'interactions: for (snake_number, snake_info) in &mut self.snakes_info {
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
                        continue 'interactions;
                    }
                }
            }
            let mut head_points_catch = false;
            for body_point in body_points {
                if head_point == body_point {
                    if head_points_catch {
                        snakes_numbers_to_remove.insert(snake_number.clone());
                        continue 'interactions;
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
                            continue 'interactions;
                        },
                        ObjectType::Eat => {
                            snake_info.snake.fill_stomach_if_empty();
                            self.eat_points.remove(&body_point);
                        }
                        ObjectType::Border => {
                            snakes_numbers_to_remove.insert(snake_number.clone());
                            continue 'interactions;
                        }
                    }
                }
            }
        }
        for snake_remove_number in snakes_numbers_to_remove {
            if let Some(removed_snake_info) = self.snakes_info.remove(&snake_remove_number) {
                if let Some(mut controller) = self.config.snake_controller(&snake_remove_number) {
                    let world_view = WorldView { world: &self.world };
                    controller.snake_will_died(&removed_snake_info, &world_view);
                }
                self.world.remove_layer(&ObjectType::Snake(snake_remove_number));
                if let Some(mut controller) = self.config.snake_controller(&snake_remove_number) {
                    let world_view = WorldView { world: &self.world };
                    controller.snake_did_died(&world_view);
                }
            }
        }
        // Eat
        let eat_to_spawn = self.config.eat_count - self.eat_points.len() as NumberSize;
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
    pub fn generate_map(&self) -> HashMap<Point<NumberSize>, ObjectType> {
        self.world.generate_map()
    }
    pub fn get_world_view(&self) -> WorldView {
         WorldView { world: &self.world }
    }
}