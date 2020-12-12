pub mod graph;
pub mod search;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

fn hash<T: Hash>(data: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
} 