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

impl<T> Graph<T> {
    pub fn bfs<'a>(&'a self, start: &'a T) -> Iter<'a, T> {
        self.search(start, Mode::Bredth)
    }

    pub fn dfs<'a>(&'a self, start: &'a T) -> Iter<'a, T> {
        self.search(start, Mode::Depth)
    }

    pub fn search<'a>(&'a self, start: &'a T, mode: Mode) -> Iter<'a, T> {
        let mut buffer = VecDeque::new();
        buffer.push_front(start);
        Iter {
            mode,
            buffer,
            graph: &self,
            visited: HashSet::new(),
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

        let n = hash(next);
        self.visited.insert(n);

        if let Some(set) = self.graph.adjacent(next) {
            for adj in set {
                let m = hash(adj);
                if !self.visited.contains(&m) {
                    self.buffer.push_front(adj);
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
    fn bfs_vs_dfs() {
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
}