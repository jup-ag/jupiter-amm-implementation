//! 18-decimal fixed-point primitives. Port of
//! `cubic-pool/src/math/fixed_point.rs`. Pure `u128` arithmetic; no
//! external deps.

use anyhow::{anyhow, Result};

pub const ONE: u128 = 1_000_000_000_000_000_000u128;

pub struct FixedPoint;

impl FixedPoint {
    pub fn mul_down(a: u128, b: u128) -> Result<u128> {
        let product = a
            .checked_mul(b)
            .ok_or_else(|| anyhow!("FixedPoint::mul_down overflow"))?;
        Ok(product / ONE)
    }

    pub fn div_down(a: u128, b: u128) -> Result<u128> {
        if b == 0 {
            return Err(anyhow!("FixedPoint::div_down by zero"));
        }
        let numerator = a
            .checked_mul(ONE)
            .ok_or_else(|| anyhow!("FixedPoint::div_down overflow"))?;
        Ok(numerator / b)
    }

    pub fn div_up(a: u128, b: u128) -> Result<u128> {
        if b == 0 {
            return Err(anyhow!("FixedPoint::div_up by zero"));
        }
        let numerator = a
            .checked_mul(ONE)
            .ok_or_else(|| anyhow!("FixedPoint::div_up overflow"))?;
        if numerator == 0 {
            return Ok(0);
        }
        Ok((numerator - 1) / b + 1)
    }

    pub fn complement(x: u128) -> Result<u128> {
        if x > ONE {
            return Err(anyhow!("FixedPoint::complement out of range"));
        }
        Ok(ONE - x)
    }
}
