#![allow(dead_code)]

mod base;
mod snake;
mod terminal;
mod terminal_snake_game_controller;

fn main() {
    start_snake_game();
}

fn start_snake_game() {
    let terminal_game_controller = terminal_snake_game_controller::GameController::new();
    let game_config = snake::game::Config {
        game_controller: std::rc::Rc::new(std::cell::RefCell::new(terminal_game_controller)),
    };
    match snake::game::Game::try_create(game_config) {
        Ok(mut game) => game.start(),
        Err(err) => println!("{:?}", err),
    }
}