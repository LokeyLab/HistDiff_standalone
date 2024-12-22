#![allow(unused_imports)]
use clap::Parser;
use csv::ReaderBuilder;
use std::collections::{HashMap, HashSet};
use std::{error::Error, fs::File, io::BufReader, path::Path};
use HistDiff_standalone::{find_common_features, integrity_check, preprocess_data};

fn main() -> Result<(), Box<dyn Error>> {
    let file = "/home/derfelt/git_repos/HistDiff_standalone/temp_store/signals/d0a5160e-9544-11ee-ac86-02420a000112_cellbycell.tsv";
    let integrity_out = Some(
        "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/rust_integrity.txt",
    );
    let output_path = Some(
        "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/final_rust_format.tsv",
    );

    let in_file = File::open(file)?;
    let reader = BufReader::new(in_file);
    let mut csv_reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_reader(reader);

    let headers = csv_reader
        .headers()?
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let meta_cols = vec![
        "ScreenName",
        "ScreenID",
        "PlateName",
        "PlateID",
        "MeasurementDate",
        "MeasurementID",
        "WellName",
        "Row",
        "Column",
        "Timepoint",
        "Field",
        "RefId",
        "Object Number",
        "X",
        "Y",
        "Bounding Box",
        "ax",
        "ay",
        "Cell Count",
        "Cell ID",
        "Instance",
        "Laser focus score",
        "Plate ID",
        "Run Settings ID",
        "Series ID",
        "Site ID",
        "Well Name",
        "Well X",
        "Well Y",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect::<Vec<String>>();

    let header_len = headers.len();
    let common_meta = find_common_features(&headers, &meta_cols);

    let id_col = if common_meta.contains(&"WellName".to_string()) {
        vec!["WellName".to_string()]
    } else {
        vec!["Well Name".to_string()]
    };

    let useless_feats = common_meta
        .iter()
        .filter(|c| !id_col.contains(c))
        .cloned()
        .collect::<Vec<_>>();

    if let Some(fp) = integrity_out {
        let (nrows_before, nrows_after) = integrity_check(file, fp, 10250)?;

        println!("{:?} {:?}", nrows_before, nrows_after);
    }

    if let Some(fp) = output_path {
        let _ = preprocess_data(
            if integrity_out.is_none() {
                file
            } else {
                integrity_out.as_ref().unwrap()
            },
            fp,
            &id_col,
            &useless_feats,
        );
    }

    return Ok(());
}
