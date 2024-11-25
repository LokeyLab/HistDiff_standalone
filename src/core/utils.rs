#[allow(unused_parens)]
pub fn exponential_smoothing(x: &[f64], alpha: f64) -> Vec<f64> {
    let n = x.len();
    let mut smoothing: Vec<f64> = Vec::with_capacity(n);

    for i in (0..n) {
        let x_i = x[i];
        let s_i = if i == 0 {
            if n > 1 {
                x_i + alpha * (x[i + 1] - x_i)
            } else {
                x_i
            }
        } else if (i == (n - 1)) {
            alpha * (x[i - 1] - x_i) + x_i
        } else {
            alpha * (x[i - 1] - x_i) + x_i + alpha * (x[i + 1] - x_i)
        };

        smoothing.push(s_i);
    }

    return smoothing;
}

pub fn normalize(x: &[f64]) -> Vec<f64> {
    let sum: f64 = x.iter().sum();
    if sum == 0.0 {
        return vec![0.0; x.len()];
    } else {
        return x.iter().map(|&e| e / sum).collect();
    }
}

#[cfg(test)]
mod utils_test {
    use super::*;

    #[test]
    fn test_smoothing_simple() {
        let test_in = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let alpha = 0.25;
        let smoothed = exponential_smoothing(&test_in, alpha);

        println!("{:?}", smoothed);

        let answer = vec![1.25, 2.0, 3.0, 4.0, 4.75];
        assert_eq!(smoothed, answer);
    }
}
