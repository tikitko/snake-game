use std::hash::Hash;

pub struct Node<V> where
    V: Copy + Hash + Eq {
    value: V,
    next_node: Option<Box<Node<V>>>,
}

impl<V> Node<V> where
    V: Copy + Hash + Eq {
    pub fn new(value: V) -> Self {
        Self {
            value,
            next_node: None,
        }
    }
    pub fn get_value(&self) -> V {
        self.value
    }
    pub fn set_value(&mut self, value: V) {
        self.value = value;
    }
    pub fn get_next_node_mut(&mut self) -> Option<&mut Node<V>> {
        self.next_node.as_mut().map(|n| n.as_mut())
    }
    pub fn get_next_node(&self) -> Option<&Node<V>> {
        self.next_node.as_ref().map(|n| n.as_ref())
    }
    pub fn set_next_node(&mut self, next_node: Option<Node<V>>) {
        self.next_node = next_node.map(|n| Box::new(n));
    }
    pub fn all_nodes_values(&self) -> Vec<V> {
        let mut child_nodes_values = match self.next_node.as_ref() {
            Some(next_node) => next_node.all_nodes_values(),
            None => Vec::new(),
        };
        child_nodes_values.push(self.value);
        child_nodes_values
    }
    pub fn recursive_run<F>(&mut self, mut entrance: F) where
        F: FnMut(&mut Node<V>) {
        entrance(self);
        match self.get_next_node_mut() {
            Some(next_node) => next_node.recursive_run(entrance),
            None => {},
        }
    }
}