#![allow(dead_code)]

mod world;
mod snake_game;
mod terminal;
mod snake;
mod node;
mod point_node;
mod point;

extern crate crossterm;

fn main() {
    let snake_game_config = snake_game::SnakeGameConfig {
        two_players: true,
        world_size: (100, 100),
        eat_count: 3
    };
    let mut snake_game = snake_game::SnakeGame::try_create(snake_game_config).unwrap();
    snake_game.start().unwrap();
}
