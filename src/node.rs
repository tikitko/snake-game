pub struct Node<V> where V: Copy {
    value: V,
    next_node: Option<Box<Node<V>>>
}

impl<V> Node<V> where V: Copy {
    pub fn new(value: V) -> Self {
        Node { value, next_node: None }
    }
    pub fn get_value(&self) -> V {
        self.value
    }
    pub fn set_value(&mut self, value: V) {
        self.value = value
    }
    pub fn get_next_node(&mut self) -> Option<&mut Box<Node<V>>> {
        self.next_node.as_mut()
    }
    pub fn set_next_node(&mut self, next_node: Option<Box<Node<V>>>) {
        self.next_node = next_node
    }
    pub fn all_nodes_values(&self) -> Vec<V> {
        match self.next_node.as_ref() {
            Some(next_node) => {
                let mut child_nodes_values = next_node.all_nodes_values();
                child_nodes_values.push(self.value);
                child_nodes_values
            },
            None => vec!(self.value)
        }
    }
}