use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;

use crate::AlgoOut;
use crate::FailType;
use crate::TestCase;
use crate::TestResult;

pub fn parse_algo_sol(output: String) -> Result<Option<AlgoOut>> {
    let mut algo_sol = AlgoOut::default();
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        if line.trim().starts_with("skipped") {
            // brute-force solvers won't run but shouldn't fail on tests with
            // too many permutations
            return Ok(None);
        }
        let mut parts = line.split(':');
        let label = parts.next().ok_or(anyhow!("No label on line {line:?}"))?;
        let value = parts.next().ok_or(anyhow!("No value on line {line:?}"))?;

        match label {
            "tmin" => {
                algo_sol.tmin = Some(
                    value
                        .parse::<f64>()
                        .map_err(|e| anyhow!("failed to parse tau max: {value:?} ({e})"))?,
                )
            }
            "tmax" => {
                algo_sol.tmax = Some(
                    value
                        .parse::<f64>()
                        .map_err(|e| anyhow!("failed to parse tau max: {value:?} ({e})"))?,
                )
            }
            "minp" => {
                let parsed = value.trim().split('/').collect::<Vec<_>>();
                if parsed.len() != 2 {
                    bail!("failed to parse min p: {value:?}");
                }
                algo_sol
                    .minp
                    .push((parsed[0].to_string(), parsed[1].to_string()));
            }
            "maxp" => {
                let parsed = value.trim().split('/').collect::<Vec<_>>();
                if parsed.len() != 2 {
                    bail!("failed to parse max p: {value:?}");
                }
                algo_sol
                    .maxp
                    .push((parsed[0].to_string(), parsed[1].to_string()));
            }
            _ => bail!("unknown label: {value}"),
        }
    }
    Ok(Some(algo_sol))
}

pub fn verify_result(case: Result<(String, TestCase)>) -> Result<TestResult> {
    let (output, input) = case?;
    let algo_sol = if let Some(x) = parse_algo_sol(output)? {
        x
    } else {
        return Ok(TestResult::Skipped);
    };
    // the algorithm can give just a min result, just a max result, or both.
    let min_sol_exists = algo_sol.tmin.is_some() && !algo_sol.minp.is_empty();
    let max_sol_exists = algo_sol.tmax.is_some() && !algo_sol.maxp.is_empty();
    // but not neither
    if !(min_sol_exists || max_sol_exists) {
        return Ok(TestResult::Empty(input));
    }

    for (mina, minb) in &algo_sol.minp {
        if !input
            .min_sol_pairs
            .iter()
            .any(|p| p.0.eq(mina) && p.1.eq(minb))
        {
            return Ok(TestResult::Fail((input, algo_sol), FailType::MinP));
        }
    }

    if algo_sol
        .tmin
        .is_some_and(|tmin| (tmin - input.tmin).abs() > 1e-5f64)
    {
        let (ta, ts) = (algo_sol.tmin.unwrap(), input.tmin);
        return Ok(TestResult::Fail((input, algo_sol), FailType::Tmin(ta, ts)));
    }

    for (maxa, maxb) in &algo_sol.maxp {
        if !input
            .max_sol_pairs
            .iter()
            .any(|p| p.0.eq(maxa) && p.1.eq(maxb))
        {
            return Ok(TestResult::Fail((input, algo_sol), FailType::MaxP));
        }
    }

    if algo_sol
        .tmax
        .is_some_and(|tmax| (tmax - input.tmax).abs() > 1e-5f64)
    {
        let (ta, ts) = (algo_sol.tmax.unwrap(), input.tmax);
        return Ok(TestResult::Fail((input, algo_sol), FailType::Tmax(ta, ts)));
    }

    Ok(TestResult::Pass)
}
