//! brute force calculation of $\tau_{min}, \tau_{max}$
use anyhow::Result;
use cli::compute;
use lib::tau_w::tau_w;
use solver::bounds::bf::tau_bounds_bf;

mod cli;

fn main() -> Result<()> {
    compute(|a, b, w| tau_bounds_bf(a, b, |x, y| tau_w(x, y, w)))
}
