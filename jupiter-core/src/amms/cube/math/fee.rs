//! Fee math ported from cubic-pool/src/instructions/swap.rs:294-319.
//!
//! - `swap_fee_rate`: precision 1_000_000 (hundredths-of-bps; 3_000 = 30 bps).
//!   Fee is taken on INPUT side, BEFORE the curve.
//! - `protocol_fee_rate`: precision 10_000 (bps). Cut of the swap fee earmarked
//!   for the protocol treasury — does NOT change the trader's `amount_out`.
//!   It DOES affect how much enters `virtual_balance_in`:
//!     amount_in_added_to_pool = amount_in - protocol_fee_amount
//!   See swap.rs:167-209.

use super::super::constants::{PROTOCOL_FEE_PRECISION, SWAP_FEE_PRECISION};
use anyhow::{anyhow, Result};

/// Returns `(fee_amount, amount_in_after_fee)`. Mirrors `calculate_swap_fee`
/// with the ZeroFeeAmount guard from swap.rs:170.
pub fn apply_swap_fee(amount_in: u64, swap_fee_rate: u32) -> Result<(u64, u64)> {
    if swap_fee_rate == 0 {
        return Ok((0, amount_in));
    }
    let fee = (amount_in as u128)
        .checked_mul(swap_fee_rate as u128)
        .ok_or_else(|| anyhow!("apply_swap_fee: overflow"))?
        / (SWAP_FEE_PRECISION as u128);
    let fee_u64: u64 = fee
        .try_into()
        .map_err(|_| anyhow!("apply_swap_fee: fee > u64::MAX"))?;
    if fee_u64 == 0 {
        return Err(anyhow!(
            "apply_swap_fee: dust input, fee rounds to zero"
        ));
    }
    Ok((fee_u64, amount_in - fee_u64))
}

/// Protocol-fee slice of a collected swap fee. Mirrors `calculate_protocol_fee`.
pub fn calculate_protocol_fee(fee_amount: u64, protocol_fee_rate: u16) -> Result<u64> {
    if protocol_fee_rate == 0 || fee_amount == 0 {
        return Ok(0);
    }
    let pf = (fee_amount as u128)
        .checked_mul(protocol_fee_rate as u128)
        .ok_or_else(|| anyhow!("calculate_protocol_fee: overflow"))?
        / (PROTOCOL_FEE_PRECISION as u128);
    pf.try_into()
        .map_err(|_| anyhow!("calculate_protocol_fee: > u64::MAX"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fee_30bps_on_1m() {
        let (fee, after) = apply_swap_fee(1_000_000, 3_000).unwrap();
        assert_eq!(fee, 3_000);
        assert_eq!(after, 997_000);
    }

    #[test]
    fn fee_zero_rate_passthrough() {
        let (fee, after) = apply_swap_fee(1_000_000, 0).unwrap();
        assert_eq!(fee, 0);
        assert_eq!(after, 1_000_000);
    }

    #[test]
    fn fee_dust_input_errors() {
        // 1 * 3_000 / 1_000_000 = 0 → ZeroFeeAmount equivalent.
        assert!(apply_swap_fee(1, 3_000).is_err());
    }

    #[test]
    fn protocol_fee_20pct() {
        // 20% of 100 = 20.
        assert_eq!(calculate_protocol_fee(100, 2_000).unwrap(), 20);
    }
}
