use std::{error::Error, fs::File};

use csv::ReaderBuilder;
use HistDiff_standalone::*;

fn main() -> Result<(), Box<dyn Error>> {
    let test_platemap =
        "/home/derfelt/git_repos/HistDiff_standalone/temp_store/platemaps/SP7238PMA.csv";
    let cell_data = "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/024ebc52-9579-11ef-b032-02420a00010f_cellbycell_HD_input.tsv";

    let id_col = vec!["id".to_string()];
    let nbins: usize = 20;
    let sample_type_col = "sample_type";
    let well_col = "384_Well";

    let platemap_file = File::open(test_platemap)?;
    let mut csv_reader = ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .from_reader(platemap_file);

    let pm_headers = csv_reader
        .headers()?
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let ref_col_idx = pm_headers
        .iter()
        .position(|h| h == sample_type_col)
        .ok_or("sample type column not found")?;
    let well_col_idx = pm_headers
        .iter()
        .position(|h| h == well_col)
        .ok_or("well column not found")?;

    let mut plate_def: Vec<String> = Vec::new();
    let mut controls: Vec<String> = Vec::new();

    for result in csv_reader.records() {
        let record = result?;

        let ref_val = record.get(ref_col_idx).unwrap().to_uppercase();
        let well_val = record.get(well_col_idx).unwrap().to_uppercase();

        plate_def.push(well_val.clone());

        if ref_val == "REFERENCE" {
            controls.push(well_val);
        }
    }

    let controls = clean_well_names(&controls);
    let plate_def = clean_well_names(&plate_def);

    let plate_def = if plate_def.len() == 384 {
        None
    } else {
        Some(plate_def)
    };

    let hd_result = calculate_scores(
        cell_data, &id_col, &controls, nbins, None, true, None, plate_def,
    );

    println!("{:?}", hd_result?.len());

    return Ok(());
}
