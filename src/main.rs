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
    let latice = make_latice(2, 2);
    let rng = &mut rand::thread_rng();

    let beta = 16.0;

    let js = range(0.0, 2.0, 20);
    let mut energy = Vec::new();
    let mut deltaE = Vec::new();
    let mut sm = Vec::new();
    let mut deltaSM = Vec::new();

    for j1 in js.clone() {
        let mut energys = Vec::new();
        let mut sms = Vec::new();

        let mut s = State::new(&latice, 10, rng);
        s.thermalize(beta, j1, rng);
        for _ in 0..1000000 {
            let (energy, sm) = s.sample(40, beta, j1, rng);
            energys.push(energy);
            sms.push(sm);
        }
        let energys = stats::bin(&energys, 1000);
        let sms = stats::bin(&sms, 1000);
        let (mean_energy, sd_energy) = stats::bootstrap(&energys, 1000);
        let (mean_sm, sd_sm) = stats::bootstrap(&sms, 1000);
        energy.push(mean_energy);
        deltaE.push(sd_energy);
        sm.push(mean_sm);
        deltaSM.push(sd_sm);
    }
    write_csv("data.csv", &vec![&js, &energy, &deltaE, &sm, &deltaSM]);
}
