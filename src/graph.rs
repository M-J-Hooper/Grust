use crate::hash;
use std::collections::{
    HashMap,
    HashSet,
};
use std::hash::Hash;

#[derive(Debug, Default)]
pub struct Graph<T> {
    nodes: HashMap<u64, Node<T>>,
}

impl<T: Default> Graph<T> {
    pub fn empty() -> Self {
        Default::default()
    }
}

impl<T: Hash + Eq + Default> Graph<T> {
    pub fn init<I: IntoIterator<Item=T>>(labels: I) -> Self {
        let mut graph = Self::empty();
        for label in labels {
            graph.add(label);
        }
        graph
    }
}

impl<T: Hash + Eq> Graph<T> {
    pub fn add(&mut self, label: T) {
        let key = hash(&label);

        let node = Node {
            label: label,
            edges: HashSet::new(),
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

    pub fn adjacent(&self, label: &T) -> Option<HashSet<&T>> {
        let key = hash(label);
        let res = self.nodes.get(&key)?
            .edges.iter()
            .map(|e| e.target)
            .map(|j| self.nodes.get(&j).unwrap())
            .map(|n| &n.label)
            .collect::<HashSet<_>>();
        
        Some(res)
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
    edges: HashSet<Edge>,
}

impl<T: Hash> Node<T> {
    pub fn connect_to(&mut self, to: &T) {
        let target = hash(to);
        self.edges.insert(Edge { 
            target,
            weight: 1,
        });
    }
    
    pub fn disconnect_from(&mut self, from: &T) {
        let target = hash(from);
        self.edges.remove(&Edge { 
            target,
            weight: 1,
        });
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Edge {
    target: u64,
    weight: i64,
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

        assert!(g.adjacent(&'a').unwrap().contains(&&'b'));
        assert!(g.adjacent(&'a').unwrap().contains(&&'c'));
        assert!(g.adjacent(&'b').unwrap().contains(&&'a'));
        assert!(g.adjacent(&'c').unwrap().contains(&&'a'));
        
        assert!(g.adjacent(&'d').is_none());

        // b <-> a <- c
        assert!(g.disconnect(&'a', &'c'));
        assert!(!g.adjacent(&'a').unwrap().contains(&&'c'));
        assert!(g.adjacent(&'c').unwrap().contains(&&'a'));

        // b <-x-> c
        assert!(g.remove(&'a').is_some());
        assert!(g.adjacent(&'a').is_none());
        assert!(g.adjacent(&'b').unwrap().is_empty());
        assert!(g.adjacent(&'c').unwrap().is_empty());
    }
}