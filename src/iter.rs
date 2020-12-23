use crate::{graph::*, hash};
use std::collections::{HashSet, VecDeque};
use std::hash::Hash;

pub enum Mode {
    Bredth,
    Depth,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Depth
    }
}

impl<T: Hash + Eq> Graph<T> {
    pub fn bfs<'a>(&'a self, start: &'a T) -> Option<Walk<'a, T>> {
        self.walk(start, Mode::Bredth)
    }

    pub fn dfs<'a>(&'a self, start: &'a T) -> Option<Walk<'a, T>> {
        self.walk(start, Mode::Depth)
    }

    pub fn walk<'a>(&'a self, start: &'a T, mode: Mode) -> Option<Walk<'a, T>> {
        self.get(start)?;

        let mut buffer = VecDeque::new();
        buffer.push_front(start);

        Some(Walk {
            mode,
            buffer,
            seen: HashSet::new(),
            graph: &self,
        })
    }
}

pub struct Walk<'a, T> {
    mode: Mode,
    graph: &'a Graph<T>,
    buffer: VecDeque<&'a T>,
    seen: HashSet<&'a T>,
}

impl<'a, T: Hash + Eq> Iterator for Walk<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = loop {
            let next = match self.mode {
                Mode::Bredth => self.buffer.pop_back()?,
                Mode::Depth => self.buffer.pop_front()?,
            };
            if !self.seen.contains(next) {
                break next;
            }
        };

        if let Some(connections) = self.graph.get_adjacent(next) {
            for con in connections {
                self.buffer.push_front(con);
            }
        }
        self.seen.insert(next);
        Some(next)
    }
}

impl<T> Graph<T> {
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            labels: self.nodes.values().map(|v| &v.label).collect(),
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

pub struct Iter<'a, T> {
    labels: Vec<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.labels.pop()
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

    #[test]
    fn empty() {
        let g: Graph<()> = Graph::new();

        assert_eq!(g.iter().count(), 0);
        assert_eq!(g.edges().count(), 0);

        assert!(g.bfs(&()).is_none());
        assert!(g.dfs(&()).is_none());
    }

    #[test]
    fn single() {
        let mut g: Graph<()> = Graph::new();
        g.add(());

        assert_eq!(g.iter().count(), 1);
        assert_eq!(g.edges().count(), 0);

        assert_eq!(g.bfs(&()).unwrap().count(), 1);
        assert_eq!(g.dfs(&()).unwrap().count(), 1);
    }

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

        let bredth = g.dfs(&'a').unwrap().collect::<Vec<_>>();
        let depth = g.bfs(&'a').unwrap().collect::<Vec<_>>();

        assert_order(&bredth);
        assert_eq!((index(&bredth, 'b') - index(&bredth, 'c')).abs(), 1); // c directly below b

        assert_order(&depth);
        assert_eq!((index(&depth, 'b') - index(&depth, 'd')).abs(), 1); // d directly beside b
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

    #[test]
    fn iter() {
        let mut g = Graph::init('a'..='f');

        assert!(g.connect(&'a', &'b'));
        assert!(g.connect(&'b', &'c'));

        assert!(g.connect(&'d', &'e'));
        assert!(g.connect(&'d', &'f'));

        assert_eq!(g.iter().count(), 6)
    }
}
