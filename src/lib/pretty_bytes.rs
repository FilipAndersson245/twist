use std::cmp;

pub fn convert(num: u64) -> String {
    let units = ["B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

    let delimiter = 1000_f64;
    let exponent = cmp::min(
        ((num as f64).ln() / delimiter.ln()).floor() as i32,
        (units.len() - 1) as i32,
    );
    let pretty_bytes = (num as f64 / delimiter.powi(exponent)) * 1_f64;
    let unit = units[exponent as usize];
    format!("{:.2} {}", pretty_bytes, unit)
}
