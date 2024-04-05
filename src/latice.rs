use rand::{rngs::ThreadRng, Rng};
use std::{mem::swap, vec};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum EdgeType {
    One,
    Two,
}

#[derive(Clone)]
pub struct Edge {
    pub edge_type: EdgeType,
    pub even: usize,
    pub odd: usize,
}
pub struct Latice {
    pub width: usize,
    pub height: usize,
    pub edges: Vec<Edge>,
}
impl Latice {
    //nodes zigzag to make graph bipartite over even odd partition
    pub fn node_id(&self, x: usize, y: usize) -> usize {
        assert!(
            x < self.width && y < self.height,
            "x and y must be less than width and height"
        );
        y * self.width + if y % 2 == 0 { x } else { self.width - x - 1 }
    }
    pub fn node_pos(&self, id: usize) -> (usize, usize) {
        assert!(
            id < self.width * self.height,
            "id must be less than width*height"
        );
        let h = id / self.width;
        if h % 2 == 0 {
            (id % self.width, h)
        } else {
            (self.width - id % self.width - 1, h)
        }
    }
    fn add_edge(&mut self, edge_type: EdgeType, mut a: usize, mut b: usize) {
        if b % 2 == 0 {
            swap(&mut a, &mut b);
        }
        assert_eq!(a % 2, 0, "a must be even, got {}", a);
        assert_eq!(b % 2, 1, "b must be odd, got {}", b);
        self.edges.push(Edge {
            edge_type,
            even: a,
            odd: b,
        });
    }
    pub fn new(width: usize, height: usize) -> Latice {
        assert!(width % 2 == 0, "width must be even");
        assert!(height % 2 == 0, "height must be even");
        let mut l = Latice {
            width,
            height,
            edges: Vec::new(),
        };
        for i in 0..width {
            for j in 0..height {
                l.add_edge(
                    EdgeType::Two,
                    l.node_id(i, j),
                    l.node_id((i + 1) % width, j),
                );
                l.add_edge(
                    if j % 2 == 0 {
                        EdgeType::One
                    } else {
                        EdgeType::Two
                    },
                    l.node_id(i, j),
                    l.node_id(i, (j + 1) % height),
                );
            }
        }
        l
    }
    pub fn random_edge(&self, rng: &mut ThreadRng) -> Edge {
        self.edges[rng.gen_range(0..self.edges.len())].clone()
    }
    pub fn staggered_magnetization_weights(&self) -> Vec<f64> {
        let mut weights = vec![0.0; self.width * self.height];
        for x in 0..self.width {
            for y in 0..self.height {
                let id = self.node_id(x, y);
                weights[id] = if (x + y) % 2 == 0 { 1.0 } else { -1.0 };
            }
        }
        weights
    }
}
