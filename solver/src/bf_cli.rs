//! brute force calculation of $\tau_{min}, \tau_{max}$
use anyhow::Result;
use lib::tau_h::hyperbolic_addtv_weight;
use lib::tau_h::tau_h;
use solver::bounds::bf::tau_bounds_bf;
use solver::cli::compute;

fn main() -> Result<()> {
    compute(|a, b| {
        tau_bounds_bf(a, b, |x, y| {
            tau_h(x, y, hyperbolic_addtv_weight, lib::tau_h::TauVariants::A)
        })
    })
}
