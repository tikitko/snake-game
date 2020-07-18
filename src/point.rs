use std::ops::{Add, Sub};
use std::hash::{Hash, Hasher};

pub struct Point<N> where
    N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash {
    x: N,
    y: N,
}

impl<N> Point<N> where
    N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash {
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

impl<N> Copy for Point<N> where
    N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash {}

impl<N> Clone for Point<N> where
    N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash {
    fn clone(&self) -> Self {
        Point::new(self.x(), self.y())
    }
}

impl<N> PartialEq for Point<N> where
    N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl<N> Hash for Point<N> where
    N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl<N> Eq for Point<N> where
    N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash {}