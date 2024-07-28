#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Direction {
    Right,
    Left,
    Down,
    Up,
}

impl Direction {
    pub fn reverse(&self) -> Self {
        match self {
            Self::Right => Self::Left,
            Self::Left => Self::Right,
            Self::Down => Self::Up,
            Self::Up => Self::Down,
        }
    }
}
