//! calculation of $\tau_{min}, \tau_{max}$
use anyhow::Result;
use solver::bounds::find_tau_bounds;
use solver::cli::compute;

fn main() -> Result<()> {
    compute(find_tau_bounds)
}
