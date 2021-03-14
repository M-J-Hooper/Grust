use fmt::Display;

use crate::hash;
use crate::graph::*;
use std::hash::Hash;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Hash)]
struct Row<'a, T> {
    label: &'a T,
    elements: Vec<Element<'a, T>>,
}

impl<'a, T: PartialEq> Row<'a, T> {
    fn new(label: &'a T) -> Self {
        Row {
            label,
            elements: Vec::new(),
        }
    }

    fn index_of(&self, label: &'a T) -> Option<usize> {
        for (i, el) in self.elements.iter().enumerate() {
            match el {
                Element::Node if label == self.label => return Some(i),
                Element::Connector(l) if label == *l => return Some(i),
                _ => {},
            }
        }
        None
    }
}

impl<'a, T: fmt::Display> fmt::Display for Row<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.elements.iter()
            .map(|el| match el {
                Element::Empty => " ".into(),
                Element::Node => self.label.to_string(),
                Element::Connector(_) => "|".into(),
            })
            .collect::<Vec<_>>()
            .join(" ");
        
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Element<'a, T> {
    Empty,
    Node,
    Connector(&'a T),
}

impl<'a, T> Element<'a, T> {
    fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            _ => false,
        }
    }

    fn is_node(&self) -> bool {
        match self {
            Self::Node => true,
            _ => false,
        }
    }
}

impl<T: fmt::Display + Hash + Eq + fmt::Debug> Graph<T> {
    
    pub fn diagram(&self) -> String {
        let mut result = Vec::new();
        let rows = self.get_rows();
        let mut i = 0;
        while i + 1 < rows.len() {
            let prev_row = &rows[i];
            let next_row = &rows[i + 1];

            result.push(prev_row.to_string());
            result.push(self.draw_connectors(prev_row, next_row));

            i += 1;
        }
        result.push(rows[i].to_string());
        result.join(r"\n")
    }

    fn draw_connectors(&self, prev: &Row<T>, next: &Row<T>) -> String {
        let mut result: Vec<Vec<char>> = Vec::new();
        for (from, el) in prev.elements.iter().enumerate() {
            match el {
                Element::Connector(l) => {
                    let to = next.index_of(*l).unwrap();
                    self.draw_connector(&mut result, from, to);
                },
                Element::Node => {
                    for neighbor in self.neighbors(prev.label).unwrap() {
                        let to = next.index_of(neighbor).unwrap();
                        self.draw_connector(result, from, to);
                    }
                },
                Element::Empty => {},
            }
        }

        result.join(r"\n")
    }

    fn draw_connector(&self, grid: &mut Vec<Vec<char>>, from: usize, to: usize) {

    }

    fn get_rows(&self) -> Vec<Row<'_, T>> {
        let mut rows: Vec<Row<'_, T>> = Vec::new();
        for label in self.ordering() {
            let mut next_row = Row::new(label);
            if let Some(prev_row) = rows.iter().last() {
                let mut placed_node = false;
                let mut prev_neighbors = self.neighbors(prev_row.label)
                    .unwrap()
                    .into_iter()
                    .filter(|l| *l != label);

                for prev_element in &prev_row.elements {
                    let next_element = match *prev_element {
                        Element::Empty => {
                            if placed_node {
                                None
                            } else {
                                Some(Element::Node)
                            }
                        },
                        Element::Connector(l) => {
                            if l == label {
                                if placed_node {
                                    None
                                } else {
                                    Some(Element::Node)
                                }
                            } else {
                                Some(Element::Connector(l))
                            }
                        },
                        Element::Node => None,
                    };
                    let new_element = if let Some(el) = next_element {
                        if el.is_node() {
                            placed_node = true;
                        }
                        el
                    } else {
                        if placed_node {
                            prev_neighbors.next()
                                .map(|l| Element::Connector(l))
                                .unwrap_or(Element::Empty)
                        } else {
                            placed_node = true;
                            Element::Node
                        }
                        
                    };
                    next_row.elements.push(new_element);

                    if !placed_node {
                        next_row.elements.push(Element::Node);
                    }
                    while let Some(l) = prev_neighbors.next() {
                        next_row.elements.push(Element::Connector(l));
                    }
                }
            } else {
                next_row.elements.push(Element::Node);
            }
            rows.push(next_row);
        }

        dbg!(&rows);
        rows
    }
}

impl<T: fmt::Display + Hash + Eq + fmt::Debug> fmt::Display for Graph<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.diagram())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn disconnected_line_rows() {
        let mut g = Graph::init('a'..='c');
        let mut rows = g.get_rows().into_iter();

