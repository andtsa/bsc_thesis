use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use anyhow::bail;

use crate::TestCase;

pub trait Case {
    fn algo_args(&self) -> Vec<String>;
}

pub fn run_solver_on<C: Case>(algo: PathBuf, inp: C) -> Result<String> {
    let mut cmd = Command::new(algo);

    for arg in inp.algo_args() {
        cmd.arg(arg);
    }

    let out = cmd.output()?;

    if !out.status.success() {
        bail!(
            "process failed: {}. stderr: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        )
    }

    Ok(String::from_utf8(out.stdout)?)
}

impl Case for TestCase {
    fn algo_args(&self) -> Vec<String> {
        vec![self.a.to_string(), self.b.to_string()]
    }
}
