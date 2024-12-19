#![allow(unused_imports, dead_code)]
use super::utils::*;
use csv::{self, StringRecord, WriterBuilder};
use std::collections::HashSet;
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

    let headers = csv_reader
        .headers()?
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let useless_set: HashSet<&str> = useless_feats.iter().map(|s| s.as_str()).collect();
    let id_set: HashSet<&str> = id_col.iter().map(|s| s.as_str()).collect();

    let mut final_cols: Vec<String> = headers
        .iter()
        .filter(|h| !useless_set.contains(h.as_str()) && id_set.contains(h.as_str()))
        .cloned()
        .collect();

    final_cols = final_cols.into_iter().map(|c| clean_col_name(&c)).collect();

    let mut final_headers = vec!["id".to_string()];
    final_headers.extend(final_cols.clone());

    // Map final_cols back to original columns
    // We need indices of original columns that correspond to final_cols.
    // We'll match by comparing cleaned original column names
    let keep_indices: Vec<usize> = final_cols
        .iter()
        .map(|c| {
            headers
                .iter()
                .position(|h| {
                    if useless_set.contains(h.as_str()) || id_set.contains(h.as_str()) {
                        false
                    } else {
                        clean_col_name(h) == *c
                    }
                })
                .expect("Could not map final column back to original")
        })
        .collect();

    let records: Vec<StringRecord> = csv_reader.records().collect::<Result<Vec<_>, _>>()?;

    let mut final_rows: Vec<Vec<String>> = Vec::with_capacity(records.len());
    for rec in records {
        let id_val = build_id(&rec, &headers, id_col);
        let renamed_id = rename_index(&id_val);

        let mut row = Vec::with_capacity(final_headers.len());
        row.push(renamed_id);

        for &idx in &keep_indices {
            let val = rec.get(idx).unwrap_or("");
            row.push(val.to_string());
        }

        final_rows.push(row);
    }

    let verification: Vec<_> = final_headers
        .iter()
        .filter(|f| useless_set.contains(f.as_str()))
        .collect();
    if !verification.is_empty() {
        eprintln!("Useless features still exist: {:?}", verification);
        return Err("Useless features present in final output".into());
    }

    // write the final oputput
    let mut wrtr = WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(output_file)?;

    wrtr.write_record(&final_headers)?;
    for row in final_rows {
        wrtr.write_record(&row)?;
    }

    let _ = wrtr.flush();
    return Ok(());
}

fn clean_col_name(col_name: &str) -> String {
    col_name
        .trim()
        .replace("\t", ",")
        .replace("%", "Pct")
        .replace(" - ", "-")
        .replace(" ", "_")
        .replace("µ", "u")
        .replace("²", "^2")
        .replace("_(RAWcells-CP2-Cyto_BMR)", "")
        .replace("_(RAWcells-CP2-EdU_BMR)", "")
}

fn rename_index(x: &str) -> String {
    if x.len() < 2 {
        return x.to_string();
    }
    let letter = &x[0..1];
    let number_str = &x[1..];
    match number_str.parse::<u32>() {
        Ok(num) => format!("{}{}", letter, num),
        Err(_) => x.to_string(),
    }
}

fn build_id(record: &StringRecord, headers: &[String], id_col: &[String]) -> String {
    if id_col.len() > 1 {
        let vals: Vec<String> = id_col
            .iter()
            .map(|col| {
                let idx = headers
                    .iter()
                    .position(|h| h == col)
                    .expect("ID column not found");
                record.get(idx).unwrap().to_string()
            })
            .collect();

        return vals.join("_");
    } else {
        let col = &id_col[0];
        let idx = headers
            .iter()
            .position(|h| h == col)
            .expect("id col not found");
        return record.get(idx).unwrap().to_string();
    }
}
