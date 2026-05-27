//! Sliding-window selloff rate-limiter check. Direct port of
//! `cubic-pool/src/math/max_selloff.rs::check_and_advance`.
//!
//! Cubic-pool enforces, per input token, a cap on `amount_in` summed
//! across a sliding window. We replicate the check in `quote` so we can
//! either return a smaller in_amount (input cap) or reject the quote
//! before submission — mirroring the on-chain `MaxSelloffExceeded`
//! (6049) revert.
//!
//! Pure function: this version does NOT mutate `dynamics`. It only tells
//! the caller whether a candidate `amount_in` would pass.
//!
//! Formula:
//!   effective = previous * (period - elapsed) / period + current
//!   require effective + amount_in <= max_selloff
//!
//! Bucket rotation when `elapsed >= period`:
//!   - `elapsed >= 2*period`  → both buckets aged out
//!   - `elapsed >= period`    → current → previous, current = 0
//!   - else                   → no rotation

use anyhow::{anyhow, Result};

/// Inputs needed for the check, copied out of `TokenSlot` so the caller
/// can drive multiple hypothetical `amount_in` values without mutating
/// state.
#[derive(Debug, Clone, Copy)]
pub struct SelloffInputs {
    /// Per-token cap. `0` disables the check (returns `Ok(())`).
    pub max_selloff: u64,
    /// Sliding-window length in seconds. Must be `> 0` if `max_selloff > 0`.
    pub period_length: u32,
    pub previous_selloff: u64,
    pub current_selloff: u64,
    pub window_start_timestamp: i64,
}

/// Returns `Ok(())` if `amount_in` would be accepted by the on-chain
/// `check_and_advance`, or an error mirroring `MaxSelloffExceeded`.
pub fn check(inputs: &SelloffInputs, amount_in: u64, now_unix_ts: i64) -> Result<()> {
    if inputs.max_selloff == 0 {
        return Ok(());
    }
    if inputs.period_length == 0 {
        return Err(anyhow!(
            "max_selloff: invalid config — period == 0 while cap > 0"
        ));
    }

    let period_i: i64 = inputs.period_length as i64;
    let raw_elapsed = now_unix_ts.saturating_sub(inputs.window_start_timestamp);
    let mut elapsed = raw_elapsed.max(0);

    let two_periods = period_i
        .checked_mul(2)
        .ok_or_else(|| anyhow!("max_selloff: period * 2 overflow"))?;

    let (new_previous, new_current, elapsed_in_window) = if elapsed >= two_periods {
        (0u64, 0u64, 0i64)
    } else if elapsed >= period_i {
        let new_ws = inputs
            .window_start_timestamp
            .checked_add(period_i)
            .ok_or_else(|| anyhow!("max_selloff: window_start + period overflow"))?;
        elapsed = now_unix_ts.saturating_sub(new_ws);
        if elapsed < 0 {
            elapsed = 0;
        }
        (inputs.current_selloff, 0u64, elapsed)
    } else {
        (inputs.previous_selloff, inputs.current_selloff, elapsed)
    };

    let period_u = period_i as u128;
    let remaining = period_u
        .checked_sub(elapsed_in_window as u128)
        .ok_or_else(|| anyhow!("max_selloff: remaining underflow"))?;
    let weighted_prev = (new_previous as u128)
        .checked_mul(remaining)
        .ok_or_else(|| anyhow!("max_selloff: weighted_prev overflow"))?
        / period_u;
    let pre_effective = weighted_prev
        .checked_add(new_current as u128)
        .ok_or_else(|| anyhow!("max_selloff: pre_effective overflow"))?;
    let effective_total = pre_effective
        .checked_add(amount_in as u128)
        .ok_or_else(|| anyhow!("max_selloff: effective_total overflow"))?;

    if effective_total > inputs.max_selloff as u128 {
        return Err(anyhow!(
            "max_selloff: would exceed cap (effective={} > cap={})",
            effective_total,
            inputs.max_selloff
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(cap: u64, period: u32) -> SelloffInputs {
        SelloffInputs {
            max_selloff: cap,
            period_length: period,
            previous_selloff: 0,
            current_selloff: 0,
            window_start_timestamp: 0,
        }
    }

    #[test]
    fn disabled_when_cap_zero() {
        assert!(check(&inputs(0, 60), u64::MAX, 0).is_ok());
    }

    #[test]
    fn passes_under_cap() {
        assert!(check(&inputs(1_000, 60), 500, 0).is_ok());
    }

    #[test]
    fn rejects_at_exact_cap_plus_one() {
        let r = check(&inputs(1_000, 60), 1_001, 0);
        assert!(r.is_err());
    }

    #[test]
    fn accepts_at_exact_cap() {
        assert!(check(&inputs(1_000, 60), 1_000, 0).is_ok());
    }

    #[test]
    fn current_window_blocks_excess() {
        let mut i = inputs(1_000, 60);
        i.current_selloff = 800;
        // 800 in current + 250 new > 1000 → rejects
        assert!(check(&i, 250, 0).is_err());
        // 800 + 199 = 999 ≤ 1000 → passes
        assert!(check(&i, 199, 0).is_ok());
    }

    #[test]
    fn one_period_rolls_current_into_previous() {
        let mut i = inputs(1_000, 60);
        i.current_selloff = 800;
        i.window_start_timestamp = 0;
        // After one period: previous = 800, current = 0.
        // Effective (now=60, elapsed_in_new_window=0):
        //   weighted_prev = 800 * (60 - 0) / 60 = 800
        //   effective = 800 + 0 = 800
        // 800 + 199 = 999 → passes; 800 + 201 = 1001 → fails.
        assert!(check(&i, 199, 60).is_ok());
        assert!(check(&i, 201, 60).is_err());
    }

    #[test]
    fn two_periods_clear_both_buckets() {
        let mut i = inputs(1_000, 60);
        i.previous_selloff = 999;
        i.current_selloff = 999;
        // After 120s, both buckets aged out → full capacity available.
        assert!(check(&i, 1_000, 120).is_ok());
        assert!(check(&i, 1_001, 120).is_err());
    }

    #[test]
    fn invalid_config_period_zero_with_cap() {
        assert!(check(&inputs(1_000, 0), 1, 0).is_err());
    }
}
