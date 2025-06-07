//! brute force calculation of $\tau_{min}, \tau_{max}$
use anyhow::Result;
use lib::tau_h::tau_unweighted;
use solver::bounds::bf::tau_bounds_bf;
use solver::cli::compute;

fn main() -> Result<()> {
    compute(|a, b| tau_bounds_bf(a, b, tau_unweighted))
}
