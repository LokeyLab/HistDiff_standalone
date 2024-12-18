#![allow(unused_imports, dead_code)]
use std::collections::HashSet;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn integrity_check<P1, P2>(
    infile: P1,
    outfile: P2,
    buff_size: usize,
) -> Result<(usize, usize), Box<dyn Error>>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let mut rows_before = 0 as usize;
    let mut rows_after = 0 as usize;
    let mut buffer: Vec<String> = Vec::with_capacity(buff_size);

    let input_file = File::open(&infile)?;
    let mut out_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&outfile)?;
    let mut reader = BufReader::new(input_file);

    let mut header_line = String::new();
    let bytes_read = reader.read_line(&mut header_line)?;
    if bytes_read == 0 {
        return Ok((rows_before, rows_after));
    }

    let header_len = header_line.trim().split('\t').count();

    let _ = out_file.write_all(header_line.as_bytes());

    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        rows_before += 1;
        let row: Vec<&str> = line.trim().split('\t').collect();
        let threshold = 0.65;

        if row.len() == header_len && majority_float(&row, threshold) {
            buffer.push(line.clone());
            rows_after += 1;

            if buffer.len() >= buff_size {
                for buf_line in &buffer {
                    let _ = out_file.write_all(buf_line.as_bytes());
                }
                buffer.clear();
            }
        }

        line.clear();
    }

    return Ok((rows_before, rows_after));
}

fn majority_float(row: &[&str], threshold: f64) -> bool {
    let total = row.len() as f64;
    let min_n_floats = (threshold * total) as i32;

    let float_count: i32 = row
        .iter()
        .filter(|col| col.parse::<f64>().is_ok())
        .count()
        .try_into()
        .unwrap();

    return float_count >= min_n_floats;
}

pub fn find_common_features(true_feats: &[String], bad_feats: &[String]) -> Vec<String> {
    let set_true_feats: HashSet<&str> = true_feats.iter().map(|s| s.as_str()).collect();
    let set_bad_feats: HashSet<&str> = bad_feats.iter().map(|s| s.as_str()).collect();

    let mut common_feats: Vec<String> = set_true_feats
        .intersection(&set_bad_feats)
        .map(|f| f.to_string())
        .collect();

    for feat in true_feats {
        let cond1 = bad_feats
            .iter()
            .filter(|i| i.len() > 2)
            .any(|i| feat.contains(i));
        let cond2 = bad_feats.iter().filter(|i| i.len() <= 2).any(|i| i == feat);

        if (cond1 || cond2) && !common_feats.contains(feat) {
            common_feats.push(feat.clone());
        }
    }

    return common_feats;
}
