mod latice;
mod new_rep;
mod state;
mod stats;

use crate::latice::{new_rectangle, EdgeType, Latice};
use crate::state::State;
use crate::stats::write_csv;
use rand::{rngs::ThreadRng, Rng};
use std::time::Instant;

pub fn range(min: f64, max: f64, n: usize) -> Vec<f64> {
    let mut v = Vec::new();
    let step = (max - min) / n as f64;
    for i in 0..n {
        v.push(min + i as f64 * step);
    }
    v
}

pub fn make_latice(width: usize, height: usize) -> Latice {
    let mut constructor = new_rectangle(width, height);
    for x in 0..width {
        for y in 0..height {
            let (x2, y2) = ((x + 1) % width, y);
            constructor.add_edge((x, y), (x2, y2), EdgeType::Two);
            let (x2, y2) = (x, (y + 1) % height);
            constructor.add_edge(
                (x, y),
                (x2, y2),
                if y % 2 == 0 {
                    EdgeType::One
                } else {
                    EdgeType::Two
                },
            );
        }
    }
    constructor.build()
}
fn main() {
    let latice = make_latice(2, 2);
    let rng = &mut rand::thread_rng();

    let beta = 16.0;

    // let mut m_data = Vec::new();
    // let mut n_data = Vec::new();
    // let mut touched = Vec::new();
    // let mut steps = Vec::new();

    // for i in 0..20000 {
    //     s.diagonal_update(&latice, beta, 0.5, rng);
    //     touched.push(s.off_diagonal_update(40, rng) as f64);
    //     let n = s.n1 + s.n2;
    //     while n > s.path.len() * 9 / 10 {
    //         s.path.push(None);
    //     }
    //     n_data.push(n as f64);
    //     m_data.push(s.path.len() as f64);
    //     steps.push(i as f64);
    // }
    // write_csv("data.csv", &steps, &m_data);
    // write_csv("data2.csv", &steps, &touched);
    //let j1 = 1.0;
    //let weights = latice.staggered_magnetization_weights();
    //let nloop = s.thermalize(&latice, beta, 0.0, rng);

    // let js = range(0.0, 2.0, 20);
    // let mut y_data = Vec::new();
    // for j1 in js.clone() {
    //     let mut s = State::new(&latice, 10, rng);
    //     let nloop = s.thermalize(&latice, beta, j1, rng);
    //     let (mean_n, mag) = s.sample_avg(1000, &weights, 40, &latice, beta, j1, rng);
    //     let energy = -mean_n / beta + latice.edges.len() as f64 / 4.0;
    //     y_data.push(energy);
    // }
    println!("start");
    //println!("latice: {:?}", latice.edges);
    let mut s = State::new(&latice, 4, rng);
    //s.thermalize(beta, j1, rng);
    for _ in 0..200000 {
        println!("s_before: {:?}", s);
        s.diagonal_update(beta, 0.5, rng);
        //println!("n: {}", s.n);
        println!("s: {:?}", s);
        s.verify();
        // s.off_diagonal_update(40, rng);
        // s.verify();
    }

    // let js = range(0.0, 2.0, 20);
    // let mut y_data = Vec::new();
    // for j1 in js.clone() {
    //     let mut measurments = Vec::new();
    //     let mut s = State::new(&latice, 10, rng);
    //     //s.thermalize(beta, j1, rng);
    //     for _ in 0..200000 {
    //         s.diagonal_update(beta, j1, rng);
    //         s.verify();
    //         // s.off_diagonal_update(40, rng);
    //         // s.verify();
    //     }
    //     println!("thermalized");
    //     for _ in 0..10000 {
    //         s.diagonal_update(beta, j1, rng);
    //         s.off_diagonal_update(40, rng);
    //         measurments.push(s.n as f64);
    //     }
    //     let mean: f64 = measurments.iter().sum::<f64>() as f64 / measurments.len() as f64;
    //     let n_bonds_1 = latice
    //         .edges
    //         .iter()
    //         .filter(|e| e.edge_type == EdgeType::One)
    //         .count() as f64;
    //     let n_bonds_2 = latice.edges.len() as f64 - n_bonds_1;
    //     let energy = -mean / beta + j1 * n_bonds_1 as f64 / 4.0 + n_bonds_2 as f64 / 4.0;
    //     y_data.push(energy);
    //     println!("energy: {}", energy);
    // }
    // write_csv("data.csv", &js, &y_data);
}
