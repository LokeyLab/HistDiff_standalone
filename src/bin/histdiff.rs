use std::{error::Error, fs::File};

use clap::*;
use csv::ReaderBuilder;
use HistDiff_standalone::*;

#[derive(Parser, Debug)]
#[command(version, about = "Calculates HistDiff scores for cell data", long_about = None, name = "HistDiff (rust edition)")]
struct Cli {
    #[arg(
        short = 'i',
        long,
        help = "path to cell by cell data (as a tab delimited file)"
    )]
    cell_path: String,

    #[arg(
        short,
        long,
        help = "output path of final HistDiff scores (must end in .csv)"
    )]
    output_path: String,

    #[arg(
        short,
        long,
        help = "Path to platemap that contains info on the wells and whether or not they are reference wells (must be a .csv file)"
    )]
    controls_file_path: String,

    #[arg(
        short,
        long,
        help = "Specify the name of the column where the REFERENCE labels are located (note: this column must contain cells with the label: REFERENCE as this specifies which wells are the reference controls for HistDiff)"
    )]
    reference_column: String,

    #[arg(
        short = 'd',
        long,
        help = "Specify the name of the index column of the cell by cell data file (This is under the assumption that the rest of the columns are numerical features. Note: program will break if there is additional meta columns not taken care of.)"
    )]
    index_column: String,

    #[arg(
        short,
        long,
        help = "Specify the name of the column containing the well locations"
    )]
    well_location: String,

    #[arg(long, help = "Verbose output to std out", action = ArgAction::SetTrue)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // let test_platemap =
    //     "/home/derfelt/git_repos/HistDiff_standalone/temp_store/platemaps/SP7238PMA.csv";
    // let cell_data = "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/024ebc52-9579-11ef-b032-02420a00010f_cellbycell_HD_input.tsv";
    //
    // let id_col = vec!["id".to_string()];
    // let nbins: usize = 20;
    // let sample_type_col = "sample_type";
    // let well_col = "384_Well";

    let platemap_path = &cli.controls_file_path;
    let cell_data = &cli.cell_path;
    let id_col = vec![cli.index_column.to_string()];
    let nbins: usize = 20;
    let reference_column = &cli.reference_column;
    let well_location = &cli.well_location;

    let platemap_file = File::open(platemap_path)?;
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
        .position(|h| h == reference_column)
        .ok_or("sample type column not found")?;
    let well_col_idx = pm_headers
        .iter()
        .position(|h| h == well_location)
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
        cell_data,
        &id_col,
        &controls,
        nbins,
        None,
        cli.verbose,
        None,
        plate_def,
    )?;

    // println!("{:?}", hd_result?.len());

    let _ = write_csv(&hd_result, &cli.output_path);

    return Ok(());
}
