use crate::latice::{Bipartite_Vector, Edge, EdgeType, Even_Site_Id, Latice, Odd_Site_Id, Site_Id};
use rand::{rngs::ThreadRng, Rng};

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
    pub even_in_id: usize,
    pub odd_out_id: usize,
}

#[derive(Clone, Debug)]
pub struct State {
    pub alpha: Bipartite_Vector<bool>,
    pub last: Bipartite_Vector<Option<usize>>,
    pub path: Vec<Option<Operator>>,
    pub n: usize,
    pub latice: Latice,
}

impl State {
    pub fn next_operator(&self, site: Site_Id, idx: usize, dir: bool) -> Option<usize> {
        for d in 1..=self.path.len() {
            let idx2 = if dir {
                (idx + d) % self.path.len()
            } else {
                (idx + self.path.len() - d) % self.path.len()
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

    pub fn insert_diag(
        &mut self,
        edge: Edge,
        idx: usize,
        former: &mut Bipartite_Vector<Option<usize>>,
    ) {
        assert!(self.path[idx].is_none(), "idx must be empty");
        self.n += 1;
        //let even_in_id = former.get_even(edge.even).unwrap_or(idx);
        let even_in_id = idx; //WRONG BUT WILL BE FIXED IN NEXT PASS
        let odd_out_id = match former.get_odd(edge.odd) {
            Some(odd_out_id) => {
                let next = self.path[*odd_out_id].as_ref().unwrap().odd_out_id;
                self.path[*odd_out_id].as_mut().unwrap().odd_out_id = idx;
                next
            }
            None => idx,
        };
        former.set_even(edge.even, Some(idx));
        former.set_odd(edge.odd, Some(idx));
        let op = Operator {
            operator_type: OperatorType::D,
            edge: edge.clone(),
            even_in_id,
            odd_out_id,
        };
        match self.last.get_even(edge.even) {
            None => {
                self.last.set_even(edge.even, Some(idx));
            }
            Some(last_idx) if *last_idx < idx => {
                self.last.set_even(edge.even, Some(idx));
            }
            _ => {}
        }
        match self.last.get_odd(edge.odd) {
            None => {
                self.last.set_odd(edge.odd, Some(idx));
            }
            Some(last_idx) if *last_idx < idx => {
                self.last.set_odd(edge.odd, Some(idx));
            }
            _ => {}
        }

        self.path[idx] = Some(op);
    }
    pub fn delete(&mut self, idx: usize, former: &mut Bipartite_Vector<Option<usize>>) {
        let op = self.path[idx].take().unwrap();
        self.n -= 1;
        let prev_odd_idx = former.get_odd(op.edge.odd).unwrap();
        if prev_odd_idx == idx {
            former.set_odd(op.edge.odd, None);
        } else {
            self.path[prev_odd_idx].as_mut().unwrap().odd_out_id = op.odd_out_id;
        }
        if former.get_even(op.edge.even).unwrap() == idx {
            former.set_even(op.edge.even, None);
        }
        if self.last.get_even(op.edge.even).unwrap() == idx {
            self.last
                .set_even(op.edge.even, *former.get_even(op.edge.even));
        }
        if self.last.get_odd(op.edge.odd).unwrap() == idx {
            self.last.set_odd(op.edge.odd, *former.get_odd(op.edge.odd));
        }
    }
    pub fn verify(&self) {
        //check that every connetion has no other operators interupting it in the same site
        for (idx, op) in self.path.iter().enumerate() {
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
        for (i, op) in self.path.iter().enumerate() {
            if let Some(op) = op {
                assert_ne!(
                    current.get_even(op.edge.even),
                    current.get_odd(op.edge.odd),
                    "op {} not oposing spins",
                    i
                );
                if op.operator_type == OperatorType::OD {
                    *current.get_even_mut(op.edge.even) ^= true;
                    *current.get_odd_mut(op.edge.odd) ^= true;
                }
            }
        }
        assert_eq!(current, self.alpha, "alpha not a loop");
        //check that counts of are correct
        let mut n = 0;
        for op in self.path.iter() {
            if op.is_some() {
                n += 1;
            }
        }
        assert_eq!(n, self.n, "n count error");
    }
    pub fn directed_loop_update(&mut self, start: usize) -> usize {
        let mut idx = start;
        let mut len = 0;
        loop {
            //going up
            let next = self.path[idx].as_ref().unwrap().odd_out_id;
            if next <= idx {
                *self
                    .alpha
                    .get_odd_mut(self.path[idx].as_ref().unwrap().edge.odd) ^= true;
            }
            idx = next;

            self.path[idx].as_mut().unwrap().operator_type.flip();

            //going down
            let next = self.path[idx].as_ref().unwrap().even_in_id;
            if next >= idx {
                *self
                    .alpha
                    .get_even_mut(self.path[idx].as_ref().unwrap().edge.even) ^= true;
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
        let mut latest = self.last.clone();
        for idx in 0..self.path.len() {
            let op = self.path[idx].clone();
            match op {
                Some(Operator {
                    operator_type: OperatorType::OD,
                    edge,
                    ..
                }) => {
                    // self.path[idx].as_mut().unwrap().even_in_id =
                    //     latest.get_even(edge.even).unwrap();
                    *current.get_even_mut(edge.even) ^= true;
                    *current.get_odd_mut(edge.odd) ^= true;
                    latest.set_even(edge.even, Some(idx));
                    latest.set_odd(edge.odd, Some(idx));
                }
                Some(Operator {
                    operator_type: OperatorType::D,
                    edge,
                    ..
                }) => {
                    //check if removal is accepted
                    // self.path[idx].as_mut().unwrap().even_in_id =
                    //     latest.get_even(edge.even).unwrap();
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
                        self.delete(idx, &mut latest);
                    } else {
                        latest.set_even(edge.even, Some(idx));
                        latest.set_odd(edge.odd, Some(idx));
                    }
                }
                None => {
                    //check if insertion is accepted
                    let edge = self.latice.random_edge(rng);
                    if current.get_even(edge.even) == current.get_odd(edge.odd) {
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
                        self.insert_diag(edge, idx, &mut latest);
                    }
                }
            };
        }
        let mut latest = self.last.clone();
        for idx in 0..self.path.len() {
            if let Some(op) = self.path[idx].clone() {
                self.path[idx].as_mut().unwrap().even_in_id =
                    latest.get_even(op.edge.even).unwrap();
                self
            }
        }
    }
    pub fn off_diagonal_update(&mut self, nloop: usize, rng: &mut ThreadRng) -> usize {
        let mut idxs = Vec::new();
        for (i, op) in self.path.iter().enumerate() {
            if op.is_some() {
                idxs.push(i);
            }
        }
        let mut count = 0;
        if idxs.is_empty() {
            let even_site = rng.gen_range(0..self.alpha.even.len());
            self.alpha.even[even_site] ^= true;
            let odd_site = rng.gen_range(0..self.alpha.odd.len());
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
            if plato == 20000 {
                return nloop;
            }
            plato += 1;
        }
    }
    // pub fn si_operator(&self, weights: &Vec<f64>) -> f64 {
    //     let mut sum = 0.0;
    //     assert_eq!(
    //         weights.len(),
    //         self.alpha.len(),
    //         "weights must have the same length as alpha"
    //     );
    //     for (w, a) in weights.iter().zip(self.alpha.iter()) {
    //         sum += w * (if *a { 0.5 } else { -0.5 });
    //     }
    //     sum.abs()
    // }
    pub fn sample(
        &mut self,
        // weights: &Vec<f64>,
        nloop: usize,
        beta: f64,
        j1: f64,
        rng: &mut ThreadRng,
    ) -> f64 {
        self.diagonal_update(beta, j1, rng);
        self.off_diagonal_update(nloop, rng);
        // let si = self.si_operator(weights);
        //((self.n1 + self.n2) as f64, si)
        self.n as f64
    }
    pub fn sample_avg(
        &mut self,
        n: usize,
        // weights: &Vec<f64>,
        nloop: usize,
        beta: f64,
        j1: f64,
        rng: &mut ThreadRng,
    ) -> f64 {
        let mut samples = Vec::new();
        for _ in 0..n {
            samples.push(self.sample(nloop, beta, j1, rng));
        }
        let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
        //let si_mean: f64 = samples.iter().map(|x| x.1).sum::<f64>() / samples.len() as f64;
        mean
    }
    pub fn new(latice: &Latice, m: usize, rng: &mut ThreadRng) -> State {
        let mut alpha_even = Vec::new();
        let mut alpha_odd = Vec::new();
        for _ in 0..latice.num_even {
            alpha_even.push(rng.gen());
        }
        for _ in 0..latice.num_odd {
            alpha_odd.push(rng.gen());
        }
        let path = vec![None; m];
        let s = State {
            alpha: Bipartite_Vector {
                even: alpha_even,
                odd: alpha_odd,
            },
            path,
            n: 0,
            last: Bipartite_Vector {
                even: vec![None; latice.num_even],
                odd: vec![None; latice.num_odd],
            },
            latice: latice.clone(),
        };
        s.verify();
        s
    }
}
