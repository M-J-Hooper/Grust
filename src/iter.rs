use crate::{graph::*, hash};
use std::collections::{HashSet, VecDeque};
use std::hash::Hash;

pub enum Mode {
    Bredth,
    Depth,
}

impl<T: Hash> Graph<T> {
    pub fn bfs<'a>(&'a self, start: &'a T) -> WalkIter<'a, T> {
        self.walk(start, Mode::Bredth)
    }

    pub fn dfs<'a>(&'a self, start: &'a T) -> WalkIter<'a, T> {
        self.walk(start, Mode::Depth)
    }

    pub fn walk<'a>(&'a self, start: &'a T, mode: Mode) -> WalkIter<'a, T> {
        let mut buffer = VecDeque::new();
        buffer.push_front(start);

        let mut visited = HashSet::new();
        visited.insert(hash(start));
        WalkIter {
            mode,
            buffer,
            visited,
            graph: &self,
        }
    }

    pub fn edges<'a>(&'a self) -> EdgeIter<'a, T> {
        EdgeIter {
            graph: &self,
            nodes: self.nodes.values().collect(),
            edges: Vec::new(),
        }
    }
}

pub struct WalkIter<'a, T> {
    mode: Mode,
    graph: &'a Graph<T>,
    buffer: VecDeque<&'a T>,
    visited: HashSet<u64>,
}

impl<'a, T: Hash + Eq> Iterator for WalkIter<'a, T> {
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

pub struct Edge<'a, T> {
    pub from: &'a T,
    pub to: &'a T,
    pub weight: i64,
}

pub struct EdgeIter<'a, T> {
    graph: &'a Graph<T>,
    nodes: Vec<&'a Node<T>>,
    edges: Vec<Edge<'a, T>>,
}

impl<'a, T> Iterator for EdgeIter<'a, T> {
    type Item = Edge<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(edge) = self.edges.pop() {
            return Some(edge);
        }

        let from = self.nodes.pop()?;
        for edge in &from.edges {
            let to = self.graph.nodes.get(edge.0).unwrap();
            self.edges.push(Edge {
                from: &from.label,
                to: &to.label,
                weight: edge.1.to_owned(),
            });
        }
        self.next()
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

    #[test]
    fn edges() {
        let mut g = Graph::init('a'..='f');

        assert!(g.connect(&'a', &'b'));
        assert!(g.connect(&'b', &'c'));

        assert!(g.connect(&'d', &'e'));
        assert!(g.connect(&'d', &'f'));

        assert_eq!(g.edges().count(), 4)
    }
}
