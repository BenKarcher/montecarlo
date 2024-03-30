use rand::{rngs::ThreadRng, Rng};
use std::time::Instant;
use std::{mem::swap, vec};

fn write_csv(filename: &str, x_data: &Vec<f64>, y_data: &Vec<f64>) {
    let mut wtr = csv::Writer::from_path(filename).unwrap();
    for (x, y) in x_data.iter().zip(y_data.iter()) {
        wtr.write_record(&[x.to_string(), y.to_string()]).unwrap();
    }
    wtr.flush().unwrap();
}

fn calc_mean(samples: &[u128]) -> f64 {
    samples.iter().sum::<u128>() as f64 / samples.len() as f64
}

fn calc_sd(samples: &[u128]) -> f64 {
    let m = calc_mean(samples);
    let v = samples.iter().map(|x| (*x as f64 - m).powi(2)).sum::<f64>() / samples.len() as f64;
    v.sqrt()
}
pub fn range(min: f64, max: f64, n: usize) -> Vec<f64> {
    let mut v = Vec::new();
    let step = (max - min) / n as f64;
    for i in 0..n {
        v.push(min + i as f64 * step);
    }
    v
}
fn main() {
    let latice = Latice::new(4, 2);
    let rng = &mut rand::thread_rng();
    let mut s = State::new(&latice, 10, rng);
    let beta = 16.0;
    //let j1 = 1.0;
    //let weights = latice.staggered_magnetization_weights();
    let nloop = s.thermalize(&latice, beta, 0.0, rng);

    // let js = range(0.0, 2.0, 20);
    // let mut y_data = Vec::new();
    // for j1 in js.clone() {
    //     let nloop = s.thermalize(&latice, beta, j1, rng);
    //     let (mean_n, mag) = s.sample_avg(1000, &weights, nloop, &latice, beta, j1, rng);
    //     let energy = -mean_n / beta + latice.edges.len() as f64 / 4.0;
    //     y_data.push(energy);
    // }
    let js = range(0.0, 2.0, 20);
    let mut y_data = Vec::new();
    for j1 in js.clone() {
        let mut measurments = Vec::new();
        for _ in 0..10000 {
            s.diagonal_update(&latice, beta, j1, rng);
            s.off_diagonal_update(nloop, rng);
            measurments.push(s.n1 + s.n2);
        }
        let mean: f64 = measurments.iter().sum::<usize>() as f64 / measurments.len() as f64;
        let energy = -mean / beta + latice.edges.len() as f64 / 4.0;
        y_data.push(energy);
        println!("energy: {}", energy);
    }
    write_csv("data.csv", &js, &y_data);
}

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
                weights[id] = if (y) % 2 == 0 { 1.0 } else { -1.0 };
            }
        }
        weights
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
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

#[derive(Clone)]
pub struct Operator {
    pub operator_type: OperatorType,
    pub even: usize,
    pub odd: usize,
    pub even_in_id: usize,
    pub odd_in_id: usize,
    pub even_out_id: usize,
    pub odd_out_id: usize,
    pub edge_type: EdgeType,
}

#[derive(Clone)]
pub struct State {
    pub alpha: Vec<bool>,
    pub path: Vec<Option<Operator>>,
    pub n1: usize,
    pub n2: usize,
}

