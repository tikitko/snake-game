extern crate snake;
extern crate terminal;

mod game_config;

fn main() {
    match snake::game::Game::new(game_config::new()) {
        Ok(mut game) => game.start(),
        Err(err) => println!("{:?}", err),
    }
}
