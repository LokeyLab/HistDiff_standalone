use core::f64;
use ndarray::prelude::*;
use ndarray::{Array1, Array2, Axis};
use rayon::iter::IntoParallelIterator;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::usize;

#[allow(dead_code)]
pub struct Hist1D {
    nbins: usize,
    xlow: f64,
    xhigh: f64,
    bin_width: f64,
    bins: Vec<f64>,
    counts: Vec<f64>,
}

impl Hist1D {
    pub fn new(nbins: usize, xlow: f64, xhigh: f64) -> Self {
        let bin_width = (xhigh - xlow) / nbins as f64;
        let bins = (0..nbins)
            .map(|i| xlow + (i as f64 + 0.5) * bin_width)
            .collect();
        let counts = vec![0 as f64; nbins];
        return Hist1D {
            nbins,
            xlow,
            xhigh,
            bin_width,
            bins,
            counts,
        };
    }

    pub fn fill(&mut self, data: &[f64]) {
        for &value in data {
            if value >= self.xlow && value < self.xhigh {
                let bin_index = ((value - self.xlow) / self.bin_width) as usize;
                self.counts[bin_index] += 1.0;
            } else if value == self.xhigh {
                // upper bound must be in last bin
                self.counts[self.nbins - 1] += 1.0;
            }

            // any values out of range get ignored
        }
    }

    pub fn data(&self) -> (&[f64], &[f64]) {
        (&self.bins, &self.counts)
    }
}

pub fn hist_square_diff(
    exp: &Array2<f64>,
    ctrl: &Array1<f64>,
    factor: f64,
) -> Result<Array1<f64>, Box<dyn Error>> {
    // shape check
    if exp.shape()[0] != ctrl.len() {
        return Err("Input arrays must have the same shape".into());
    }

    //ctrl mean proxy
    let ctrl_indices = Array1::from_iter(1..=ctrl.shape()[0]).mapv(|x| x as f64);
    let ctrl_mean_proxy = (ctrl * &ctrl_indices).sum();

    // exp mean proxy
    let exp_indices = ctrl_indices.clone().insert_axis(Axis(1));
    let exp_mean_proxy = (exp * &exp_indices).sum_axis(Axis(0));

    //determine negative scores
    let neg_score = exp_mean_proxy.mapv(|e| if ctrl_mean_proxy > e { -1.0 } else { 1.0 });

    // compute differential
    let exp_scaled = exp.mapv(|x| x * factor);
    let ctrl_expanded = ctrl.clone().insert_axis(Axis(1));
    let diff = &ctrl_expanded - &exp_scaled;

    // square the diffs
    let square_diff = diff.mapv(|x| x.powi(2));

    // sum along axis=1
    let sum_diff = square_diff.sum_axis(Axis(0));

    // multiply negative score
    let result = sum_diff * &neg_score;

    return Ok(result);
}

#[derive(Debug, Clone)]
pub struct MinMax {
    pub xlow: f64,
    pub xhigh: f64,
}

#[derive(Debug)]
pub struct MinMaxPlateResult {
    pub min_max: Vec<(String, MinMax)>,
    pub features: Vec<String>,
    pub problemativ_features: Option<Vec<String>>,
}

pub fn get_min_max_plate<P: AsRef<Path>>(
    file_path: P,
    id_cols: &[String],
    verbose: bool,
    prob_out: Option<&str>,
) -> Result<MinMaxPlateResult, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_reader(reader);

    let headers = csv_reader.headers()?.clone();
    let headers_vec = headers.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    // println!("{:?}", headers_vec);

    let id_col_indices: Vec<usize> = id_cols
        .iter()
        .map(|col| headers.iter().position(|h| h == col))
        .collect::<Option<Vec<_>>>()
        .ok_or("ID column not foind in headers")?;

    let feature_indices: Vec<usize> = (0..headers.len())
        .filter(|i| !id_col_indices.contains(i))
        .collect();

    let mut xlow: HashMap<String, f64> = HashMap::new();
    let mut xhigh: HashMap<String, f64> = HashMap::new();
    let mut feats: Vec<String> = Vec::new();
    let mut problematic_features: HashSet<String> = HashSet::new();

    feats = feature_indices
        .iter()
        .map(|&x| headers[x].to_string())
        .collect();

    // initialize xlow and xhigh
    for feat in &feats {
        xlow.insert(feat.clone(), f64::INFINITY);
        xhigh.insert(feat.clone(), f64::NEG_INFINITY);
    }

    for result in csv_reader.records() {
        let record = result?;

        // println!("{:?}", record);
        for &i in &feature_indices {
            let feat = &headers[i];
            let field = &record[i];
            let value = field.parse::<f64>().ok();

            // try to find the min and max
            match value {
                Some(v) if v.is_finite() => {
                    xlow.entry(feat.to_string()).and_modify(|e| *e = e.min(v));
                    xhigh.entry(feat.to_string()).and_modify(|e| *e = e.max(v));
                }

                _ => {
                    problematic_features.insert(feat.to_string());
                }
            }
        }
    }

    println!("Finished reading file!"); // WARN: delete this line

    // adjust the xhigh when xhigh == xlow
    for feat in &feats {
        let low = *xlow.get(feat).unwrap_or(&f64::NAN);
        let high = *xhigh.get(feat).unwrap_or(&f64::NAN);
        if low.is_nan() || high.is_nan() {
            problematic_features.insert(feat.clone());
        } else if low == high {
            let adjusted_high = if low != 0.0 {
                low + low * 0.5
            } else {
                low + 1.0
            };

            xhigh.insert(feat.clone(), adjusted_high);
        }
    }

    let mut min_max_vec: Vec<(String, MinMax)> = Vec::new();

    for feat in &feats {
        let low = xlow.get(feat).cloned();
        let high = xhigh.get(feat).cloned();
        if let (Some(low), Some(high)) = (low, high) {
            if low.is_nan() || high.is_nan() {
                problematic_features.insert(feat.clone());
            } else {
                min_max_vec.push((
                    feat.clone(),
                    MinMax {
                        xlow: low,
                        xhigh: high,
                    },
                ));
            }
        } else {
            problematic_features.insert(feat.clone());
        }
    }

    //remove problematic features
    feats.retain(|feat| !problematic_features.contains(feat));
    min_max_vec.retain(|(feat, _)| !problematic_features.contains(feat));

    // outputting problemativ features
    let problematic_features_vec = if !problematic_features.is_empty() {
        let problematic_features_list = problematic_features.into_iter().collect::<Vec<_>>();
        if verbose {
            eprintln!(
                "MinMax: No values have been found in the following features: {}",
                problematic_features_list.join(" | ")
            );
        }
        if let Some(prob_path_out) = prob_out {
            //let's write this out to a file'
            use std::fs::File;
            use std::io::Write;

            let mut file = File::create(format!("{}_problematicFeats.csv", prob_path_out))?;
            for feat in &problematic_features_list {
                writeln!(file, "{},noValues", feat)?;
            }
        }
        Some(problematic_features_list)
    } else {
        None
    };

    if verbose {
        eprintln!("length of good feats: {}", feats.len());
    }

    return Ok(MinMaxPlateResult {
        min_max: min_max_vec,
        features: feats,
        problemativ_features: problematic_features_vec,
    });
}

#[cfg(test)]
mod min_max_test {

    use super::*;

    #[test]
    fn test_min_max_text() {
        let fp = "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/024ebc52-9579-11ef-b032-02420a00010f_cellbycell_HD_input.tsv";
        let id_cols = vec!["id".to_string()];

        get_min_max_plate(fp, &id_cols, true, None).unwrap();
    }
}