impl State {
    pub fn trace(&self, a: usize, idx: usize, dir: bool) -> usize {
        for d in 1..self.path.len() {
            let idx2 = if dir {
                (idx + d) % self.path.len()
            } else {
                (idx + self.path.len() - d) % self.path.len()
            };
            if let Some(ref op) = self.path[idx2] {
                if a % 2 == 0 {
                    if op.even == a {
                        return idx2;
                    }
                } else {
                    if op.odd == a {
                        return idx2;
                    }
                }
            }
        }
        idx
    }
    pub fn insert_diag(&mut self, even: usize, odd: usize, idx: usize, edge_type: EdgeType) {
        assert!(self.path[idx].is_none(), "idx must be empty");
        match edge_type {
            EdgeType::One => self.n1 += 1,
            EdgeType::Two => self.n2 += 1,
        }
        let op = Operator {
            operator_type: OperatorType::D,
            even,
            odd,
            even_in_id: self.trace(even, idx, false),
            odd_in_id: self.trace(odd, idx, false),
            even_out_id: self.trace(even, idx, true),
            odd_out_id: self.trace(odd, idx, true),
            edge_type: edge_type,
        };
        if op.even_in_id != idx {
            self.path[op.even_in_id].as_mut().unwrap().even_out_id = idx;
        }
        if op.odd_in_id != idx {
            self.path[op.odd_in_id].as_mut().unwrap().odd_out_id = idx;
        }
        if op.even_out_id != idx {
            self.path[op.even_out_id].as_mut().unwrap().even_in_id = idx;
        }
        if op.odd_out_id != idx {
            self.path[op.odd_out_id].as_mut().unwrap().odd_in_id = idx;
        }
        self.path[idx] = Some(op);
    }
    pub fn delete(&mut self, idx: usize) {
        let op = self.path[idx].take().unwrap();
        match op.edge_type {
            EdgeType::One => self.n1 -= 1,
            EdgeType::Two => self.n2 -= 1,
        }
        if op.even_in_id != idx {
            self.path[op.even_in_id].as_mut().unwrap().even_out_id = op.even_out_id;
        }
        if op.odd_in_id != idx {
            self.path[op.odd_in_id].as_mut().unwrap().odd_out_id = op.odd_out_id;
        }
        if op.even_out_id != idx {
            self.path[op.even_out_id].as_mut().unwrap().even_in_id = op.even_in_id;
        }
        if op.odd_out_id != idx {
            self.path[op.odd_out_id].as_mut().unwrap().odd_in_id = op.odd_in_id;
        }
    }
    pub fn verify(&self) {
        //check that every operator has even evens and odd odds
        for op in self.path.iter() {
            if let Some(op) = op {
                assert_eq!(op.even % 2, 0, "even must be even");
                assert_eq!(op.odd % 2, 1, "odd must be odd");
            }
        }
        //check that every connection is bidirectional
        for (i, op) in self.path.iter().enumerate() {
            if let Some(op) = op {
                assert_eq!(
                    self.path[op.even_in_id].as_ref().unwrap().even_out_id,
                    i,
                    "even_in_id error"
                );
                assert_eq!(
                    self.path[op.odd_in_id].as_ref().unwrap().odd_out_id,
                    i,
                    "odd_in_id error"
                );
                assert_eq!(
                    self.path[op.even_out_id].as_ref().unwrap().even_in_id,
                    i,
                    "even_out_id error"
                );
                assert_eq!(
                    self.path[op.odd_out_id].as_ref().unwrap().odd_in_id,
                    i,
                    "odd_out_id error"
                );
            }
        }
        //check that every connetion has no other operators interupting it in the same site
        for (idx, op) in self.path.iter().enumerate() {
            if let Some(op) = op {
                assert_eq!(
                    self.trace(op.even, idx, true),
                    op.even_out_id,
                    "even out trace error"
                );
                assert_eq!(
                    self.trace(op.odd, idx, true),
                    op.odd_out_id,
                    "odd out trace error"
                );
                assert_eq!(
                    self.trace(op.even, idx, false),
                    op.even_in_id,
                    "even in trace error"
                );
                assert_eq!(
                    self.trace(op.odd, idx, false),
                    op.odd_in_id,
                    "odd in trace error"
                );
            }
        }
        //check that operators sit on oposing spins and that alpha loops
        let mut current = self.alpha.clone();
        for (i, op) in self.path.iter().enumerate() {
            if let Some(op) = op {
                assert_eq!(
                    current[op.even], !current[op.odd],
                    "op {} not oposing spins",
                    i
                );
                if op.operator_type == OperatorType::OD {
                    current[op.even] ^= true;
                    current[op.odd] ^= true;
                }
            }
        }
        assert_eq!(current, self.alpha, "alpha not a loop");
        //check that counts of each edge type are correct
        let mut n1 = 0;
        let mut n2 = 0;
        for op in self.path.iter() {
            if let Some(op) = op {
                match op.edge_type {
                    EdgeType::One => n1 += 1,
                    EdgeType::Two => n2 += 1,
                }
            }
        }
        assert_eq!(n1, self.n1, "n1 count error");
        assert_eq!(n2, self.n2, "n2 count error");
    }
    pub fn directed_loop_update(&mut self, start: usize) -> usize {
        let mut idx = start;
        let mut len = 0;
        loop {
            //going up
            let next = self.path[idx].as_ref().unwrap().even_out_id;
            if next <= idx {
                self.alpha[self.path[idx].as_ref().unwrap().even] ^= true;
            }
            idx = next;

            self.path[idx].as_mut().unwrap().operator_type.flip();

            //going down
            let next = self.path[idx].as_ref().unwrap().odd_in_id;
            if next >= idx {
                self.alpha[self.path[idx].as_ref().unwrap().odd] ^= true;
            }
            idx = next;

            self.path[idx].as_mut().unwrap().operator_type.flip();
            len += 1;
            if next == start {
                return len;
            }
        }
    }
    pub fn diagonal_update(&mut self, latice: &Latice, beta: f64, j1: f64, rng: &mut ThreadRng) {
        let mut current = self.alpha.clone();
        for i in 0..self.path.len() {
            let op = &self.path[i];
            match op {
                Some(Operator {
                    operator_type: OperatorType::OD,
                    even,
                    odd,
                    ..
                }) => {
                    current[*even] ^= true;
                    current[*odd] ^= true;
                }
                Some(Operator {
                    operator_type: OperatorType::D,
                    edge_type,
                    ..
                }) => {
                    //check if removal is accepted
                    let p = (self.path.len() - self.n1 - self.n2 + 1) as f64
                        / (latice.edges.len() as f64
                            * beta
                            * 0.5
                            * match edge_type {
                                EdgeType::One => j1,
                                EdgeType::Two => 1.0,
                            });
                    if rng.gen::<f64>() < p {
                        //if 1.0 < p {
                        self.delete(i);
                    }
                }
                None => {
                    //check if insertion is accepted
                    let edge = latice.random_edge(rng);
                    if current[edge.even] == current[edge.odd] {
                        continue;
                    }
                    let p = (latice.edges.len() as f64
                        * beta
                        * 0.5
                        * match edge.edge_type {
                            EdgeType::One => j1,
                            EdgeType::Two => 1.0,
                        })
                        / (self.path.len() - self.n1 - self.n2) as f64;
                    if rng.gen::<f64>() < p {
                        //if 0.0 < p {
                        self.insert_diag(edge.even, edge.odd, i, edge.edge_type);
                    }
                }
            };
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
            return 0;
        }
        for _ in 0..nloop {
            let idx = idxs[rng.gen_range(0..idxs.len())];
            count += self.directed_loop_update(idx);
        }
        count
    }

