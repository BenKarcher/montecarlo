use rand::{rngs::ThreadRng, Rng};

pub fn calc_mean(samples: &Vec<f64>) -> f64 {
    samples.iter().sum::<f64>() / samples.len() as f64
}

pub fn calc_sd(samples: &Vec<f64>) -> f64 {
    let mean = calc_mean(samples);
    let variance = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / samples.len() as f64;
    variance.sqrt()
}

pub fn bin(data: &Vec<f64>, bin_size: usize) -> Vec<f64> {
    let mut binned = Vec::new();
    let mut sum = 0.0;
    for (i, &x) in data.iter().enumerate() {
        sum += x;
        if (i + 1) % bin_size == 0 {
            binned.push(sum / bin_size as f64);
            sum = 0.0;
        }
    }
    if data.len() % bin_size != 0 {
        binned.push(sum / (data.len() % bin_size) as f64);
    }
    binned
}

pub fn bootstrap(data: &Vec<f64>, n: usize) -> (f64, f64) {
    let mut rng = rand::thread_rng();
    let mut means = Vec::new();
    for _ in 0..n {
        let mut sum = 0.0;
        for _ in 0..data.len() {
            sum += data[rng.gen_range(0..data.len())];
        }
        means.push(sum / data.len() as f64);
    }
    let mean = calc_mean(&means);
    let sd = calc_sd(&means);
    (mean, sd)
}

pub fn write_csv(filename: &str, data: &Vec<Vec<f64>>) {
    let mut wtr = csv::Writer::from_path(filename).unwrap();
    //write the vecs in parallel
    for i in 0..data[0].len() {
        let mut record = Vec::new();
        for j in 0..data.len() {
            record.push(data[j][i].to_string());
        }
        wtr.write_record(record).unwrap();
    }
    wtr.flush().unwrap();
}
