use crate::lattice::{
    Bipartite_Id_Vec, Edge, EdgeType, Even_Site_Id, Lattice, Odd_Site_Id, Site_Id,
};
use id_collections::{id_type, Id, IdVec};
use rand::{rngs::ThreadRng, Rng};

#[id_type]
pub struct OperatorId(usize);

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum OperatorType {
    //identities are stored as Nones
    D,
    OD,
}
impl OperatorType {
    pub fn flip(&mut self) {
        *self = match self {
            OperatorType::D => OperatorType::OD,
            OperatorType::OD => OperatorType::D,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Operator {
    pub operator_type: OperatorType,
    pub edge: Edge,
    pub even_in_id: OperatorId,
    pub odd_out_id: OperatorId,
}

#[derive(Clone, Debug)]
pub struct State {
    pub alpha: Bipartite_Id_Vec<bool>,
    pub path: IdVec<OperatorId, Option<Operator>>,
    pub n: usize,
    pub latice: Lattice,
}

impl State {
    pub fn next_operator(&self, site: Site_Id, idx: OperatorId, dir: bool) -> Option<OperatorId> {
        for d in 1..=self.path.len() {
            let idx2 = if dir {
                OperatorId((idx.0 + d) % self.path.len())
            } else {
                OperatorId((idx.0 + self.path.len() - d) % self.path.len())
            };
            if let Some(ref op) = self.path[idx2] {
                match site {
                    Site_Id::Even(even) => {
                        if op.edge.even == even {
                            return Some(idx2);
                        }
                    }
                    Site_Id::Odd(odd) => {
                        if op.edge.odd == odd {
                            return Some(idx2);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn insert_diag(&mut self, edge: Edge, idx: OperatorId) {
        assert!(self.path[idx].is_none(), "idx must be empty");
        self.n += 1;
        let op = Operator {
            operator_type: OperatorType::D,
            edge,
            even_in_id: OperatorId(usize::MAX),
            odd_out_id: OperatorId(usize::MAX),
        };
        self.path[idx] = Some(op);
    }
    pub fn delete(&mut self, idx: OperatorId) {
        assert!(self.path[idx].is_some(), "idx must be occupied");
        self.path[idx] = None;
        self.n -= 1;
    }
    pub fn verify(&self) {
        //check that every connetion has no other operators interupting it in the same site
        for (idx, op) in self.path.iter() {
            if let Some(op) = op {
                assert_eq!(
                    self.next_operator(Site_Id::Odd(op.edge.odd), idx, true)
                        .unwrap(),
                    op.odd_out_id,
                    "odd out trace error"
                );
                assert_eq!(
                    self.next_operator(Site_Id::Even(op.edge.even), idx, false)
                        .unwrap(),
                    op.even_in_id,
                    "even in trace error"
                );
            }
        }
        //check that operators sit on oposing spins and that alpha loops
        let mut current = self.alpha.clone();
        for (i, op) in self.path.iter() {
            if let Some(op) = op {
                assert_ne!(
                    current.even[op.edge.even], current.odd[op.edge.odd],
                    "op {:?} not oposing spins",
                    i
                );
                if op.operator_type == OperatorType::OD {
                    current.even[op.edge.even] ^= true;
                    current.odd[op.edge.odd] ^= true;
                }
            }
        }
        assert_eq!(current, self.alpha, "alpha not a loop");
        //check that counts of are correct
        let mut n = 0;
        for (_, op) in self.path.iter() {
            if op.is_some() {
                n += 1;
            }
        }
        assert_eq!(n, self.n, "n count error");
    }
    pub fn directed_loop_update(&mut self, start: OperatorId) -> usize {
        let mut idx = start;
        let mut len = 0;
        loop {
            //going up
            let next = self.path[idx].as_ref().unwrap().odd_out_id;
            if next <= idx {
                self.alpha.odd[self.path[idx].as_ref().unwrap().edge.odd] ^= true;
            }
            idx = next;

            self.path[idx].as_mut().unwrap().operator_type.flip();

            //going down
            let next = self.path[idx].as_ref().unwrap().even_in_id;
            if next >= idx {
                self.alpha.even[self.path[idx].as_ref().unwrap().edge.even] ^= true;
            }
            idx = next;

            self.path[idx].as_mut().unwrap().operator_type.flip();
            len += 1;
            if next == start {
                return len;
            }
        }
    }
    pub fn diagonal_update(&mut self, beta: f64, j1: f64, rng: &mut ThreadRng) {
        let mut current = self.alpha.clone();
        let mut last = Bipartite_Id_Vec {
            even: IdVec::new(),
            odd: IdVec::new(),
        };
        for _ in 0..self.latice.num_even {
            last.even.push(OperatorId(usize::MAX));
        }
        for _ in 0..self.latice.num_odd {
            last.odd.push(OperatorId(usize::MAX));
        }
        for idx in 0..self.path.len() {
            let idx = OperatorId(idx);
            let op = self.path[idx].clone();
            match op {
                Some(Operator {
                    operator_type: OperatorType::OD,
                    edge,
                    ..
                }) => {
                    current.even[edge.even] ^= true;
                    current.odd[edge.odd] ^= true;
                    last.even[edge.even] = idx;
                    last.odd[edge.odd] = idx;
                }
                Some(Operator {
                    operator_type: OperatorType::D,
                    edge,
                    ..
                }) => {
                    let p = (self.path.len() - self.n + 1) as f64
                        / (self.latice.edges.len() as f64
                            * beta
                            * 0.5
                            * match edge.edge_type {
                                EdgeType::One => j1,
                                EdgeType::Two => 1.0,
                            });
                    if rng.gen::<f64>() < p {
                        //if 1.0 < p {
                        self.delete(idx);
                    } else {
                        last.even[edge.even] = idx;
                        last.odd[edge.odd] = idx;
                    }
                }
                None => {
                    //check if insertion is accepted
                    let edge = self.latice.random_edge(rng);
                    if current.even[edge.even] == current.odd[edge.odd] {
                        continue;
                    }
                    let p = (self.latice.edges.len() as f64
                        * beta
                        * 0.5
                        * match edge.edge_type {
                            EdgeType::One => j1,
                            EdgeType::Two => 1.0,
                        })
                        / (self.path.len() - self.n) as f64;
                    if rng.gen::<f64>() < p {
                        //if 0.0 < p {
                        self.insert_diag(edge, idx);
                        last.even[edge.even] = idx;
                        last.odd[edge.odd] = idx;
                    }
                }
            };
        }
        for idx in 0..self.path.len() {
            let idx = OperatorId(idx);
            if let Some(Operator { edge, .. }) = self.path[idx].clone() {
                self.path[idx].as_mut().unwrap().even_in_id = last.even[edge.even];
                self.path[last.odd[edge.odd]].as_mut().unwrap().odd_out_id = idx;
                last.even[edge.even] = idx;
                last.odd[edge.odd] = idx;
            }
        }
    }
    pub fn off_diagonal_update(&mut self, nloop: usize, rng: &mut ThreadRng) -> usize {
        let mut idxs = Vec::new();
        for (i, op) in self.path.iter() {
            if op.is_some() {
                idxs.push(i);
            }
        }
        let mut count = 0;
        if idxs.is_empty() {
            let even_site = Even_Site_Id(rng.gen_range(0..self.alpha.even.len()));
            self.alpha.even[even_site] ^= true;
            let odd_site = Odd_Site_Id(rng.gen_range(0..self.alpha.odd.len()));
            self.alpha.odd[odd_site] ^= true;
            return 0;
        }
        for _ in 0..nloop {
            let idx = idxs[rng.gen_range(0..idxs.len())];
            count += self.directed_loop_update(idx);
        }
        count
    }

    pub fn thermalize(&mut self, beta: f64, j1: f64, rng: &mut ThreadRng) -> usize {
        let mut nloop = 40;
        let mut plato = 0;
        // let mut touched = 0;
        loop {
            self.diagonal_update(beta, j1, rng);
            self.off_diagonal_update(nloop, rng);
            while self.n > self.path.len() * 9 / 10 {
                self.path.push(None);
                plato = 0;
            }
            //self.verify();
            if plato == 5000 {
                return nloop;
            }
            plato += 1;
        }
    }
    pub fn staggered_magnetization(&self) -> f64 {
        let mut sum: f64 = 0.0;
        let mut points = 0.0;
        let mut current = self.alpha.clone();
        let mut current_sm: f64 = 0.0;
        for (_, spin) in &current.even {
            if *spin {
                current_sm += 0.5;
            } else {
                current_sm -= 0.5;
            }
        }
        for (_, spin) in &current.odd {
            if *spin {
                current_sm -= 0.5;
            } else {
                current_sm += 0.5;
            }
        }
        for (_, op) in self.path.iter() {
            if let Some(op) = op {
                if op.operator_type == OperatorType::OD {
                    if current.even[op.edge.even] {
                        current_sm -= 2.0;
                    } else {
                        current_sm += 2.0;
                    }
                    current.even[op.edge.even] ^= true;
                    current.odd[op.edge.odd] ^= true;
                }
            }
            sum += current_sm.abs();
            points += 1.0;
        }
        sum / points
    }
    pub fn sample(
        &mut self,
        // weights: &Vec<f64>,
        nloop: usize,
        beta: f64,
        j1: f64,
        rng: &mut ThreadRng,
    ) -> (f64, f64) {
        self.diagonal_update(beta, j1, rng);
        self.off_diagonal_update(nloop, rng);

        let energy = -(self.n as f64) / beta
            + j1 * self.latice.edge_count_1 as f64 / 4.0
            + self.latice.edge_count_2 as f64 / 4.0;
        let sm = self.staggered_magnetization();
        (energy, sm)
    }

    pub fn new(latice: &Lattice, m: usize, rng: &mut ThreadRng) -> State {
        let mut alpha_even = IdVec::new();
        let mut alpha_odd = IdVec::new();
        for _ in 0..latice.num_even {
            alpha_even.push(rng.gen());
        }
        for _ in 0..latice.num_odd {
            alpha_odd.push(rng.gen());
        }
        let mut path = IdVec::new();
        for _ in 0..m {
            path.push(None);
        }
        let s = State {
            alpha: Bipartite_Id_Vec {
                even: alpha_even,
                odd: alpha_odd,
            },
            path,
            n: 0,
            latice: latice.clone(),
        };
        s.verify();
        s
    }
}
