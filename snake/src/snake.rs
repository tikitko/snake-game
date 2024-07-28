use super::components::direction::Direction;
use super::components::node::Node;
use super::components::point::Point;

use std::hash::Hash;
use std::ops::{Add, Sub};

pub struct Snake<N>
where
    N: Add<Output = N> + Sub<Output = N> + Copy + Eq + Hash,
{
    head_point_node: Box<Node<Point<N>>>,
    is_stomach_not_empty: bool,
}

impl<N> Snake<N>
where
    N: Add<Output = N> + Sub<Output = N> + Copy + Eq + Hash,
{
    pub fn new(point: Point<N>) -> Self {
        Self {
            head_point_node: Box::new(Node::new(point)),
            is_stomach_not_empty: false,
        }
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
        self.is_stomach_not_empty = true;
    }
    pub fn head_point(&self) -> Point<N> {
        self.head_point_node.get_value()
    }
}

impl<N> Snake<N>
where
    N: Add<Output = N> + Sub<Output = N> + Copy + Eq + Hash + From<u8>,
{
    pub fn next_head_point(&self, move_direction: Direction) -> Point<N> {
        let head_point = self.head_point_node.get_value();
        let mut x = head_point.x();
        let mut y = head_point.y();
        let step_value = N::from(1);
        match move_direction {
            Direction::Right => x = x.add(step_value),
            Direction::Left => x = x.sub(step_value),
            Direction::Down => y = y.add(step_value),
            Direction::Up => y = y.sub(step_value),
        }
        Point::new(x, y)
    }
    pub fn move_to(&mut self, move_direction: Direction) {
        let is_body_increased = self.is_stomach_not_empty;
        self.is_stomach_not_empty = false;
        let next_head_point = self.next_head_point(move_direction);
        self.recursive_move_body_to(next_head_point, is_body_increased);
    }
    fn recursive_move_body_to(&mut self, point: Point<N>, add_body_to_end: bool) {
        let mut next_point: Option<Point<N>> = Some(point);
        self.head_point_node.recursive_run(|node| match next_point {
            Some(point) => {
                let current_point = node.get_value();
                node.set_value(point);
                match node.get_next_node() {
                    Some(_) => next_point = Some(current_point),
                    None => {
                        if add_body_to_end {
                            node.set_next_node(Some(Node::new(current_point)));
                            next_point = None;
                        } else {
                            next_point = Some(current_point);
                        }
                    }
                }
            }
            None => {}
        });
    }
    pub fn recursive_remove_tail<F>(&mut self, should_remove: F)
    where
        F: Fn(Point<N>) -> bool,
    {
        self.head_point_node
            .recursive_run(|node| match node.get_next_node_mut() {
                Some(next_node) => {
                    if should_remove(next_node.get_value()) {
                        node.set_next_node(None);
                    }
                }
                None => {}
            });
    }
}
