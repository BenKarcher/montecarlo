use std::mem::swap;

fn main() {
    println!("Hello, world!");
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum EdgeType {
    One,
    Two,
}

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
    pub fn node_id(&self, x: usize, y: usize) -> usize {
        assert!(
            x < self.width && y < self.height,
            "x and y must be less than width and height"
        );
        y * self.width + x
    }
    pub fn node_pos(&self, id: usize) -> (usize, usize) {
        assert!(
            id < self.width * self.height,
            "id must be less than width*height"
        );
        (id % self.width, id / self.width)
    }
    fn add_edge(&mut self, edge_type: EdgeType, mut a: usize, mut b: usize) {
        if b % 2 == 0 {
            swap(&mut a, &mut b);
        }
        assert_eq!(a % 2, 0, "a must be even");
        assert_eq!(b % 2, 1, "b must be odd");
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
                    EdgeType::One,
                    l.node_id(i, j),
                    l.node_id((i + 1) % width, j),
                );
                l.add_edge(
                    if j % 2 == 0 {
                        EdgeType::Two
                    } else {
                        EdgeType::One
                    },
                    l.node_id(i, j),
                    l.node_id(i, (j + 1) % height),
                );
            }
        }
        l
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

pub struct Operator {
    pub operator_type: OperatorType,
    pub even: usize,
    pub odd: usize,
    pub even_in_id: usize,
    pub odd_in_id: usize,
    pub even_out_id: usize,
    pub odd_out_id: usize,
}

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
        a
    }
    pub fn insert_diag(&mut self, even: usize, odd: usize, idx: usize) {
        assert!(self.path[idx].is_none(), "idx must be empty");
        let op = Operator {
            operator_type: OperatorType::D,
            even,
            odd,
            even_in_id: self.trace(even, idx, false),
            odd_in_id: self.trace(odd, idx, false),
            even_out_id: self.trace(even, idx, true),
            odd_out_id: self.trace(odd, idx, true),
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
}
