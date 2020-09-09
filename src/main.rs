#![allow(dead_code)]

use crate::snake::game;

mod base;
mod snake;
mod terminal;
mod terminal_snake_game_config;

fn main() {
    match game::Game::try_create(game::Config::terminal()) {
        Ok(mut game) => game.start(),
        Err(e) => panic!(e),
    };
}