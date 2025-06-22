//! different weight functions. some of them inhibit the proposed algorithm's
//! optimality.

use anyhow::Result;

use crate::def::StrictOrder;
use crate::tau_w::tau_w;

/// no weight, kendall's tau
pub fn unweighted(_: (usize, usize), _: (usize, usize)) -> f64 {
    1.0
}

/// an asymmetric additive hyperbolic weighting function, as described in Vigna
/// 2014. it uses the left ranking X as the reference.
pub fn hyperbolic_addtv_weight(x: (usize, usize), y: (usize, usize)) -> f64 {
    (1.0 / (x.0 as f64 + 1.0)) + (1.0 / (y.0 as f64 + 1.0))
}

/// an asymmetric multiplicative hyperbolic weighting function, as described in
/// Vigna 2014. it uses the left ranking X as the reference.
pub fn hyperbolic_mult_weight(x: (usize, usize), y: (usize, usize)) -> f64 {
    (1.0 / (x.0 as f64 + 1.0)) * (1.0 / (y.0 as f64 + 1.0))
}

/// a symmetric multiplicative hyperbolic weighting function, as described in
/// Vigna 2014. it uses the left ranking X as the reference.
pub fn hyperbolic_sym_mult_weight(x: (usize, usize), y: (usize, usize)) -> f64 {
    ((1.0 / (x.0 as f64 + 1.0)) * (1.0 / (y.0 as f64 + 1.0))
        + (1.0 / (x.1 as f64 + 1.0)) * (1.0 / (y.1 as f64 + 1.0)))
        / 2.0
}

/// a weighting function to achieve tau_AP from Yilmaz 2008.
/// assymetric, uses X as reference.
pub fn ap_weight(x: (usize, usize), y: (usize, usize)) -> f64 {
    1.0 / (x.0.max(y.0) as f64)
}

pub fn tau_unweighted(a: &StrictOrder, b: &StrictOrder) -> Result<f64> {
    tau_w(a, b, unweighted)
}

/// a weight I made up to test my hypothesis
pub fn ap_high_weight(x: (usize, usize), y: (usize, usize)) -> f64 {
    1.0 / (x.0.min(y.0) as f64)
}

/// a weight I made up to test my hypothesis
pub fn const_weight_42(_x: (usize, usize), _y: (usize, usize)) -> f64 {
    42.0
}

/// a weight I made up to test my hypothesis
pub fn weight_inv_left(x: (usize, usize), _y: (usize, usize)) -> f64 {
    1.0 / (x.0 as f64)
}

/// a weight I made up to test my hypothesis
pub fn hyper_left_weight(x: (usize, usize), _y: (usize, usize)) -> f64 {
    1.0 / (x.0 as f64 + 1.0)
}

/// a weight I made up to test my hypothesis
pub fn weight_inv_right(_x: (usize, usize), y: (usize, usize)) -> f64 {
    1.0 / (y.0 as f64)
}

/// a weight I made up to test my hypothesis
pub fn weight_right(_x: (usize, usize), y: (usize, usize)) -> f64 {
    y.0 as f64
}

/// a weight I made up to test my hypothesis
pub fn weight_left(x: (usize, usize), _y: (usize, usize)) -> f64 {
    x.0 as f64
}

/// a weight I made up to test my hypothesis
pub fn weight_zero(_x: (usize, usize), _y: (usize, usize)) -> f64 {
    0.0
}

/// a weight I made up to test my hypothesis
pub fn weight_sum(x: (usize, usize), y: (usize, usize)) -> f64 {
    (x.0 + y.0) as f64
}

/// a weight I made up to test my hypothesis
pub fn weight_inv_log(x: (usize, usize), y: (usize, usize)) -> f64 {
    1.0 / ((x.0 + y.0 + 1) as f64).ln()
}

/// a weight I made up to test my hypothesis
pub fn threshold_bin_weight(x: (usize, usize), _y: (usize, usize)) -> f64 {
    ((x.0 < 5) as usize) as f64
}

/// a weight I made up to test my hypothesis
pub fn threshold_weight(x: (usize, usize), y: (usize, usize)) -> f64 {
    let d = x.0.max(y.0);
    if d <= 5 {
        (2usize.pow(5 - d as u32)) as f64
    } else {
        0.0
    }
}

/// a weight I made up to test my hypothesis
pub fn rbo_weight(x: (usize, usize), y: (usize, usize)) -> f64 {
    let p = 0.9f64;
    p.powi(x.0.max(y.0) as i32) / (1.0 - p)
}

/// a weight I made up to test my hypothesis
pub fn rbo_other_weight(x: (usize, usize), y: (usize, usize)) -> f64 {
    let p = 0.5f64;
    p.powi(x.0.max(y.0) as i32)
}

/// a weight I made up to test my hypothesis
pub fn expo_thresh_weight(x: (usize, usize), y: (usize, usize)) -> f64 {
    let d = x.0.max(y.0);
    (2usize.pow(5 - d as u32)) as f64
}
