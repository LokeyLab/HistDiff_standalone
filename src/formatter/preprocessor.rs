#![allow(unused_imports, dead_code)]
use super::utils::*;
use csv;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn preprocess_data<P1, P2>(
    input_file: P1,
    output_file: P2,
    id_col: &[String],
    useless_feats: &[String],
) -> Result<(), Box<dyn Error>>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let infile = File::open(&input_file)?;
    let reader = BufReader::new(infile);

    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_reader(reader);

    let headers = csv_reader.headers()?.clone();

    return Ok(());
}
