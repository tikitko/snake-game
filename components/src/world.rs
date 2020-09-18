use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::{Add, Sub};
use crate::point::Point;

#[derive(Clone)]
pub struct World<L, N> where
    L: Hash + Eq,
    N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash {
    layers: HashMap<L, HashSet<Point<N>>>,
}

impl<L, N> World<L, N> where
    L: Hash + Eq + Copy,
    N: Add<Output=N> + Sub<Output=N> + Copy + Eq + Hash {
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
        }
    }
    pub fn set_layer(&mut self, layer_key: L, layer: HashSet<Point<N>>) {
        self.layers.insert(layer_key, layer);
    }
    pub fn remove_layer(&mut self, layer_key: &L) {
        self.layers.remove(layer_key);
    }
    pub fn remove_all_layers(&mut self) {
        self.layers.clear();
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
    pub fn generate_map<PointM, ObjectM, PointR, ObjectR>(
        &self,
        point_mapper: PointM,
        object_mapper: ObjectM
    ) -> HashMap<PointR, ObjectR> where
        PointM: Fn(&Point<N>) -> PointR,
        ObjectM: Fn(&L) -> ObjectR,
        PointR: Eq + Hash {
        let mut map = HashMap::new();
        for (layer_key, layer) in &self.layers {
            for point in layer {
                map.insert(point_mapper(point), object_mapper(layer_key));
            }
        }
        map
    }
}