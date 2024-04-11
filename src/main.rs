mod lattice;
mod state;
mod stats;

use crate::lattice::{new_rectangle, EdgeType, Lattice};
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

pub fn make_latice(width: usize, height: usize) -> Lattice {
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
    let rng = &mut rand::thread_rng();

    let question_8 = true;
    let question_9 = true;
    let question_10 = true;

    // //question8
    if question_8 {
        println!("Question 8");

        let latice = make_latice(4, 4);
        let mut s = State::new(&latice, 10, rng);
        let beta = 16.0;
        let j1 = 1.0;

        let mut ns = Vec::new();
        let mut ms = Vec::new();

        for _ in 0..100000 {
            s.diagonal_update(beta, j1, rng);
            s.off_diagonal_update(40, rng);
            while s.path.len() < s.n * 10 / 8 {
                _ = s.path.push(None);
            }
            ns.push(s.n as f64);
            ms.push(s.path.len() as f64);
        }
        write_csv("question8.csv", &vec![ns, ms]);
    }
    // //question 9
    if question_9 {
        println!("Question 9");

        let lattice = make_latice(2, 2);

        let betas = vec![1.0, 2.0, 4.0, 8.0, 16.0];
        let js = range(0.0, 2.0, 20);

        let mut energies = Vec::new();
        for _ in 0..11 {
            energies.push(Vec::new());
        }
        for (b_idx, beta) in betas.iter().enumerate() {
            for j in js.iter() {
                let mut samples = Vec::new();
                let mut s = State::new(&lattice, 10, rng);
                s.thermalize(*beta, *j, rng);
                for _ in 0..10000 {
                    let (energy, sm) = s.sample(40, *beta, *j, rng);
                    samples.push(energy);
                }
                let samples = stats::bin(&samples, 100);
                let (mean, sd) = stats::bootstrap(&samples, 1000);
                energies[2 * b_idx + 1].push(mean);
                energies[2 * b_idx + 2].push(sd);
            }
        }
        energies[0] = js.clone();
        write_csv("question9.csv", &energies);
    }
    //question 10
    if question_10 {
        println!("Question 10");

        let Ls = vec![2, 4, 6, 8];
        let js = range(0.0, 2.0, 20);
        let mut mags = Vec::new();
        for _ in 0..Ls.len() * 2 + 1 {
            mags.push(Vec::new());
        }
        for (L_idx, L) in Ls.iter().enumerate() {
            let lattice = make_latice(*L, *L);
            let beta = *L as f64 * 8.0;
            for j in js.iter() {
                let mut samples = Vec::new();
                let mut s = State::new(&lattice, 10, rng);
                s.thermalize(beta, *j, rng);
                for _ in 0..10000 {
                    let (energy, sm) = s.sample(40, beta, *j, rng);
                    samples.push(sm);
                }
                samples = stats::bin(&samples, 100);
                let (mean, sd) = stats::bootstrap(&samples, 1000);
                mags[2 * L_idx + 1].push(mean);
                mags[2 * L_idx + 2].push(sd);
            }
            println!("L = {} Done", L)
        }
        mags[0] = js.clone();
        write_csv("question10.csv", &mags);
    }
}
