use crate::{
    hash,
    graph::*,
};
use std::hash::Hash;
use std::collections::{
    HashSet,
    HashMap,
};

impl<T: Hash + Eq> Graph<T> {
    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    pub fn pick(&self) -> Option<&T> {
        self.iter().next()
    }

    pub fn partition(self) -> Parts<T> {
        Parts::new(self)
    }
}

pub struct Parts<T> {
    graph: Graph<T>,
    disjoint: HashMap<u64, u64>,
}

impl<T: Hash + Eq> Parts<T> {
    pub fn new(graph: Graph<T>) -> Self {
        let mut disjoint = HashMap::new();
        
        for start in graph.iter() {
            let start_key = hash(start);
            if disjoint.contains_key(&start_key) {
                continue;
            }

            graph.dfs(start)
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
        let mut n = 0;
        for part in g.partition() {
            dbg!(&part);
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

        let mut parts = g.partition();
        while let Some(next) = parts.next() {
            dbg!(&next);
            if next.get(&'a').is_some() {
                assert_eq!(next.size(), 3);
            } else {
                assert_eq!(next.size(), 2);
            }
        }
    }
}