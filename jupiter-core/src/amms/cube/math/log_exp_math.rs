//! Balancer V2 LogExpMath port — 18-decimal `ln` / `exp` / `pow`.
//! Faithfully mirrors cubic-pool/src/math/log_exp_math.rs (constants and
//! Taylor-series term counts unchanged); only the Anchor error shell is
//! swapped for `anyhow`.

use anyhow::{anyhow, Result};

const ONE: u128 = 1_000_000_000_000_000_000;
const MAX_NATURAL_EXPONENT: i128 = 46_000_000_000_000_000_000;
const MIN_NATURAL_EXPONENT: i128 = -41_000_000_000_000_000_000;

// Decomposition constants: a_i = e^(x_i) * ONE.
const X0: u128 = 32_000_000_000_000_000_000;
const A0: u128 = 78_962_960_182_680_695_161_000_000_000_000;
const X1: u128 = 16_000_000_000_000_000_000;
const A1: u128 = 8_886_110_520_507_872_636_760_000;
const X2: u128 = 8_000_000_000_000_000_000;
const A2: u128 = 2_980_957_987_041_728_274_740;
const X3: u128 = 4_000_000_000_000_000_000;
const A3: u128 = 54_598_150_033_144_239_078;
const X4: u128 = 2_000_000_000_000_000_000;
const A4: u128 = 7_389_056_098_930_650_227;
const X5: u128 = 1_000_000_000_000_000_000;
const A5: u128 = 2_718_281_828_459_045_235;
const X6: u128 = 500_000_000_000_000_000;
const A6: u128 = 1_648_721_270_700_128_146;
const X7: u128 = 250_000_000_000_000_000;
const A7: u128 = 1_284_025_416_687_741_484;
const X8: u128 = 125_000_000_000_000_000;
const A8: u128 = 1_133_148_453_066_826_316;
const X9: u128 = 62_500_000_000_000_000;
const A9: u128 = 1_064_494_458_917_859_429;

const STEPS: usize = 10;
const X_STEPS: [u128; STEPS] = [X0, X1, X2, X3, X4, X5, X6, X7, X8, X9];
const A_STEPS: [u128; STEPS] = [A0, A1, A2, A3, A4, A5, A6, A7, A8, A9];

const EXP_TAYLOR_TERMS: usize = 12;
const LN_TAYLOR_TERMS: usize = 6;

pub struct LogExpMath;

impl LogExpMath {
    pub fn pow(x: u128, y: u128) -> Result<u128> {
        if y == 0 {
            return Ok(ONE);
        }
        if x == 0 {
            return Ok(0);
        }
        if x == ONE {
            return Ok(ONE);
        }

        let ln_x = Self::ln(x)?;

        let y_i = y as i128;
        let one_i = ONE as i128;

        let int_part = ln_x / one_i;
        let frac_part = ln_x % one_i;

        let logx_times_y = int_part
            .checked_mul(y_i)
            .ok_or_else(|| anyhow!("pow: int_part * y overflow"))?
            .checked_add(
                frac_part
                    .checked_mul(y_i)
                    .ok_or_else(|| anyhow!("pow: frac_part * y overflow"))?
                    / one_i,
            )
            .ok_or_else(|| anyhow!("pow: logx_times_y overflow"))?;

        if !(MIN_NATURAL_EXPONENT..=MAX_NATURAL_EXPONENT).contains(&logx_times_y) {
            return Err(anyhow!("pow: exponent out of range"));
        }

        Self::exp(logx_times_y)
    }

    pub fn ln(a: u128) -> Result<i128> {
        if a == 0 {
            return Err(anyhow!("ln(0) undefined"));
        }
        if a < ONE {
            let a_inv = mul_div_down(ONE, ONE, a)?;
            return Ok(-Self::ln_positive(a_inv)?);
        }
        Self::ln_positive(a)
    }

    fn ln_positive(mut a: u128) -> Result<i128> {
        let mut sum: i128 = 0;

        for i in 0..STEPS {
            if a >= A_STEPS[i] {
                a = mul_div_down(a, ONE, A_STEPS[i])?;
                sum += X_STEPS[i] as i128;
            }
        }

        let a_minus_one = a
            .checked_sub(ONE)
            .ok_or_else(|| anyhow!("ln: a < ONE underflow after decomposition"))?;
        let a_plus_one = a
            .checked_add(ONE)
            .ok_or_else(|| anyhow!("ln: a+ONE overflow"))?;
        let z = mul_div_down(a_minus_one, ONE, a_plus_one)? as i128;

        let one_i = ONE as i128;
        let z_sq = z
            .checked_mul(z)
            .ok_or_else(|| anyhow!("ln: z*z overflow"))?
            / one_i;

        let mut num = z;
        let mut series_sum = num;

        for k in 1..LN_TAYLOR_TERMS {
            num = num
                .checked_mul(z_sq)
                .ok_or_else(|| anyhow!("ln: num*z_sq overflow"))?
                / one_i;
            let divisor = (2 * k + 1) as i128;
            series_sum = series_sum
                .checked_add(num / divisor)
                .ok_or_else(|| anyhow!("ln: series_sum overflow"))?;
        }

        series_sum = series_sum
            .checked_mul(2)
            .ok_or_else(|| anyhow!("ln: series_sum*2 overflow"))?;

        Ok(sum + series_sum)
    }

