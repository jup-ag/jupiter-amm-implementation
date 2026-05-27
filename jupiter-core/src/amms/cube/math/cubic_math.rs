//! Weighted constant-product swap formula. Port of
//! cubic-pool/src/math/cubic_math.rs `calc_out_given_in`.
//!
//! `aO = bO * (1 - (bI / (bI + aI)) ^ (wI / wO))`
//!
//! All operands enter as raw `u64` token units. Intermediate fixed-point
//! work happens in `u128` at 1e18 scale. Result is the raw `u64` payout
//! to the trader.

use super::super::constants::{weight_to_fixed_point, ONE};
use super::fixed_point::FixedPoint;
use super::log_exp_math::LogExpMath;
use anyhow::{anyhow, Result};

pub fn calc_out_given_in(
    virtual_balance_in: u64,
    weight_in: u64,
    virtual_balance_out: u64,
    weight_out: u64,
    amount_in_after_fee: u64,
    actual_balance_out: u64,
) -> Result<u64> {
    let weight_in_fp = weight_to_fixed_point(weight_in);
    let weight_out_fp = weight_to_fixed_point(weight_out);

    let balance_in = virtual_balance_in as u128;
    let amt_in = amount_in_after_fee as u128;
    let denominator = balance_in
        .checked_add(amt_in)
        .ok_or_else(|| anyhow!("cubic_math: bI+aI overflow"))?;

    let base = FixedPoint::div_up(balance_in, denominator)?;
    let exponent = FixedPoint::div_down(weight_in_fp, weight_out_fp)?;

    // LP-safe rounding: pow() rounds DOWN, +1 ULP then clamp at ONE
    // pushes the user's payout strictly downward.
    let power_raw = LogExpMath::pow(base, exponent)?;
    let power = power_raw.saturating_add(1).min(ONE);
    let complement = FixedPoint::complement(power)?;

    let balance_out = virtual_balance_out as u128;
    let amount_out_calc = FixedPoint::mul_down(balance_out, complement)?;

    let actual_out = actual_balance_out as u128;
    if amount_out_calc > actual_out {
        return Err(anyhow!(
            "cubic_math: amount_out {amount_out_calc} exceeds actual_balance_out {actual_out}"
        ));
    }

    u64::try_from(amount_out_calc).map_err(|_| anyhow!("cubic_math: amount_out > u64::MAX"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worked_example_30bps_50_50() {
        // From contracts-research worked example (Case A):
        //   vbI=1e9, vbO=2e9, w=50/50, amt_in=1e7, fee=3000 (= 30 bps).
        //   amount_in_after_fee = 1e7 - 30_000 = 9_970_000.
        //   Expected ≈ 19_743_160.
        let out = calc_out_given_in(
            1_000_000_000, 5_000,
            2_000_000_000, 5_000,
            9_970_000,
            2_000_000_000,
        )
        .unwrap();
        let expected = 19_743_160u64;
        let diff = out.abs_diff(expected);
        assert!(diff < 1_000, "got {out}, expected ~{expected}");
    }

    #[test]
    fn sdk_gold_50_50_10pct_trade() {
        // SDK tests/math/cubicMath.test.ts:11-25 — balanced 50/50 pool,
        // 10% input → ~9.09% output of the out-side virtual balance.
        let out = calc_out_given_in(
            1_000_000_000, 5_000,
            1_000_000_000, 5_000,
            100_000_000,
            1_000_000_000,
        )
        .unwrap();
        let frac = out as f64 / 1_000_000_000.0;
        assert!((frac - 0.0909).abs() < 0.001, "frac = {frac}");
    }

    #[test]
    fn reverts_when_exceeds_actual() {
        // Trade so large the curve wants more than the LP can supply.
        let r = calc_out_given_in(
            1_000_000_000, 5_000,
            1_000_000_000, 5_000,
            100_000_000_000,
            500_000_000,
        );
        assert!(r.is_err());
    }

    #[test]
    fn asymmetric_weights_amplify_output() {
        let balanced = calc_out_given_in(
            1_000_000_000, 5_000,
            1_000_000_000, 5_000,
            100_000_000,
            1_000_000_000,
        )
        .unwrap();
        let asym = calc_out_given_in(
            1_000_000_000, 8_000,
            1_000_000_000, 2_000,
            100_000_000,
            1_000_000_000,
        )
        .unwrap();
        assert!(asym > balanced, "asym {asym} should beat balanced {balanced}");
    }
}
