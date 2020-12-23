use crate::{hash, iter::Walk};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq)]
pub struct Graph<T> {
    pub(crate) nodes: HashMap<u64, Node<T>>,
}

impl<T> Default for Graph<T> {
    fn default() -> Self {
        Graph {
            nodes: HashMap::new(),
        }
    }
}

impl<T> Graph<T> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T: Hash + Eq> Graph<T> {
    pub fn init<I: IntoIterator<Item = T>>(labels: I) -> Self {
        let mut graph = Self::new();
        for label in labels {
            graph.add(label);
        }
        graph
    }

    pub(crate) fn get(&self, label: &T) -> Option<&Node<T>> {
        let key = hash(label);
        self.nodes.get(&key)
    }

    pub(crate) fn add_node(&mut self, node: Node<T>) {
        let key = hash(&node.label);
        self.nodes.insert(key, node);
    }

    pub fn add(&mut self, label: T) {
        let node = Node {
            label,
            edges: HashMap::new(),
        };
        self.add_node(node);
    }

    pub fn remove(&mut self, label: &T) -> Option<Node<T>> {
        let key = hash(label);
        let node = self.nodes.remove(&key)?;

        for other in self.nodes.values_mut() {
            other.disconnect_from(label);
        }
        Some(node)
    }

    pub fn get_adjacent(&self, label: &T) -> Option<HashSet<&T>> {
        let res = self
            .get(label)?
            .edges
            .keys()
            .map(|k| self.nodes.get(k).unwrap())
            .map(|n| &n.label)
            .collect::<HashSet<_>>();

        Some(res)
    }

    pub fn is_adjacent(&self, from: &T, to: &T) -> bool {
        let node = self.get(from);
        node.is_some() && node.unwrap().is_adjacent_to(to)
    }

    pub fn connect(&mut self, from: &T, to: &T) -> bool {
        let a = hash(&from);
        let b = hash(&to);
        if a == b {
            return false; // Self-connection
        }

        let from_exists = self.nodes.contains_key(&a);
        let to_exists = self.nodes.contains_key(&b);
        if !from_exists || !to_exists {
            return false; // Node non-existent
        }

        if self.dfs(to).unwrap().any(|n| n == from) {
            false // Connection creates cycle
        } else {
            self.nodes.get_mut(&a).unwrap().connect_to(to);
            true
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node<T> {
    pub label: T,
    pub(crate) edges: HashMap<u64, i64>, // key is target, value is weight
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
        let mut g = Graph::init('a'..='d');

        // a
        // |\
        // b c
        // |
        // d
        assert!(g.connect(&'a', &'b'));
        assert!(g.connect(&'a', &'c'));
        assert!(g.connect(&'b', &'d'));

        assert!(g.get_adjacent(&'a').unwrap().contains(&&'b'));
        assert!(g.get_adjacent(&'a').unwrap().contains(&&'c'));
        assert!(g.get_adjacent(&'b').unwrap().contains(&&'d'));
        assert!(g.get_adjacent(&'c').unwrap().is_empty());
        assert!(g.get_adjacent(&'d').unwrap().is_empty());

        assert!(g.get_adjacent(&'e').is_none());

        assert!(g.disconnect(&'a', &'c'));
        assert!(!g.get_adjacent(&'a').unwrap().contains(&&'c'));

        assert!(g.remove(&'b').is_some());
        assert!(g.get_adjacent(&'b').is_none());
        assert!(g.get_adjacent(&'a').unwrap().is_empty());
        assert!(g.get_adjacent(&'c').unwrap().is_empty());
    }

    #[test]
    fn no_cycles() {
        let mut g = Graph::init('a'..='e');

        // a
        // |\
        // b c
        // |
        // d
        assert!(g.connect(&'a', &'b'));
        assert!(g.connect(&'a', &'c'));
        assert!(g.connect(&'b', &'d'));

        // Allowed
        assert!(g.connect(&'b', &'d'));
        assert!(g.connect(&'a', &'d'));
        assert!(g.connect(&'d', &'c'));

        // Not allowed
        assert!(!g.connect(&'a', &'a'));
        assert!(!g.connect(&'c', &'b'));
        assert!(!g.connect(&'d', &'a'));
        assert!(!g.connect(&'b', &'a'));
    }
}
