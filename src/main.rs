#![allow(dead_code)]

extern crate crossterm;

mod base;
mod snake;
mod terminal;
mod terminal_snake_game_controller;

use crate::snake::game;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let game_controller = terminal_snake_game_controller::GameController::new();
    let game_config = game::Config {
        game_controller: Rc::new(RefCell::new(game_controller))
    };
    let mut game = game::Game::try_create(game_config).unwrap();
    game.start();
}