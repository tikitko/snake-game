use super::point;
use super::point_node;

use std::ops::{Add, Sub};
use point::Point;
use point_node::PointNode;
use std::hash::Hash;

impl<N> PointNode<N> where N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash {
    fn recursive_move_chain_to(&mut self, point: Point<N>, add_node_to_end: bool) {
        let current_point = self.get_value();
        self.set_value(point);
        if let Some(next_node) = self.get_next_node_mut() {
            next_node.recursive_move_chain_to(current_point, add_node_to_end)
        } else if add_node_to_end {
            self.set_next_node(Some(Box::new(PointNode::new(current_point))))
        }
    }
    fn x(&self) -> N {
        self.get_value().x()
    }
    fn y(&self) -> N {
        self.get_value().y()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MoveDirection {
    Right,
    Left,
    Down,
    Up,
}

impl MoveDirection {
    pub fn reverse(&self) -> MoveDirection {
        match self {
            MoveDirection::Right => MoveDirection::Left,
            MoveDirection::Left => MoveDirection::Right,
            MoveDirection::Down => MoveDirection::Up,
            MoveDirection::Up => MoveDirection::Down,
        }
    }
}

pub struct Snake<N> where N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash {
    head_point_node: Box<PointNode<N>>,
    is_stomach_not_empty: bool,
}

impl<N> Snake<N> where N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash {
    pub fn make_on(point: Point<N>) -> Self {
        Snake { head_point_node: Box::new(PointNode::new(point)), is_stomach_not_empty: false }
    }
    pub fn body_parts_points(&self, include_head: bool) -> Vec<Point<N>> {
        if include_head {
            self.head_point_node.all_nodes_values()
        } else {
            match self.head_point_node.get_next_node() {
                Some(node) => node.all_nodes_values(),
                None => Vec::new(),
            }
        }
    }
    pub fn fill_stomach_if_empty(&mut self) {
        self.is_stomach_not_empty = true
    }
    pub fn head_point(&self) -> Point<N> {
        self.head_point_node.get_value()
    }
}

impl<N> Snake<N> where N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash + From<u8> {
    pub fn next_head_point(&self, move_direction: MoveDirection) -> Point<N> {
        let mut x = self.head_point_node.x();
        let mut y = self.head_point_node.y();
        let step_value = N::from(1);
        match move_direction {
            MoveDirection::Right => x = x.add(step_value),
            MoveDirection::Left => x = x.sub(step_value),
            MoveDirection::Down => y = y.add(step_value),
            MoveDirection::Up => y = y.sub(step_value)
        }
        Point::new(x, y)
    }
    pub fn move_to(&mut self, move_direction: MoveDirection) {
        let is_body_increased = self.is_stomach_not_empty;
        self.is_stomach_not_empty = false;
        let next_head_point = self.next_head_point(move_direction);
        self.head_point_node.recursive_move_chain_to(next_head_point, is_body_increased)
    }
}