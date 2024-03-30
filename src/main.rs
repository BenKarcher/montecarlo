mod latice;
mod state;

use crate::latice::{EdgeType, Latice};
use crate::state::State;
use rand::{rngs::ThreadRng, Rng};
use std::time::Instant;

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
