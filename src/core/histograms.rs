use ndarray::prelude::*;
use ndarray::{Array1, Array2, Axis};
use std::error::Error;
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

// hist_square_diff
#[cfg(test)]
mod hist_square_diff {
    use super::*;
    use ndarray::array;

    #[test]
    fn test_hist_square_diff_basic() {
        // Control histogram (1D array)
        let ctrl = array![2.0, 3.0, 4.0]; // num_bins = 3

        // Experimental histograms (2D array with 2 samples)
        let exp = array![
            [1.0, 2.0], // Bin 1
            [2.0, 3.0], // Bin 2
            [3.0, 4.0], // Bin 3
        ]; // Shape: (3 bins, 2 samples)

        let factor = 1.0;

        let result = hist_square_diff(&exp, &ctrl, factor).unwrap();

        // Manually compute expected results
        // For sample 1:
        // ctrl_mean_proxy = (2*1 + 3*2 + 4*3) = 2 + 6 + 12 = 20
        // exp_mean_proxy[0] = (1*1 + 2*2 + 3*3) = 1 + 4 + 9 = 14
        // neg_score[0] = if 20 > 14 => -1.0
        // diff = ctrl - exp_scaled = [2-1, 3-2, 4-3] = [1, 1, 1]
        // squared_diff = [1^2, 1^2, 1^2] = [1, 1, 1]
        // sum_diff[0] = 1 + 1 + 1 = 3
        // result[0] = sum_diff[0] * neg_score[0] = 3 * (-1) = -3

        // For sample 2:
        // exp_mean_proxy[1] = (1*2 + 2*3 + 3*4) = 2 + 6 + 12 = 20
        // neg_score[1] = if 20 > 20 => 1.0
        // diff = [2-2, 3-3, 4-4] = [0, 0, 0]
        // squared_diff = [0, 0, 0]
        // sum_diff[1] = 0
        // result[1] = 0 * 1 = 0

        let expected = array![-3.0, 0.0];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_hist_square_diff_with_factor() {
        let ctrl = array![2.0, 3.0, 4.0];
        let exp = array![[1.0, 2.0], [2.0, 3.0], [3.0, 4.0],];
        let factor = 0.5;

        let result = hist_square_diff(&exp, &ctrl, factor).unwrap();
        // Adjust calculations for factor = 0.5
        // exp_scaled = exp * 0.5

        // For sample 1:
        // exp_scaled[:,0] = [0.5, 1.0, 1.5]
        // diff = [2-0.5, 3-1.0, 4-1.5] = [1.5, 2.0, 2.5]
        // squared_diff = [2.25, 4.0, 6.25]
        // sum_diff[0] = 2.25 + 4.0 + 6.25 = 12.5
        // ctrl_mean_proxy = 20 (same as before)
        // exp_mean_proxy[0] = (1*0.5 + 2*1.0 + 3*1.5) = 0.5 + 2.0 + 4.5 = 7.0
        // neg_score[0] = if 20 > 7 => -1.0
        // result[0] = 12.5 * (-1) = -12.5

        // Similarly for sample 2

        let expected = array![-12.5, -3.0];

        println!("result: {} expected: {}", result, expected);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_hist_square_diff_ctrl_less_than_exp() {
        let ctrl = array![1.0, 1.0, 1.0];
        let exp = array![[2.0, 3.0], [2.0, 3.0], [2.0, 3.0],];
        let factor = 1.0;

        let result = hist_square_diff(&exp, &ctrl, factor).unwrap();

        // ctrl_mean_proxy = (1*1 + 1*2 + 1*3) = 1 + 2 + 3 = 6
        // exp_mean_proxy[0] = (1*2 + 2*2 + 3*2) = 2 + 4 + 6 = 12
        // neg_score[0] = if 6 > 12 => 1.0 (since 6 is not greater than 12, neg_score = 1.0)
        // diff = ctrl_expanded - exp_scaled
        // diff[:,0] = [1-2, 1-2, 1-2] = [-1, -1, -1]
        // squared_diff[:,0] = [1, 1, 1]
        // sum_diff[0] = 3
        // result[0] = 3 * 1 = 3

        // Similarly for sample 2
        // diff[:,1] = [1-3, 1-3, 1-3] = [-2, -2, -2]
        // squared_diff[:,1] = [4, 4, 4]
        // sum_diff[1] = 12
        // result[1] = 12 * 1 = 12

        let expected = array![3.0, 12.0];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_hist_square_diff_dimension_mismatch() {
        let ctrl = array![1.0, 2.0];
        let exp = array![[1.0, 2.0], [2.0, 3.0], [3.0, 4.0],]; // More bins than ctrl

        let result = hist_square_diff(&exp, &ctrl, 1.0);

        assert!(result.is_err());
    }

    #[test]
    fn test_hist_square_diff_empty_arrays() {
        let ctrl = Array1::<f64>::zeros(0);
        let exp = Array2::<f64>::zeros((0, 0));

        let result = hist_square_diff(&exp, &ctrl, 1.0);

        // Depending on your preference, you might allow empty inputs or return an error
        // Assuming the function allows empty inputs and returns an empty array

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}

// histogram.rs
// Tests all pass

// #[cfg(test)]
// mod hist_test {
//     use super::*;
//
//     #[test]
//     fn test_hist1d_initialization() {
//         let nbins = 10;
//         let xlow = 0.0;
//         let xhigh = 100.0;
//         let hist = Hist1D::new(nbins, xlow, xhigh);
//
//         // Check that the histogram is initialized correctly
//         assert_eq!(hist.nbins, nbins);
//         assert_eq!(hist.xlow, xlow);
//         assert_eq!(hist.xhigh, xhigh);
//         assert_eq!(hist.counts.len(), nbins);
//         assert_eq!(hist.bins.len(), nbins);
//
//         // Check that counts are initialized to zero
//         for count in &hist.counts {
//             assert_eq!(*count, 0.0);
//         }
//
//         // Check bin centers
//         let expected_bin_width = (xhigh - xlow) / nbins as f64;
//         for (i, &bin_center) in hist.bins.iter().enumerate() {
//             let expected_center = xlow + (i as f64 + 0.5) * expected_bin_width;
//             assert_eq!(bin_center, expected_center);
//         }
//     }
//
//     #[test]
//     fn test_hist1d_fill() {
//         let mut hist = Hist1D::new(5, 0.0, 5.0);
//         let data = vec![0.5, 1.5, 2.5, 3.5, 4.5];
//         hist.fill(&data);
//
//         // Expected counts: each bin should have 1.0 count
//         let expected_counts = vec![1.0, 1.0, 1.0, 1.0, 1.0];
//         assert_eq!(hist.counts, expected_counts);
//     }
//
//     #[test]
//     fn test_hist1d_fill_with_values_out_of_range() {
//         let mut hist = Hist1D::new(5, 0.0, 5.0);
//         let data = vec![-1.0, 0.0, 2.5, 5.0, 6.0];
//         hist.fill(&data);
//
//         // Expected counts:
//         // -1.0 is out of range, ignored
//         // 0.0 falls into first bin
//         // 2.5 falls into third bin
//         // 5.0 is equal to xhigh, should be included in last bin
//         // 6.0 is out of range, ignored
//         let expected_counts = vec![1.0, 0.0, 1.0, 0.0, 1.0];
//         assert_eq!(hist.counts, expected_counts);
//     }
//
//     #[test]
//     fn test_hist1d_data() {
//         let nbins = 4;
//         let xlow = 0.0;
//         let xhigh = 4.0;
//         let hist = Hist1D::new(nbins, xlow, xhigh);
//
//         let (bins, counts) = hist.data();
//         assert_eq!(bins.len(), nbins);
//         assert_eq!(counts.len(), nbins);
//
//         let expected_bins = vec![0.5, 1.5, 2.5, 3.5];
//         assert_eq!(bins, expected_bins.as_slice());
//
//         // Counts should be zero initially
//         for &count in counts {
//             assert_eq!(count, 0.0);
//         }
//     }
//
//     #[test]
//     fn test_hist1d_fill_multiple_values_per_bin() {
//         let mut hist = Hist1D::new(5, 0.0, 5.0);
//         let data = vec![0.1, 0.2, 1.1, 1.2, 1.3, 2.5, 3.6, 4.7, 4.8];
//         hist.fill(&data);
//
//         // Expected counts:
//         // Bin 0: 0.1, 0.2 -> count = 2.0
//         // Bin 1: 1.1, 1.2, 1.3 -> count = 3.0
//         // Bin 2: 2.5 -> count = 1.0
//         // Bin 3: 3.6 -> count = 1.0
//         // Bin 4: 4.7, 4.8 -> count = 2.0
//         let expected_counts = vec![2.0, 3.0, 1.0, 1.0, 2.0];
//         assert_eq!(hist.counts, expected_counts);
//     }
//
//     #[test]
//     fn test_hist1d_fill_empty_data() {
//         let mut hist = Hist1D::new(5, 0.0, 5.0);
//         let data: Vec<f64> = Vec::new();
//         hist.fill(&data);
//
//         // Expected counts: all zeros
//         let expected_counts = vec![0.0, 0.0, 0.0, 0.0, 0.0];
//         assert_eq!(hist.counts, expected_counts);
//     }
//
//     #[test]
//     fn test_hist1d_with_floating_point_edges() {
//         let mut hist = Hist1D::new(4, 0.0, 1.0);
//         let data = vec![0.0, 0.25, 0.5, 0.75, 1.0];
//         hist.fill(&data);
//
//         // Expected counts:
//         // Bin 0: 0.0
//         // Bin 1: 0.25
//         // Bin 2: 0.5
//         // Bin 3: 0.75, 1.0 (since xhigh is included in last bin)
//         let expected_counts = vec![1.0, 1.0, 1.0, 2.0];
//         assert_eq!(hist.counts, expected_counts);
//     }
// }
