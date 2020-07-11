mod snake_game;
mod terminal;
mod snake;
mod node;
mod point_node;
mod point;

extern crate crossterm;

use crossterm::{Result, ErrorKind};
use crossterm::event::{read, Event, KeyCode, poll};
use crossterm::style::{Colorize};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::time::Duration;
use std::thread;


fn main() -> Result<()> {
    let mut snake_game = snake_game::SnakeGame::new((50, 20));
    enable_raw_mode();
    loop {
        let current_key_code = match current_key_code() {
            Ok(key_code) => key_code,
            Err(_) => KeyCode::Null,
        };
        snake_game.tick(current_key_code);
        thread::sleep(Duration::from_millis(100));
    }
    disable_raw_mode()
}

fn current_key_code() -> Result<KeyCode> {
    match poll(Duration::from_millis(0)) {
        Ok(is_success) => {
            if is_success {
                match read() {
                    Ok(event) => match event {
                        Event::Key(key_event) => Ok(key_event.code),
                        _ => Ok(KeyCode::Null)
                    },
                    Err(err) => Err(err)
                }
            } else {
                Ok(KeyCode::Null)
            }
        },
        Err(err) => return Err(err)
    }
}