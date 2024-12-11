#![allow(dead_code)]
use super::histograms::*;
use super::utils::*;
use csv;
use dashmap::DashMap;
use ndarray::Array1;
use ndarray::Array2;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;
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
) -> Result<HashMap<String, HashMap<String, f64>>, Box<dyn Error>> {
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
    // let mut histograms: HashMap<String, HashMap<String, Hist1D>> = HashMap::new();

    let start = Instant::now(); // WARN: you can delete this if you want

    // WARNING: PARALLEL VERSION BELOW

    // NOTE: So the below code isn't really multithreaded.
    // Why? Well this is because into_records() isn't producing
    // records fast enough for rayon to distrbute tasks amongst its workers
    // With that said, it rayon might kick in for smaller datasets
    let concurrent_histograms: DashMap<String, DashMap<String, Hist1D>> = DashMap::new();

    csv_reader.into_records().par_bridge().for_each(|res| {
        let record = match res {
            Ok(record) => record,
            Err(_) => return,
        };

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
            return;
        }

        let feature_values: Vec<(&str, f64)> = feat_idx
            .iter()
            .map(|&i| {
                let feat_name = &headers[i];
                let value = record.get(i).unwrap().parse::<f64>().unwrap_or(f64::NAN);
                (feat_name, value)
            })
            .collect();

        concurrent_histograms
            .entry(curr_well.clone())
            .or_insert_with(|| {
                let dh: DashMap<String, Hist1D> = DashMap::new();
                for feat in &features {
                    let (_, vals) = min_max_map.iter().find(|(f, _)| f == feat).unwrap();
                    dh.insert(feat.clone(), Hist1D::new(nbins, vals.xlow, vals.xhigh));
                }

                return dh;
            });

        let well_histogram = concurrent_histograms.get_mut(&curr_well).unwrap();
        feature_values.par_iter().for_each(|(feat, value)| {
            if let Some(mut hist) = well_histogram.get_mut(*feat) {
                hist.fill(&[*value]);
            }
        });
    });

    // convert back
    let histograms: HashMap<String, HashMap<String, Hist1D>> = concurrent_histograms
        .into_iter()
        .map(|(well_id, well_hist)| {
            let hists = well_hist.into_iter().collect();
            (well_id, hists)
        })
        .collect();

    // WARNING: END OF PARALLEL VERSION

    if verbose {
        let end = start.elapsed();
        println!("INIT LOOP TIME: {:?}", end);
    }

    let start = Instant::now(); // WARNING: this is the start of the second timer

    let well_384 = plate_def.clone();
    let block_def = if let Some(mut blocks) = block_def {
        // clean well names in block_def
        let mut undefined_blocks: HashSet<String> = HashSet::new();
        for block in &blocks {
            let cleaned = clean_well_names(block);
            undefined_blocks.extend(cleaned);
        }
        let undefined_blocks: HashSet<String> = well_384
            .into_iter()
            .filter(|well| !undefined_blocks.contains(well))
            .collect();
        blocks.push(undefined_blocks.into_iter().collect());
        blocks
    } else {
        vec![well_384]
    };

    let mut hd_results: HashMap<String, HashMap<String, f64>> = HashMap::new();

    for group in block_def {
        let select_wells: HashSet<String> = clean_well_names(&group)
            .into_iter()
            .collect::<HashSet<String>>();

        // grab histograms from selected wells
        let mut hd_group: HashMap<String, HashMap<String, Hist1D>> = histograms
            .iter()
            .filter(|(well_id, _)| select_wells.contains(*well_id))
            .map(|(well_id, hists)| (well_id.clone(), hists.clone()))
            .collect();

        if verbose {
            println!("Processing group with wells {:?}", select_wells);
            println!("Length of group: {:?}", hd_group.len());
        }

        let mut cntrl_hists: HashMap<String, Hist1D> = HashMap::new();

        for feat in &features {
            let mut sum_hist: Option<Hist1D> = None;

            for well in vehicle_cntrls {
                if let Some(hist) = hd_group.get(well).and_then(|hists| hists.get(feat)) {
                    if let Some(ref mut sum_hist) = sum_hist {
                        sum_hist.add(hist);
                    } else {
                        sum_hist = Some(hist.clone())
                    }
                }
            }

            if let Some(hist) = sum_hist {
                cntrl_hists.insert(feat.clone(), hist);
            }
        }

        //add vehicle controls to hist diff group
        if verbose {
            println!("Adding control sum onto HD group");
        }
        hd_group.insert("CNTRL".to_string(), cntrl_hists);

        // smooth and normalize histograms
        if verbose {
            println!("Smoothing and normalizing histograms");
        }
        for histogram_collection in hd_group.values_mut() {
            for hist in histogram_collection.values_mut() {
                hist.smooth(0.25);
                hist.normalize();
            }
        }

        // hist square diff
        if verbose {
            println!("calculating scores!");
        }

        let per_feature_scores: Vec<HashMap<String, HashMap<String, f64>>> = features
            .par_iter()
            .map(|feat| {
                let mut local_scores: HashMap<String, HashMap<String, f64>> = HashMap::new();

                // collect exp hist for feature
                let mut exp_hists: Vec<Vec<f64>> = Vec::new();
                let mut well_ids: Vec<String> = Vec::new();

                for (well_id, histogram) in &hd_group {
                    if well_id == "CNTRL" {
                        continue; //ignore controls
                    }

                    if let Some(hist) = histogram.get(feat) {
                        exp_hists.push(hist.data().1.to_vec());
                        well_ids.push(well_id.clone());
                    }
                }

                let exp_arr = Array2::from_shape_vec(
                    (exp_hists.len(), nbins),
                    exp_hists.iter().flatten().cloned().collect(),
                )
                .expect("Failed to create experimental array!")
                .reversed_axes();

                let cntrl_hist = hd_group
                    .get("CNTRL")
                    .and_then(|hist| hist.get(feat))
                    .expect("Control histogram not found!");
                let cntrl_arr = Array1::from(cntrl_hist.data().1.to_vec());

                //WARNING: DELETE THE BELOW
                // if feat == "Nuclei-Cell_Region_Alexa_488_(global)_Axial_Length_Ratio" {
                //     println!("{:?}", exp_arr);
                // }

                // do the actual HD calculation
                let factor = 1.0;
                let hd_score = hist_square_diff(&exp_arr, &cntrl_arr, factor)
                    .expect("Unable to calculat HistDiff");

                for (well_id, hd_value) in well_ids.iter().zip(hd_score.into_iter()) {
                    local_scores
                        .entry(well_id.clone())
                        .or_insert_with(HashMap::new)
                        .insert(feat.clone(), hd_value);
                }

                return local_scores;
            })
            .collect();

        for local_scores in per_feature_scores {
            for (well_id, feat_map) in local_scores {
                hd_results
                    .entry(well_id)
                    .or_insert_with(HashMap::new)
                    .extend(feat_map);
            }
        }
    }

    if verbose {
        let end = start.elapsed(); // WARNING: this the end of the second timer
        println!("Calculation procedure run time: {:?}", end);
    }

    return Ok(hd_results);
}

#[cfg(test)]
mod hd_test {
    use super::*;

    #[test]
    fn test_hd() {
        let fp = "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/024ebc52-9579-11ef-b032-02420a00010f_cellbycell_HD_input.tsv";

        // let fp = "/Users/dterciano/Desktop/LokeyLabFiles/TargetMol/GR_followup/dataset/cell_by_cell_data/024ebc52-9579-11ef-b032-02420a00010f_cellbycell_HD_input.tsv";
        let id_cols = vec!["id".to_string()];
        let vehicle_cntrls = vec!["A1".to_string(), "P24".to_string(), "K12".to_string()];
        let nbins = 20;

        let hd_res = calculate_scores(fp, &id_cols, &vehicle_cntrls, nbins, None, true, None, None)
            .expect("No HD results");

        // let output_fp = "/home/derfelt/git_repos/HistDiff_standalone/temp_store/rust_out.csv";
        // let _ = write_csv(&hd_res, output_fp);

        println!("hd_res length {:?}", hd_res.len());
    }
}
