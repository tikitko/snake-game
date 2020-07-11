use super::node;
use super::point;
use node::Node;
use point::Point;
use std::ops::{Add, Sub};

pub type PointNode<N> where N: Add<Output = N> + Sub<Output = N> + Copy = Node<Point<N>>;