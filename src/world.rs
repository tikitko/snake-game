use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::{Add, Sub};
use crate::point::Point;

struct World<L, N> where
    L: Hash + Eq,
    N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash
{
    layers: HashMap<L, HashSet<Point<N>>>
}

impl<L, N> World<L, N> where
    L: Hash + Eq + Copy,
    N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash
{
    fn set_layer(&mut self, layer_key: L, layer: HashSet<Point<N>>) {
        self.layers.insert(layer_key, layer);
    }
    fn point_occurrences(&self, point: &Point<N>) -> HashSet<L> {
        let mut occurrences = HashSet::new();
        for (layer_key, layer) in &self.layers {
            if layer.contains(point) {
                let _ = occurrences.insert(*layer_key);
            }
        }
        occurrences
    }
}