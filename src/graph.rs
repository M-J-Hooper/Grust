use crate::hash;
use std::collections::{
    HashMap,
    HashSet,
};
use std::hash::Hash;

#[derive(Debug)]
pub struct Graph<T> {
    nodes: HashMap<u64, Node<T>>,
}

impl<T> Default for Graph<T> {
    fn default() -> Self {
        Graph {
            nodes: HashMap::new()
        }
    }
}

impl<T> Graph<T> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T: Hash + Eq + Default> Graph<T> {
    pub fn init<I: IntoIterator<Item=T>>(labels: I) -> Self {
        let mut graph = Self::new();
        for label in labels {
            graph.add(label);
        }
        graph
    }
}

impl<T: Hash + Eq> Graph<T> {
    fn get(&self, label: &T) -> Option<&Node<T>> {
        let key = hash(label);
        self.nodes.get(&key)
    }

    pub fn add(&mut self, label: T) {
        let key = hash(&label);
        let node = Node {
            label: label,
            edges: HashMap::new(),
        };
        self.nodes.insert(key, node);
    }

    pub fn remove(&mut self, label: &T) -> Option<Node<T>> {
        let key = hash(label);
        let node = self.nodes.remove(&key)?;

        for other in self.nodes.values_mut() {
            other.disconnect_from(label);
        }
        Some(node)
    }

    pub fn connections(&self, label: &T) -> Option<HashSet<&T>> {
        let res = self.get(label)?
            .edges.keys()
            .map(|k| self.nodes.get(k).unwrap())
            .map(|n| &n.label)
            .collect::<HashSet<_>>();
        
        Some(res)
    }

    pub fn is_connected(&self, from: &T, to: &T) -> bool {
        let node = self.get(from);
        node.is_some() && node.unwrap().is_adjacent_to(to)
    }

    pub fn connect(&mut self, from: &T, to: &T) -> bool {
        let a = hash(&from);
        let b = hash(&to);
        let bb = self.nodes.contains_key(&b);
        let na = self.nodes.get_mut(&a);
        if bb && na.is_some() {
            na.unwrap().connect_to(to);
            true
        } else {
            false
        }
    }

    pub fn disconnect(&mut self, from: &T, to: &T) -> bool {
        let a = hash(&from);
        let b = hash(&to);
        let bb = self.nodes.contains_key(&b);
        let na = self.nodes.get_mut(&a);
        if bb && na.is_some() {
            na.unwrap().disconnect_from(to);
            true
        } else {
            false
        }
    }

    pub fn is_biconnected(&self, a: &T, b: &T) -> bool {
        self.is_connected(a, b) && self.is_connected(b, a)
    }

    pub fn biconnect(&mut self, a: &T, b: &T) -> bool {
        self.connect(a, b) && self.connect(b, a)
    }

    pub fn bidisconnect(&mut self, a: &T, b: &T) -> bool {
        self.disconnect(a, b) && self.disconnect(b, a)
    }
}

#[derive(Debug)]
pub struct Node<T> {
    pub label: T,
    edges: HashMap<u64, i64>, // key is target, value is weight
}

impl<T: Hash> Node<T> {
    pub fn is_adjacent_to(&self, to: &T) -> bool {
        let target = hash(to);
        self.edges.contains_key(&target)
    }

    pub fn connect_to(&mut self, to: &T) {
        let target = hash(to);
        self.edges.insert(target, 1);
    }
    
    pub fn disconnect_from(&mut self, from: &T) {
        let target = hash(from);
        self.edges.remove(&target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut g = Graph::init('a'..='c');
        
        // b <-> a <-> c
        assert!(g.biconnect(&'a', &'b'));
        assert!(g.biconnect(&'a', &'c'));
        assert!(!g.biconnect(&'a', &'d'));

        assert!(g.connections(&'a').unwrap().contains(&&'b'));
        assert!(g.connections(&'a').unwrap().contains(&&'c'));
        assert!(g.connections(&'b').unwrap().contains(&&'a'));
        assert!(g.connections(&'c').unwrap().contains(&&'a'));
        
        assert!(g.connections(&'d').is_none());

        // b <-> a <- c
        assert!(g.disconnect(&'a', &'c'));
        assert!(!g.connections(&'a').unwrap().contains(&&'c'));
        assert!(g.connections(&'c').unwrap().contains(&&'a'));

        // b <-x-> c
        assert!(g.remove(&'a').is_some());
        assert!(g.connections(&'a').is_none());
        assert!(g.connections(&'b').unwrap().is_empty());
        assert!(g.connections(&'c').unwrap().is_empty());
    }
}