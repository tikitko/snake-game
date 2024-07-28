extern crate components;

pub mod game;
pub mod snake;
pub mod world;

pub type Point<N> = components::point::Point<N>;
pub type Direction = components::direction::Direction;
pub type AreaSize = u16;