    pub fn thermalize(
        &mut self,
        latice: &Latice,
        beta: f64,
        j1: f64,
        rng: &mut ThreadRng,
    ) -> usize {
        let mut nloop = 10;
        let mut plato = 0;
        let mut touched = 0;
        loop {
            self.diagonal_update(latice, beta, j1, rng);
            touched += self.off_diagonal_update(nloop, rng);
            if self.n1 + self.n2 > self.path.len() * 8 / 10 {
                for _ in 0..self.path.len() * 2 / 10 {
                    self.path.push(None);
                }
                nloop = self.path.len() * nloop * plato / touched;
                if nloop == 0 {
                    nloop = 1;
                }
                plato = 0;
            }
            //self.verify();
            if plato == 10000 {
                nloop = self.path.len() * nloop * plato / touched;
                if nloop == 0 {
                    nloop = 1;
                }
                return nloop;
            }
            plato += 1;
        }
    }
    pub fn si_operator(&self, weights: &Vec<f64>) -> f64 {
        let mut sum = 0.0;
        assert_eq!(
            weights.len(),
            self.alpha.len(),
            "weights must have the same length as alpha"
        );
        for (w, a) in weights.iter().zip(self.alpha.iter()) {
            sum += w * (if *a { 0.5 } else { -0.5 });
        }
        sum.abs()
    }
    pub fn sample(
        &mut self,
        weights: &Vec<f64>,
        nloop: usize,
        latice: &Latice,
        beta: f64,
        j1: f64,
        rng: &mut ThreadRng,
    ) -> (f64, f64) {
        self.diagonal_update(latice, beta, j1, rng);
        self.off_diagonal_update(nloop, rng);
        let si = self.si_operator(weights);
        ((self.n1 + self.n2) as f64, si)
    }
    pub fn sample_avg(
        &mut self,
        n: usize,
        weights: &Vec<f64>,
        nloop: usize,
        latice: &Latice,
        beta: f64,
        j1: f64,
        rng: &mut ThreadRng,
    ) -> (f64, f64) {
        let mut samples = Vec::new();
        for _ in 0..n {
            samples.push(self.sample(weights, nloop, latice, beta, j1, rng));
        }
        let mean: f64 = samples.iter().map(|x| x.0).sum::<f64>() / samples.len() as f64;
        let si_mean: f64 = samples.iter().map(|x| x.1).sum::<f64>() / samples.len() as f64;
        (mean, si_mean)
    }
    pub fn new(latice: &Latice, m: usize, rng: &mut ThreadRng) -> State {
        let mut alpha = Vec::new();
        for _ in 0..latice.width * latice.height {
            alpha.push(rng.gen());
        }
        let path = vec![None; m];
        let s = State {
            alpha,
            path,
            n1: 0,
            n2: 0,
        };
        s.verify();
        s
    }
}
