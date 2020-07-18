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
        world_size: (20, 20),
        eat_count: 3
    };
    /// TODO: Refactor
    match snake_game::SnakeGame::try_create(snake_game_config) {
        Ok(mut snake_game) => match snake_game.start() {
            Ok(_) => {},
            Err(err) => panic!(err),
        },
        Err(err) => panic!(err),
    }
}
