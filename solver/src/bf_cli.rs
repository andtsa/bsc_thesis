//! brute force calculation of $\tau_{min}, \tau_{max}$
use anyhow::Result;
use lib::tau_h::tau_h;
use solver::bounds::bf::tau_bounds_bf;
use solver::cli::compute;

fn main() -> Result<()> {
    compute(|a, b, w| {
        tau_bounds_bf(a, b, |x, y| {
            tau_h(x, y, w, lib::tau_h::TauVariants::A)
        })
    })

}
