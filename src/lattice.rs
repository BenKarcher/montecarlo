use id_collections::{id_type, IdVec};
use rand::distributions::Uniform;
use rand::{rngs::ThreadRng, Rng};
use std::fmt::Debug;
use std::{mem::swap, vec};

pub fn new_rectangle(width: usize, height: usize) -> LatticeConstructor<(usize, usize)> {
    let mut nodes = Vec::new();
    for i in 0..width {
        for j in 0..height {
            nodes.push((i, j));
        }
    }
    LatticeConstructor::new(nodes)
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum EdgeType {
    One,
    Two,
}

struct EdgeConstructor<T: Eq> {
    pub a: T,
    pub b: T,
    pub edge_type: EdgeType,
}
pub struct LatticeConstructor<T: Eq + Clone> {
    nodes: Vec<T>,
    edges: Vec<EdgeConstructor<T>>,
}

impl<T: Eq + Clone> LatticeConstructor<T> {
    pub fn new(nodes: Vec<T>) -> LatticeConstructor<T> {
        LatticeConstructor {
            nodes: nodes,
            edges: Vec::new(),
        }
    }
    pub fn add_edge(&mut self, a: T, b: T, edge_type: EdgeType) {
        self.edges.push(EdgeConstructor { a, b, edge_type });
    }
    pub fn get_bipartite_coloring(&self) -> (Vec<T>, Vec<T>) {
        for edge in self.edges.iter() {
            if !self.nodes.contains(&edge.a) {
                panic!("Node not in graph");
            }
            if !self.nodes.contains(&edge.b) {
                panic!("Node not in graph");
            }
        }
        let mut even = Vec::new();
        let mut odd = Vec::new();
        even.push(self.nodes[0].clone());
        while even.len() + odd.len() < self.nodes.len() {
            let mut progress = false;
            for edge in self.edges.iter() {
                match (
                    even.contains(&edge.a),
                    odd.contains(&edge.a),
                    even.contains(&edge.b),
                    odd.contains(&edge.b),
                ) {
                    (true, _, true, _) | (_, true, _, true) => panic!("Graph is not bipartite"),
                    (true, false, false, false) => {
                        odd.push(edge.b.clone());
                        progress = true;
                    }
                    (false, true, false, false) => {
                        even.push(edge.b.clone());
                        progress = true;
                    }
                    (false, false, true, false) => {
                        odd.push(edge.a.clone());
                        progress = true;
                    }
                    (false, false, false, true) => {
                        even.push(edge.a.clone());
                        progress = true;
                    }
                    (true, false, false, true) | (false, true, true, false) => {}
                    _ => panic!("Error in bipartite coloring"),
                }
            }
            if !progress {
                for node in self.nodes.iter() {
                    if !even.contains(node) && !odd.contains(node) {
                        even.push(node.clone());
                        break;
                    }
                }
            }
        }
        (even, odd)
    }
    pub fn build(self) -> Lattice {
        let (even, odd) = self.get_bipartite_coloring();
        let mut edges = Vec::new();
        for edge in self.edges.iter() {
            let even_id = even
                .iter()
                .position(|x| x == &edge.a || x == &edge.b)
                .unwrap();
            let odd_id = odd
                .iter()
                .position(|x| x == &edge.a || x == &edge.b)
                .unwrap();
            edges.push(Edge {
                edge_type: edge.edge_type,
                even: Even_Site_Id(even_id),
                odd: Odd_Site_Id(odd_id),
            });
        }
        Lattice::new(even.len(), odd.len(), edges)
    }
}

#[id_type]
pub struct Even_Site_Id(pub usize);
#[id_type]
pub struct Odd_Site_Id(pub usize);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Site_Id {
    Even(Even_Site_Id),
    Odd(Odd_Site_Id),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bipartite_Id_Vec<T> {
    pub even: IdVec<Even_Site_Id, T>,
    pub odd: IdVec<Odd_Site_Id, T>,
}
impl<T> Bipartite_Id_Vec<T> {
    pub fn new() -> Bipartite_Id_Vec<T> {
        Bipartite_Id_Vec {
            even: IdVec::new(),
            odd: IdVec::new(),
        }
    }
    pub fn get(&self, site_id: Site_Id) -> &T {
        match site_id {
            Site_Id::Even(id) => &self.even[id],
            Site_Id::Odd(id) => &self.odd[id],
        }
    }
    pub fn get_mut(&mut self, site_id: Site_Id) -> &mut T {
        match site_id {
            Site_Id::Even(id) => &mut self.even[id],
            Site_Id::Odd(id) => &mut self.odd[id],
        }
    }
    pub fn set(&mut self, site_id: Site_Id, value: T) {
        match site_id {
            Site_Id::Even(id) => self.even[id] = value,
            Site_Id::Odd(id) => self.odd[id] = value,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Edge {
    pub edge_type: EdgeType,
    pub even: Even_Site_Id,
    pub odd: Odd_Site_Id,
}

#[derive(Clone, Debug)]
pub struct Lattice {
    pub num_even: usize,
    pub num_odd: usize,
    pub edge_count_1: usize,
    pub edge_count_2: usize,
    pub edges: Vec<Edge>,
    distribution: Uniform<usize>,
}
impl Lattice {
    pub fn new(num_even: usize, num_odd: usize, edges: Vec<Edge>) -> Lattice {
        let distribution = Uniform::new(0, edges.len());
        let edge_count_1 = edges
            .iter()
            .filter(|e| e.edge_type == EdgeType::One)
            .count();
        let edge_count_2 = edges.len() - edge_count_1;
        Lattice {
            num_even: num_even,
            num_odd: num_odd,
            edges: edges,
            distribution,
            edge_count_1,
            edge_count_2,
        }
    }
    pub fn random_edge(&self, rng: &mut ThreadRng) -> Edge {
        self.edges[rng.sample(self.distribution)].clone()
    }
}

// #[derive(Eq, Clone)]
// pub struct Bipartite_Vector<T> {
//     pub even: Vec<T>,
//     pub odd: Vec<T>,
// }

// impl<T> Bipartite_Vector<T> {
//     pub fn new() -> Bipartite_Vector<T> {
//         Bipartite_Vector {
//             even: Vec::new(),
//             odd: Vec::new(),
//         }
//     }
//     pub fn get(&self, site_id: Site_Id) -> &T {
//         match site_id {
//             Site_Id::Even(Even_Site_Id(id)) => &self.even[id],
//             Site_Id::Odd(Odd_Site_Id(id)) => &self.odd[id],
//         }
//     }
//     pub fn get_mut(&mut self, site_id: Site_Id) -> &mut T {
//         match site_id {
//             Site_Id::Even(Even_Site_Id(id)) => &mut self.even[id],
//             Site_Id::Odd(Odd_Site_Id(id)) => &mut self.odd[id],
//         }
//     }
//     pub fn set(&mut self, site_id: Site_Id, value: T) {
//         match site_id {
//             Site_Id::Even(Even_Site_Id(id)) => self.even[id] = value,
//             Site_Id::Odd(Odd_Site_Id(id)) => self.odd[id] = value,
//         }
//     }
//     pub fn get_even(&self, id: Even_Site_Id) -> &T {
//         &self.even[id.0]
//     }
//     pub fn get_odd(&self, id: Odd_Site_Id) -> &T {
//         &self.odd[id.0]
//     }
//     pub fn get_even_mut(&mut self, id: Even_Site_Id) -> &mut T {
//         &mut self.even[id.0]
//     }
//     pub fn get_odd_mut(&mut self, id: Odd_Site_Id) -> &mut T {
//         &mut self.odd[id.0]
//     }
//     pub fn set_even(&mut self, id: Even_Site_Id, value: T) {
//         self.even[id.0] = value;
//     }
//     pub fn set_odd(&mut self, id: Odd_Site_Id, value: T) {
//         self.odd[id.0] = value;
//     }
// }
// impl<T: PartialEq> PartialEq for Bipartite_Vector<T> {
//     fn eq(&self, other: &Self) -> bool {
//         self.even == other.even && self.odd == other.odd
//     }
// }
// impl<T: Debug> Debug for Bipartite_Vector<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Even: {:?}, Odd: {:?}", self.even, self.odd)
//     }
// }
