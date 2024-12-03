#![allow(unused_parens)]

use core::f64;
use std::{collections::HashMap, error::Error, path::Path, u8};

use csv::Writer;
pub fn exponential_smoothing(x: &[f64], alpha: f64) -> Vec<f64> {
    let n = x.len();
    let mut smoothing: Vec<f64> = Vec::with_capacity(n);

    for i in (0..n) {
        let x_i = x[i];
        let s_i = if i == 0 {
            if n > 1 {
                x_i + alpha * (x[i + 1] - x_i)
            } else {
                x_i
            }
        } else if (i == (n - 1)) {
            alpha * (x[i - 1] - x_i) + x_i
        } else {
            alpha * (x[i - 1] - x_i) + x_i + alpha * (x[i + 1] - x_i)
        };

        smoothing.push(s_i);
    }

    return smoothing;
}

pub fn normalize(x: &[f64]) -> Vec<f64> {
    let sum: f64 = x.iter().sum();
    if sum == 0.0 {
        return vec![0.0; x.len()];
    } else {
        return x.iter().map(|&e| e / sum).collect();
    }
}

pub fn clean_well_names(well_names: &[String]) -> Vec<String> {
    well_names
        .iter()
        .map(|name| {
            if name.len() >= 2 {
                let letter = &name[0..1];
                let number_str = &name[1..];

                match number_str.parse::<u32>() {
                    Ok(number) => format!("{}{}", letter, number),
                    Err(_) => name.clone(),
                }
            } else {
                name.clone()
            }
        })
        .collect()
}

pub fn plate_definition() -> Vec<String> {
    const WELL_384_LETTERS: std::ops::RangeInclusive<u8> = ('A' as u8)..=('P' as u8);
    const WELL_384_NUMBERS: std::ops::RangeInclusive<i32> = (1..=24);

    let mut res: Vec<String> = Vec::new();

    for letter in WELL_384_LETTERS {
        for num in WELL_384_NUMBERS {
            let format_string = format!("{}{}", letter as char, num);
            res.push(format_string);
        }
    }

    return res;
}

pub fn write_csv<P: AsRef<Path>>(
    hd_res: &HashMap<String, HashMap<String, f64>>,
    output_file: P,
) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(output_file)?;

    let mut feat_names = hd_res
        .values()
        .flat_map(|feat_map| feat_map.keys().cloned())
        .collect::<Vec<String>>();

    feat_names.sort();
    feat_names.dedup();

    // header row
    let mut header = vec!["id".to_string()];
    header.extend(feat_names.clone());
    let _ = writer.write_record(header);

    // write data
    for (well_id, feature_map) in hd_res {
        let mut row = vec![well_id.clone()];
        for feat in &feat_names {
            let value = feature_map.get(feat).unwrap_or(&f64::NAN);
            row.push(value.to_string());
        }
        let _ = writer.write_record(row);
    }

    // flush and close
    writer.flush()?;

    return Ok(());
}

#[cfg(test)]
mod utils_test {
    use super::*;

    #[test]
    fn test_smoothing_simple() {
        let test_in = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let alpha = 0.25;
        let smoothed = exponential_smoothing(&test_in, alpha);

        println!("{:?}", smoothed);

        let answer = vec![1.25, 2.0, 3.0, 4.0, 4.75];
        assert_eq!(smoothed, answer);
    }

    #[test]
    fn test_plate_def() {
        println!("{:?}", plate_definition());
        println!("{:?}", plate_definition().len());
    }
}
