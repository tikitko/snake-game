#![allow(dead_code)]

mod base;
mod snake;
mod terminal;
mod terminal_snake_game_config;

fn main() {
    match snake::game::Game::new(terminal_snake_game_config::new()) {
        Ok(mut game) => game.start(),
        Err(err) => println!("{:?}", err),
    }
}