//! calculation of $\tau_{min}, \tau_{max}$
use anyhow::Result;
use cli::compute;
use solver::bounds::find_tau_bounds;

mod cli;

fn main() -> Result<()> {
    compute(find_tau_bounds)
}