    pub fn exp(x: i128) -> Result<u128> {
        if !(MIN_NATURAL_EXPONENT..=MAX_NATURAL_EXPONENT).contains(&x) {
            return Err(anyhow!("exp: argument out of range"));
        }

        if x < 0 {
            let pos = Self::exp(-x)?;
            return mul_div_down(ONE, ONE, pos);
        }

        let mut x = x as u128;
        let mut product: u128 = ONE;

        for i in 0..STEPS {
            if x >= X_STEPS[i] {
                x -= X_STEPS[i];
                product = mul_div_down(product, A_STEPS[i], ONE)?;
            }
        }

        let mut series_sum: u128 = ONE;
        let mut term: u128 = x;

        series_sum = series_sum
            .checked_add(term)
            .ok_or_else(|| anyhow!("exp: series_sum overflow"))?;

        for n in 2..=EXP_TAYLOR_TERMS {
            term = mul_div_down(term, x, ONE)? / (n as u128);
            series_sum = series_sum
                .checked_add(term)
                .ok_or_else(|| anyhow!("exp: term add overflow"))?;
        }

        mul_div_down(product, series_sum, ONE)
    }
}

/// `(a * b) / denom` rounded down, with 256-bit intermediate.
pub fn mul_div_down(a: u128, b: u128, denom: u128) -> Result<u128> {
    if denom == 0 {
        return Err(anyhow!("mul_div_down by zero"));
    }
    if a == 0 || b == 0 {
        return Ok(0);
    }
    if let Some(product) = a.checked_mul(b) {
        return Ok(product / denom);
    }
    let (hi, lo) = u128_mul_wide(a, b);
    div_256_by_128(hi, lo, denom)
}

fn u128_mul_wide(a: u128, b: u128) -> (u128, u128) {
    let mask: u128 = u64::MAX as u128;
    let a_lo = a & mask;
    let a_hi = a >> 64;
    let b_lo = b & mask;
    let b_hi = b >> 64;

    let p0 = a_lo * b_lo;
    let p1 = a_hi * b_lo;
    let p2 = a_lo * b_hi;
    let p3 = a_hi * b_hi;

    let mid = (p0 >> 64)
        .wrapping_add(p1 & mask)
        .wrapping_add(p2 & mask);

    let lo = (p0 & mask) | ((mid & mask) << 64);
    let hi = p3
        .wrapping_add(p1 >> 64)
        .wrapping_add(p2 >> 64)
        .wrapping_add(mid >> 64);

    (hi, lo)
}

fn div_256_by_128(hi: u128, lo: u128, denom: u128) -> Result<u128> {
    if denom == 0 {
        return Err(anyhow!("div_256_by_128 by zero"));
    }
    if hi >= denom {
        return Err(anyhow!("div_256_by_128 quotient overflows u128"));
    }
    if hi == 0 {
        return Ok(lo / denom);
    }

    let mask: u128 = u64::MAX as u128;

    if denom <= mask {
        let d1 = (hi << 64) | (lo >> 64);
        let q1 = d1 / denom;
        let r1 = d1 % denom;
        let d2 = (r1 << 64) | (lo & mask);
        let q2 = d2 / denom;
        return Ok((q1 << 64) | q2);
    }

    let mut rem = hi;
    let mut quot: u128 = 0;
    for i in (0..128u32).rev() {
        let bit = (lo >> i) & 1;
        let overflow = rem >> 127;
        rem = (rem << 1) | bit;
        if overflow != 0 || rem >= denom {
            rem -= denom;
            quot |= 1u128 << i;
        }
    }
    Ok(quot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp_zero() {
        assert_eq!(LogExpMath::exp(0).unwrap(), ONE);
    }

    #[test]
    fn exp_one() {
        let result = LogExpMath::exp(ONE as i128).unwrap();
        let expected = 2_718_281_828_459_045_235u128;
        let diff = result.abs_diff(expected);
        assert!(diff < 1_000_000, "exp(1) = {result} vs {expected}");
    }

    #[test]
    fn ln_one() {
        assert_eq!(LogExpMath::ln(ONE).unwrap(), 0);
    }

    #[test]
    fn pow_identity() {
        let r = LogExpMath::pow(ONE * 5, ONE).unwrap();
        let diff = r.abs_diff(ONE * 5);
        assert!(diff < 1_000);
    }

    #[test]
    fn pow_sqrt_four_is_two() {
        let r = LogExpMath::pow(ONE * 4, ONE / 2).unwrap();
        let diff = r.abs_diff(ONE * 2);
        assert!(diff < 1_000_000_000);
    }
}
