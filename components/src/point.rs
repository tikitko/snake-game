use std::hash::Hash;
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Point<N>
where
    N: Add<Output = N> + Sub<Output = N> + Copy + Eq + Hash,
{
    x: N,
    y: N,
}

impl<N> Point<N>
where
    N: Add<Output = N> + Sub<Output = N> + Copy + Eq + Hash,
{
    pub fn new(x: N, y: N) -> Self {
        Self { x, y }
    }
    pub fn x(&self) -> N {
        self.x
    }
    pub fn y(&self) -> N {
        self.y
    }
}
