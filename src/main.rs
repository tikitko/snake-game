#![allow(dead_code)]

extern crate crossterm;

mod base;
mod snake;
mod terminal;
mod terminal_snake_game_controller;

use crate::snake::game;

use std::os::raw::c_uint;
use std::time::{UNIX_EPOCH, SystemTime};

fn main() {
    match game::Game::try_create(game::Config::terminal()) {
        Ok(mut game) => game.start(),
        Err(e) => panic!(e),
    };
}

extern "C" {
    fn srand(seed: c_uint);
    fn rand() -> c_uint;
}

pub unsafe fn get_rand_in_range(a: i32, b: i32) -> i32 {
    let m = (b - a + 1) as u32;
    a + (rand() % m) as i32
}

pub unsafe fn set_rand_current_time_seed() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    srand(nanos);
}