#![allow(unused_imports, dead_code)]
use super::histograms::*;
use super::utils::*;
use csv;
use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::usize;

pub fn calculate_scores<P: AsRef<Path>>(
    cell_data: P,
    id_cols: &[String],
    vehicle_cntrls: &[String],
    nbins: usize,
    prob_out: Option<&str>,
    verbose: bool,
    block_def: Option<Vec<Vec<String>>>,
    plate_def: Option<Vec<String>>,
) -> Result<(), Box<dyn Error>> {
    let plate_def = match plate_def {
        Some(definition) => definition,
        None => plate_definition(),
    };

    let min_max = get_min_max_plate(&cell_data, id_cols, verbose, prob_out)?;
    let min_max_map = min_max.min_max;
    let features = min_max.features;
    // println!("{:?}", features.len());

    let file = File::open(&cell_data)?;
    let reader = BufReader::new(file);

    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_reader(reader);

    let headers = csv_reader.headers()?.clone();

    let id_col_idx: Vec<usize> = id_cols
        .iter()
        .map(|col| headers.iter().position(|h| h == col))
        .collect::<Option<Vec<_>>>()
        .ok_or("ID column not found in headers")?;

    let feat_idx: Vec<usize> = features
        .iter()
        .filter_map(|feats| headers.iter().position(|h| h == feats))
        .collect();

    // the hisogram structure is as follows:
    // histograms = {
    //  well_id: {
    //      feature: Histogram Struct
    //      }
    // }
    let mut histograms: DashMap<String, DashMap<String, Hist1D>> = DashMap::new();

    for res in csv_reader.records() {
        let record = res?;

        // in the event of multiple columns being the "id column"
        // note: if the id column is multiple columns then they
        // must be specified in the plate definition parameters
        // with "_"s separating each value of each columns
        let curr_well = id_cols
            .iter()
            .map(|id_feat| {
                record
                    .get(headers.iter().position(|h| h == id_feat).unwrap())
                    .unwrap()
            })
            .collect::<Vec<&str>>()
            .join("_");

        if !plate_def.contains(&curr_well) {
            continue;
        }
    }

    Ok(())
}

#[cfg(test)]
mod hd_test {
    use super::*;

    #[test]
    fn test_hd() {
        let fp = "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/024ebc52-9579-11ef-b032-02420a00010f_cellbycell_HD_input.tsv";
        let id_cols = vec!["id".to_string()];
        let vehicle_cntrls = vec!["A1".to_string(), "P24".to_string()];
        let nbins = 20;

        let _ = calculate_scores(fp, &id_cols, &vehicle_cntrls, nbins, None, true, None, None);
    }
}
