use std::ops::{Add, Sub};

pub struct Point<N> where N: Add<Output = N> + Sub<Output = N> + Copy + PartialEq {
    x: N,
    y: N
}

impl<N> Point<N> where N: Add<Output = N> + Sub<Output = N> + Copy + PartialEq {
    pub fn new(x: N, y: N) -> Self {
        Point { x, y }
    }
    pub fn x(&self) -> N {
        self.x
    }
    pub fn y(&self) -> N {
        self.y
    }
}

impl<N> Copy for Point<N> where N: Add<Output = N> + Sub<Output = N> + Copy + PartialEq {}

impl<N> Clone for Point<N> where N: Add<Output = N> + Sub<Output = N> + Copy + PartialEq {
    fn clone(&self) -> Self {
        Point::new(self.x(), self.y())
    }
}

impl<N> PartialEq for Point<N> where N: Add<Output = N> + Sub<Output = N> + Copy + PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}