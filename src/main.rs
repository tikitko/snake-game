#![allow(dead_code)]

extern crate crossterm;

mod base;
mod snake;
mod terminal;
mod terminal_snake_game_controller;

use crate::snake::game;

fn main() {
    match game::Game::try_create(game::Config::terminal()) {
        Ok(mut game) => game.start(),
        Err(e) => panic!(e),
    };
}