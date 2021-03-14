use crate::{
    hash,
    graph::*,
    iter::Mode,
};
use std::hash::Hash;
use std::collections::{
    HashSet,
    HashMap,
};

impl<T: Hash + Eq> Graph<T> {
    pub fn size(&self) -> usize {
        self.nodes.keys().len()
    }

    pub fn pick(&self) -> Option<&T> {
        self.iter().next()
    }

    pub fn sinks(&self) -> HashSet<&T> {
        self.nodes.values()
            .filter(|n| n.neighbors.is_empty())
            .map(|n| &n.label)
            .collect()
    }

    pub fn outdegree(&self, label: &T) -> Option<usize> {
        self.get(label).map(|n| n.neighbors.len())
    }

    pub fn indegree(&self, label: &T) -> Option<usize> {
        self.indegrees().remove(label)
    }

    pub fn indegrees<'a>(&'a self) -> HashMap<&'a T, usize> {
        let mut degrees = HashMap::new();
        for node in self.nodes.values() {
            if !degrees.contains_key(&node.label) {
                degrees.insert(&node.label, 0);
            }
            for key in node.neighbors.iter() {
                let label = &self.nodes.get(key).unwrap().label;
                *degrees.entry(label).or_insert(0) += 1;
            }

        }
        degrees
    }

    pub fn sources(&self) -> HashSet<&T> {
        let indegrees = self.indegrees();
        self.iter()
            .filter(|l| *indegrees.get(*l).unwrap() == 0)
            .collect()
    }

    pub fn partition(self) -> Parts<T> {
        Parts::new(self)
    }
}

impl<T: Hash + Eq> Extend<T> for Graph<T> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        for label in iter {
            self.add(label);
        }
    }
}

impl<'a, T: Hash + Eq + 'a> Extend<(&'a T, &'a T)> for Graph<T> {
    fn extend<I: IntoIterator<Item=(&'a T, &'a T)>>(&mut self, iter: I) {
        for edge in iter {
            self.connect(edge.0, edge.1);
        }
    }
}

pub struct Parts<T> {
    graph: Graph<T>,
    disjoint: HashMap<u64, u64>,
}

impl<T: Hash + Eq> Parts<T> {
    pub fn new(graph: Graph<T>) -> Self {
        let mut disjoint = HashMap::new();
        
        for start in graph.sources() {
            let start_key = hash(start);
            graph.walk(start, Mode::Depth)
                .unwrap()
                .map(|l| hash(l))
                .for_each(|k| {
                    disjoint.insert(k, start_key);
                });
        }

        Parts {
            graph,
            disjoint
        }
    }
}

impl<T: Hash + Eq> Iterator for Parts<T> {
    type Item = Graph<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_key = hash(self.graph.pick()?);
        let parent_key = self.disjoint.get(&next_key).unwrap();
        let (part, rest): (HashSet<_>, HashSet<_>) = self.disjoint
            .iter()
            .partition(|kv| kv.1 == parent_key);
        
        let mut g = Graph::new();
        for key in part {
            let original = self.graph.nodes.remove(&key.0)?;
            g.add_node(original);
        }
        self.disjoint = rest.into_iter()
            .map(|kv| (*kv.0, *kv.1))
            .collect();

        Some(g)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn trivial_disconnected() {
        let g = Graph::init('a'..='z');

        assert_eq!(g.sources().len(), 26);
        assert_eq!(g.sinks().len(), 26);

        let mut n = 0;
        for part in g.partition() {
            assert_eq!(part.size(), 1);
            n += 1;
        }
        assert_eq!(n, 26);
    }

    #[test]
    fn trivial_connected() {
        let mut g = Graph::init('a'..='c');
        assert!(g.connect(&'a', &'b'));
        assert!(g.connect(&'a', &'c'));
        assert!(g.connect(&'b', &'c'));

        assert_eq!(g.sources().len(), 1);
        assert!(g.sources().contains(&'a'));
        
        assert_eq!(g.sinks().len(), 1);
        assert!(g.sinks().contains(&'c'));
        
        let orig = g.clone(); 
        let mut parts = g.partition();
        assert_eq!(orig, parts.next().unwrap());
        assert_eq!(parts.next(), None);
    }

    #[test]
    fn mix() {
        let mut g = Graph::init('a'..='e');
        assert!(g.connect(&'a', &'b'));
        assert!(g.connect(&'a', &'c'));
        assert!(g.connect(&'b', &'c'));
        assert!(g.connect(&'d', &'e'));

        assert_eq!(g.sources().len(), 2);
        assert!(g.sources().contains(&'a'));
        assert!(g.sources().contains(&'d'));

        assert_eq!(g.sinks().len(), 2);
        assert!(g.sinks().contains(&'c'));
        assert!(g.sinks().contains(&'e'));

        let mut parts = g.partition();
        while let Some(next) = parts.next() {
            if next.get(&'a').is_some() {
                assert_eq!(next.size(), 3);
            } else {
                assert_eq!(next.size(), 2);
            }
        }
    }
}