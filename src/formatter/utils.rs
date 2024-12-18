#![allow(unused_imports, dead_code)]
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
