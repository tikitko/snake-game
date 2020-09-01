use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::{Add, Sub};
use crate::base::point::Point;

#[derive(Clone)]
pub struct World<L, N> where
    L: Hash + Eq,
    N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash {
    layers: HashMap<L, HashSet<Point<N>>>
}

impl<L, N> World<L, N> where
    L: Hash + Eq + Copy,
    N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash {
    pub fn new() -> Self {
        World { layers: HashMap::new() }
    }
    pub fn set_layer(&mut self, layer_key: L, layer: HashSet<Point<N>>) {
        self.layers.insert(layer_key, layer);
    }
    pub fn remove_layer(&mut self, layer_key: &L) {
        self.layers.remove(layer_key);
    }
    pub fn remove_all_layers(&mut self) {
        self.layers = HashMap::new()
    }
    pub fn point_occurrences(&self, point: &Point<N>) -> HashSet<L> {
        let mut occurrences = HashSet::new();
        for (layer_key, layer) in &self.layers {
            if layer.contains(point) {
                let _ = occurrences.insert(*layer_key);
            }
        }
        occurrences
    }
    pub fn generate_map(&self) -> HashMap<Point<N>, L> {
        let mut map = HashMap::new();
        for (layer_key, layer) in &self.layers {
            for point in layer {
                map.insert(point.clone(), layer_key.clone());
            }
        }
        map
    }
}