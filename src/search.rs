use crate::{
    hash,
    graph::*,
};
use std::collections::{
    HashSet,
    VecDeque,
};
use std::hash::Hash;

pub enum Mode {
    Bredth,
    Depth
}

impl<T: Hash> Graph<T> {
    pub fn bfs<'a>(&'a self, start: &'a T) -> Iter<'a, T> {
        self.search(start, Mode::Bredth)
    }

    pub fn dfs<'a>(&'a self, start: &'a T) -> Iter<'a, T> {
        self.search(start, Mode::Depth)
    }

    pub fn search<'a>(&'a self, start: &'a T, mode: Mode) -> Iter<'a, T> {
        let mut buffer = VecDeque::new();
        buffer.push_front(start);

        let mut visited = HashSet::new();
        visited.insert(hash(start));
        Iter {
            mode,
            buffer,
            visited,
            graph: &self,
        }
    }
}

pub struct Iter<'a, T> {
    mode: Mode,
    graph: &'a Graph<T>,
    buffer: VecDeque<&'a T>,
    visited: HashSet<u64>, 
}

impl<'a, T: Hash + Eq> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = match self.mode {
            Mode::Bredth => self.buffer.pop_back()?,
            Mode::Depth => self.buffer.pop_front()?,
        };

        if let Some(connections) = self.graph.connections(next) {
            for connection in connections {
                let key = hash(connection);
                if !self.visited.contains(&key) {
                    self.visited.insert(key);
                    self.buffer.push_front(connection);
                }
            }
        }
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index<T: Eq>(v: &Vec<&T>, t: T) -> i8 {
        v.iter().position(|el| el == &&t).unwrap() as i8
    }

    fn assert_order(search: &Vec<&char>) {
        assert!(index(search, 'a') < index(search, 'b'));
        assert!(index(search, 'a') < index(search, 'd'));
        assert!(index(search, 'b') < index(search, 'c'));
        assert!(index(search, 'd') < index(search, 'e'));
        assert!(index(search, 'd') < index(search, 'f'));
    }

    #[test]
    fn tree_bfs_vs_dfs() {
        let mut g = Graph::init('a'..='f');

        // a -> b -> c
        assert!(g.connect(&'a', &'b'));
        assert!(g.connect(&'b', &'c'));

        // a -> d -> e
        //      d -> f
        assert!(g.connect(&'a', &'d'));
        assert!(g.connect(&'d', &'e'));
        assert!(g.connect(&'d', &'f'));
        
        let bredth = g.dfs(&'a').collect::<Vec<_>>();
        let depth = g.bfs(&'a').collect::<Vec<_>>();

        assert_order(&bredth);
        assert_eq!((index(&bredth, 'b') - index(&bredth, 'c')).abs(), 1); // c directly below b

        assert_order(&depth);
        assert_eq!((index(&depth, 'b') - index(&depth, 'd')).abs(), 1); // d directly beside b
    } 
    
    #[test]
    fn unidirectional_cycle() {
        let mut g = Graph::init('a'..='c');

        // a -> b -> c -> a
        assert!(g.connect(&'a', &'b'));
        assert!(g.connect(&'b', &'c'));
        assert!(g.connect(&'c', &'a'));
        
        let depth = g.bfs(&'b').collect::<Vec<_>>();
        assert_eq!(depth, vec![&'b', &'c', &'a']);
    }

    #[test]
    fn bidirectional_cycle() {
        let mut g = Graph::init('a'..='c');

        // a <-> b <-> c <-> a
        assert!(g.biconnect(&'a', &'b'));
        assert!(g.biconnect(&'b', &'c'));
        assert!(g.biconnect(&'c', &'a'));
        
        let depth = g.bfs(&'b').collect::<Vec<_>>();
        dbg!(&g, &depth);
        assert_eq!(depth.len(), 3); // Only visit each once
    }
}