use crate::graph::*;
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
    
    pub fn ordering<'a>(&'a self) -> Ordering<'a, T> {
        Ordering::new(self)
    }

    pub fn bfs<'a>(&'a self) -> Walk<'a, T> {
        self.search(Mode::Bredth)
    }

    pub fn dfs<'a>(&'a self) -> Walk<'a, T> {
        self.search(Mode::Depth)
    }

    pub fn search<'a>(&'a self, mode: Mode) -> Walk<'a, T> {
        let mut buffer = VecDeque::new();
        for root in self.sources() {
            buffer.push_front(root);
        }

        Walk {
            mode,
            buffer,
            visited: HashSet::new(),
            graph: &self,
        }
    }

    pub fn walk<'a>(&'a self, start: &'a T, mode: Mode) -> Option<Walk<'a, T>> {
        self.get(start)?;

        let mut buffer = VecDeque::new();
        buffer.push_front(start);

        Some(Walk {
            mode,
            buffer,
            visited: HashSet::new(),
            graph: &self,
        })
    }
}

pub struct Walk<'a, T> {
    mode: Mode,
    graph: &'a Graph<T>,
    buffer: VecDeque<&'a T>,
    visited: HashSet<&'a T>,
}

impl<'a, T: Hash + Eq> Iterator for Walk<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = loop {
            let next = match self.mode {
                Mode::Bredth => self.buffer.pop_back()?,
                Mode::Depth => self.buffer.pop_front()?,
            };
            if !self.visited.contains(next) {
                break next;
            }
        };

        if let Some(connections) = self.graph.neighbors(next) {
            for con in connections {
                self.buffer.push_front(con);
            }
        }
        self.visited.insert(next);
        Some(next)
    }
}

impl<T> Graph<T> {
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            inner: self.nodes()
        }
    }

    pub fn nodes<'a>(&'a self) -> NodeIter<'a, T> {
        NodeIter {
            inner: self.nodes.values(),
        }
    }

    pub fn edges<'a>(&'a self) -> EdgeIter<'a, T> {
        EdgeIter {
            graph: &self,
            nodes: self.nodes(),
            next: Vec::new(),
        }
    }
}

pub struct NodeIter<'a, T> {
    inner: std::collections::hash_map::Values<'a, u64, Node<T>>,
}

impl<'a, T> Iterator for NodeIter<'a, T> {
    type Item = &'a Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub struct Iter<'a, T> {
    inner: NodeIter<'a, T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|n| &n.label)
    }
}

pub struct EdgeIter<'a, T> {
    graph: &'a Graph<T>,
    nodes: NodeIter<'a, T>,
    next: Vec<(&'a T, &'a  T)>,
}

impl<'a, T> Iterator for EdgeIter<'a, T> {
    type Item = (&'a T, &'a  T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(edge) = self.next.pop() {
            return Some(edge);
        }

        let from = self.nodes.next()?;
        for n in &from.neighbors {
            let to = self.graph.nodes.get(n).unwrap();
            self.next.push((&from.label, &to.label));
        }
        self.next()
    }
}

pub struct Ordering<'a, T> {
    inner: std::vec::IntoIter<&'a T>,
}

impl<'a, T: Hash + Eq> Ordering<'a, T> {
    fn new(graph: &'a Graph<T>) -> Self {
        let mut result = Vec::new();
        let mut degrees = graph.indegrees();

        let mut next = Vec::new();
        next.extend(graph.sources());
        while let Some(label) = next.pop() {
            result.push(label);
            for neighbor in graph.neighbors(label).unwrap() {
                let degree = degrees.get_mut(neighbor).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    next.push(neighbor);
                }
            }
        }

        Ordering {
            inner: result.into_iter()
        }
    }
}

impl<'a, T> Iterator for Ordering<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
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

        assert!(g.walk(&(), Mode::Depth).is_none());
        assert!(g.walk(&(), Mode::Bredth).is_none());
        
        assert_eq!(g.size(), 0);
        assert_eq!(g.dfs().count(), 0);
        assert_eq!(g.bfs().count(), 0);
    }

    #[test]
    fn single() {
        let mut g = Graph::new();
        g.add(());

        assert_eq!(g.iter().count(), 1);
        assert_eq!(g.edges().count(), 0);

        assert_eq!(g.size(), 1);
        assert_eq!(g.bfs().count(), 1);
        assert_eq!(g.dfs().count(), 1);
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

        let bredth = g.dfs().collect::<Vec<_>>();
        let depth = g.bfs().collect::<Vec<_>>();

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

    #[test]
    fn ordering() {
        let mut g = Graph::init('a'..='f');
    
        assert!(g.connect(&'a', &'b')); // a
        assert!(g.connect(&'b', &'c')); // |\
        assert!(g.connect(&'a', &'d')); // b d
        assert!(g.connect(&'c', &'e')); // |/|\
        assert!(g.connect(&'d', &'c')); // c | f
        assert!(g.connect(&'d', &'e')); //  \|/
        assert!(g.connect(&'d', &'f')); //   e
        assert!(g.connect(&'f', &'e'));

        let order = g.ordering().collect::<Vec<_>>();
        assert_eq!(index(&order, 'a'), 0);
        assert!(index(&order, 'b') < index(&order, 'c'));
        assert!(index(&order, 'd') < index(&order, 'c'));
        assert!(index(&order, 'd') < index(&order, 'f'));
        assert_eq!(index(&order, 'e'), 5);
    }
}