        let mut expected = HashSet::new();
        let ls = ('a'..='c').into_iter().collect::<Vec<_>>();
        for l in &ls {
            expected.insert(Row {
                label: l,
                elements: vec![Element::Node],
            });
        }

        assert!(expected.contains(&rows.next().unwrap()));
        assert!(expected.contains(&rows.next().unwrap()));
        assert!(expected.contains(&rows.next().unwrap()));
        assert!(rows.next().is_none());
    }

    #[test]
    fn connected_line_rows() {
        let mut g = Graph::init('a'..='c');
        assert!(g.connect(&'a', &'b'));
        assert!(g.connect(&'b', &'c'));
        let mut rows = g.get_rows().into_iter();

        assert_eq!(rows.next(), Some(Row { label: &'a', elements: vec![Element::Node]}));
        assert_eq!(rows.next(), Some(Row { label: &'b', elements: vec![Element::Node]}));
        assert_eq!(rows.next(), Some(Row { label: &'c', elements: vec![Element::Node]}));
        assert!(rows.next().is_none());
    }

    #[test]
    fn simple_unique_ordering_rows() {
        let mut g = Graph::init('a'..='c');
        assert!(g.connect(&'a', &'b'));
        assert!(g.connect(&'a', &'c'));
        assert!(g.connect(&'b', &'c'));

        assert_eq!(g.get_rows(), vec![
            Row {
                label: &'a',
                elements: vec![Element::Node],
            },
            Row {
                label: &'b',
                elements: vec![Element::Node, Element::Connector(&'c')],
            },
            Row {
                label: &'c',
                elements: vec![Element::Node, Element::Empty],
            }
        ]);
    }

    #[test]
    fn complex_unique_ordering_rows() {
        let mut g = Graph::init('a'..='e');
        assert!(g.connect(&'a', &'b')); // a
        assert!(g.connect(&'a', &'c')); // |\
        assert!(g.connect(&'b', &'c')); // b |
        assert!(g.connect(&'b', &'e')); // |\|
                                        // | |\
                                        // | | |
                                        // |/ /
        assert!(g.connect(&'c', &'d')); // c | 
        assert!(g.connect(&'c', &'e')); // |\|
                                        // | |\
                                        // | | |
                                        // |/ /
        assert!(g.connect(&'e', &'d')); // e |
                                        // |/
                                        // d

        pretty_assertions::assert_eq!(g.get_rows(), vec![
            Row {
                label: &'a',
                elements: vec![Element::Node],
            },
            Row {
                label: &'b',
                elements: vec![Element::Node, Element::Connector(&'c')],
            },
            Row {
                label: &'c',
                elements: vec![Element::Node, Element::Connector(&'e'), Element::Empty],
            },
            Row {
                label: &'e',
                elements: vec![Element::Node, Element::Connector(&'d'), Element::Empty, Element::Empty],
            },
            Row {
                label: &'d',
                elements: vec![Element::Node, Element::Empty, Element::Empty, Element::Empty],
            }
        ]);

        dbg!(g.diagram());
    }

    #[test]
    fn disconnected_line() {
        let g = Graph::init('a'..='c');
        assert_eq!(g.to_string(), "
a

b

c
"
        );
    }

    #[test]
    fn connected_line() {
        let mut g = Graph::init('a'..='c');
        assert!(g.connect(&'a', &'b'));
        assert!(g.connect(&'b', &'c'));

        assert_eq!(g.to_string(), r"
a
|
b
|
c
"
        );
    }

    #[test]
    fn unique_ordering() {
        let mut g = Graph::init('a'..='e');
        assert!(g.connect(&'a', &'b'));
        assert!(g.connect(&'a', &'c'));
        assert!(g.connect(&'b', &'c'));
        assert!(g.connect(&'b', &'e'));
        assert!(g.connect(&'c', &'d'));
        assert!(g.connect(&'c', &'e'));
        assert!(g.connect(&'e', &'d'));

        assert_eq!(g.to_string(), r"
a
|\
| b
|/|
c |
|\|
| e
|/
d
"
        );
    }

    #[test]
    fn complex() {
        let mut g = Graph::init('a'..='f');
    
        assert!(g.connect(&'a', &'b')); // a
        assert!(g.connect(&'b', &'c')); // |\
        assert!(g.connect(&'a', &'d')); // b d
        assert!(g.connect(&'c', &'e')); // |/|\
        assert!(g.connect(&'d', &'c')); // c | f
        assert!(g.connect(&'d', &'e')); //  \|/
        assert!(g.connect(&'d', &'f')); //   e
        assert!(g.connect(&'f', &'e'));

        assert_eq!(g.to_string(), r"
a
|\
b d
|/|\
c | f
 \|/
  e
"
        );
    }
}
