//! Quote engine: Q64.64 price math, fee calculation, and bin traversal simulation.
//!
//! All math is ported directly from the on-chain program to ensure exact quote parity.
//!
//! Source of truth:
//!   backend_dlmm/programs/orbit_finance/src/math/price.rs
//!   backend_dlmm/programs/orbit_finance/src/math/fees.rs
//!   backend_dlmm/programs/orbit_finance/src/math/dynamic_fees.rs
//!   backend_dlmm/programs/orbit_finance/src/math/bin_traversal.rs

use anyhow::{bail, Result};
use uint::construct_uint;

use crate::state::{BinArrayState, PoolState};

construct_uint! {
    pub struct U256(4);
}

// ---------------------------------------------------------------------------
// Constants (match on-chain constants.rs)
// ---------------------------------------------------------------------------

pub const BPS_DEN: u128 = 10_000;
pub const Q64_RESOLUTION: u128 = 1u128 << 64;
pub const Q64_SHIFT: u32 = 64;
pub const MAX_BINS_PER_SWAP: usize = 64;

/// Dynamic fee denominator: 1e11
const VAR_FEE_DENOM: u128 = 100_000_000_000;

// ---------------------------------------------------------------------------
// Q64.64 arithmetic
// ---------------------------------------------------------------------------

#[inline]
fn one_q64() -> u128 {
    Q64_RESOLUTION
}

#[inline]
fn mul_q64(a: u128, b: u128) -> Result<u128> {
    let prod = U256::from(a) * U256::from(b);
    let shifted = prod >> Q64_SHIFT;
    if shifted > U256::from(u128::MAX) {
        bail!("mul_q64 overflow");
    }
    Ok(shifted.as_u128())
}

#[inline]
fn div_q64(a: u128, b: u128) -> Result<u128> {
    if b == 0 {
        bail!("div_q64: division by zero");
    }
    let num = U256::from(a) << Q64_SHIFT;
    let q = num / U256::from(b);
    if q > U256::from(u128::MAX) {
        bail!("div_q64 overflow");
    }
    Ok(q.as_u128())
}

/// safe_mul_div: (a * b) / den using U256 intermediate
#[inline]
fn safe_mul_div(a: u128, b: u128, den: u128) -> Result<u128> {
    if den == 0 {
        bail!("safe_mul_div: division by zero");
    }
    let prod = U256::from(a) * U256::from(b);
    let q = prod / U256::from(den);
    if q > U256::from(u128::MAX) {
        bail!("safe_mul_div overflow");
    }
    Ok(q.as_u128())
}

/// step_ratio_q64: 1 + bin_step_bps / 10000 in Q64.64
fn step_ratio_q64(bin_step_bps: u16) -> Result<u128> {
    let addend = safe_mul_div(one_q64(), bin_step_bps as u128, BPS_DEN)?;
    one_q64()
        .checked_add(addend)
        .ok_or_else(|| anyhow::anyhow!("step_ratio overflow"))
}

/// Exponentiation by squaring in Q64.64
fn pow_q64(mut ratio: u128, mut exp: u32) -> Result<u128> {
    let mut result = one_q64();
    while exp > 0 {
        if exp & 1 == 1 {
            result = mul_q64(result, ratio)?;
        }
        exp >>= 1;
        if exp > 0 {
            ratio = mul_q64(ratio, ratio)?;
        }
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Price from bin index
// ---------------------------------------------------------------------------

/// price = (1 + bin_step_bps / 10000) ^ bin_index  in Q64.64
pub fn price_from_bin(bin_index: i32, bin_step_bps: u16) -> Result<u128> {
    if bin_step_bps == 0 {
        bail!("bin_step_bps must be > 0");
    }
    if bin_index == 0 {
        return Ok(one_q64());
    }
    let ratio = step_ratio_q64(bin_step_bps)?;
    let exp = bin_index.unsigned_abs();
    let p = pow_q64(ratio, exp)?;
    if bin_index > 0 {
        Ok(p)
    } else {
        div_q64(one_q64(), p)
    }
}

// ---------------------------------------------------------------------------
// Fee calculation
// ---------------------------------------------------------------------------

/// Calculate fee: amount * fee_bps / 10000
#[inline]
pub fn calculate_fee(amount: u128, fee_bps: u16) -> Result<u128> {
    safe_mul_div(amount, fee_bps as u128, BPS_DEN)
}

/// Compute the effective fee bps (base + variable, capped).
///
/// For the Jupiter adapter we use a simplified approach:
/// we read the current volatility_accumulator from the cached pool state
/// and compute the variable fee without mutating state.
pub fn effective_fee_bps(pool: &PoolState) -> Result<u16> {
    if pool.dynamic_fee_enabled == 0 {
        return Ok(pool.base_fee_bps);
    }

    // Compute variable fee: fv = (va * s)^2 * C / 1e11
    let va = pool.volatility_accumulator as u128;
    let s = pool.bin_step_bps as u128;
    let c = pool.variable_fee_control as u128;

    let x = va.checked_mul(s).ok_or_else(|| anyhow::anyhow!("va*s overflow"))?;
    let x2 = x.checked_mul(x).ok_or_else(|| anyhow::anyhow!("x^2 overflow"))?;
    let fv = x2.checked_mul(c).ok_or_else(|| anyhow::anyhow!("fv overflow"))? / VAR_FEE_DENOM;
    let fv_bps = fv.min(u16::MAX as u128) as u32;

    let total = (pool.base_fee_bps as u32).saturating_add(fv_bps);
    let capped = total.min(pool.max_dynamic_fee_bps as u32);
    Ok(capped as u16)
}

// ---------------------------------------------------------------------------
// Bin traversal conversion
// ---------------------------------------------------------------------------

/// base_in → quote_out at fixed Q64.64 price: floor(base * price / 2^64)
#[inline]
fn base_to_quote_at_price(base_in: u128, price_q64: u128) -> Result<u128> {
    let prod = U256::from(base_in) * U256::from(price_q64);
    let out = prod >> Q64_SHIFT;
    if out > U256::from(u128::MAX) {
        bail!("base_to_quote overflow");
    }
    Ok(out.as_u128())
}

/// quote_in → base_out at fixed Q64.64 price: floor(quote * 2^64 / price)
#[inline]
fn quote_to_base_at_price(quote_in: u128, price_q64: u128) -> Result<u128> {
    if price_q64 == 0 {
        bail!("price is zero");
    }
    let num = U256::from(quote_in) << Q64_SHIFT;
    let q = num / U256::from(price_q64);
    if q > U256::from(u128::MAX) {
        bail!("quote_to_base overflow");
    }
    Ok(q.as_u128())
}

/// Fill a single bin Base→Quote: user provides base, bin provides quote from reserve.
fn fill_bin_b2q(
    base_remaining: u128,
    reserve_quote: u128,
    price_q64: u128,
) -> Result<(u128, u128)> {
    if base_remaining == 0 || reserve_quote == 0 {
        return Ok((0, 0));
    }
    // max base the bin can absorb given its quote reserve
    let max_base = quote_to_base_at_price(reserve_quote, price_q64)?;
    let fill_base = base_remaining.min(max_base);
    let out_quote = base_to_quote_at_price(fill_base, price_q64)?;
    Ok((fill_base, out_quote))
}

/// Fill a single bin Quote→Base: user provides quote, bin provides base from reserve.
fn fill_bin_q2b(
    quote_remaining: u128,
    reserve_base: u128,
    price_q64: u128,
) -> Result<(u128, u128)> {
    if quote_remaining == 0 || reserve_base == 0 {
        return Ok((0, 0));
    }
    // max quote the bin can absorb given its base reserve
    let max_quote = base_to_quote_at_price(reserve_base, price_q64)?;
    let fill_quote = quote_remaining.min(max_quote);
    let out_base = quote_to_base_at_price(fill_quote, price_q64)?;
    Ok((fill_quote, out_base))
}

// ---------------------------------------------------------------------------
// Swap direction
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwapDirection {
    BaseToQuote,
    QuoteToBase,
}

// ---------------------------------------------------------------------------
// Quote simulation result
// ---------------------------------------------------------------------------

pub struct QuoteResult {
    pub in_amount: u64,
    pub out_amount: u64,
    pub fee_amount: u64,
    /// BinArray lower_bin_indices touched during traversal (for remaining_accounts).
    pub bin_arrays_touched: Vec<i32>,
}

// ---------------------------------------------------------------------------
// Main quote function: ExactIn
// ---------------------------------------------------------------------------

/// Simulate an ExactIn swap and return (out_amount, fee_amount, bin_arrays_touched).
///
/// Fee domain rules (matching on-chain):
///   BaseToQuote: fee charged ON OUTPUT (quote domain)
///   QuoteToBase: fee charged ON INPUT (quote domain, deducted before traversal)
pub fn quote_exact_in(
    pool: &PoolState,
    bin_arrays: &ahash::HashMap<i32, BinArrayState>,
    amount_in: u64,
    direction: SwapDirection,
) -> Result<QuoteResult> {
    if amount_in == 0 {
        return Ok(QuoteResult {
            in_amount: 0,
            out_amount: 0,
            fee_amount: 0,
            bin_arrays_touched: vec![],
        });
    }

    let fee_bps = effective_fee_bps(pool)?;
    let mut remaining: u128;
    let total_fee: u128;
    let mut total_out: u128 = 0;
    let mut touched: Vec<i32> = Vec::new();

    match direction {
        SwapDirection::QuoteToBase => {
            // Fee on input: deduct fee first
            let fee = calculate_fee(amount_in as u128, fee_bps)?;
            remaining = (amount_in as u128).saturating_sub(fee);
            total_fee = fee;

            // Walk UP from active_bin
            let mut bin_idx = pool.active_bin;
            let mut bins_visited = 0usize;

            while remaining > 0 && bins_visited < MAX_BINS_PER_SWAP {
                let lbi = BinArrayState::lower_bin_index_from(bin_idx);
                if let Some(ba) = bin_arrays.get(&lbi) {
                    if !touched.contains(&lbi) {
                        touched.push(lbi);
                    }
                    if let Some(bin) = ba.get_bin(bin_idx) {
                        if !bin.is_empty() {
                            let price = price_from_bin(bin_idx, pool.bin_step_bps)?;
                            let (fill_quote, out_base) =
                                fill_bin_q2b(remaining, bin.reserve_base, price)?;
                            remaining = remaining.saturating_sub(fill_quote);
                            total_out += out_base;
                        }
                    }
                }
                bin_idx += 1;
                bins_visited += 1;
            }
        }
        SwapDirection::BaseToQuote => {
            // Fee on output: traverse first, then deduct
            remaining = amount_in as u128;

            // Walk DOWN from active_bin
            let mut bin_idx = pool.active_bin;
            let mut bins_visited = 0usize;
            let mut gross_out: u128 = 0;

            while remaining > 0 && bins_visited < MAX_BINS_PER_SWAP {
                let lbi = BinArrayState::lower_bin_index_from(bin_idx);
                if let Some(ba) = bin_arrays.get(&lbi) {
                    if !touched.contains(&lbi) {
                        touched.push(lbi);
                    }
                    if let Some(bin) = ba.get_bin(bin_idx) {
                        if !bin.is_empty() {
                            let price = price_from_bin(bin_idx, pool.bin_step_bps)?;
                            let (fill_base, out_quote) =
                                fill_bin_b2q(remaining, bin.reserve_quote, price)?;
                            remaining = remaining.saturating_sub(fill_base);
                            gross_out += out_quote;
                        }
                    }
                }
                bin_idx -= 1;
                bins_visited += 1;
            }

            // Deduct fee from output
            let fee = calculate_fee(gross_out, fee_bps)?;
            total_out = gross_out.saturating_sub(fee);
            total_fee = fee;
        }
    }

    Ok(QuoteResult {
        in_amount: amount_in,
        out_amount: total_out.min(u64::MAX as u128) as u64,
        fee_amount: total_fee.min(u64::MAX as u128) as u64,
        bin_arrays_touched: touched,
    })
}

// ---------------------------------------------------------------------------
// Main quote function: ExactOut
// ---------------------------------------------------------------------------

/// Simulate an ExactOut swap: compute the input needed to produce exactly `amount_out`.
///
/// This is an approximation — we binary-search on ExactIn to find the input that
/// yields >= amount_out. Accurate to ±1 unit.
pub fn quote_exact_out(
    pool: &PoolState,
    bin_arrays: &ahash::HashMap<i32, BinArrayState>,
    amount_out: u64,
    direction: SwapDirection,
) -> Result<QuoteResult> {
    if amount_out == 0 {
        return Ok(QuoteResult {
            in_amount: 0,
            out_amount: 0,
            fee_amount: 0,
            bin_arrays_touched: vec![],
        });
    }

    // Binary search: find minimum input that produces >= amount_out
    let mut lo: u64 = amount_out; // optimistic lower bound (1:1)
    let mut hi: u64 = amount_out.saturating_mul(3).max(amount_out.saturating_add(1_000_000));
    // Cap to prevent infinite search
    if hi < lo {
        hi = u64::MAX / 2;
    }

    // Expand hi if needed
    for _ in 0..10 {
        let q = quote_exact_in(pool, bin_arrays, hi, direction)?;
        if q.out_amount >= amount_out {
            break;
        }
        hi = hi.saturating_mul(2);
        if hi == 0 {
            bail!("ExactOut: cannot find sufficient input");
        }
    }

    // Binary search
    for _ in 0..64 {
        if lo >= hi {
            break;
        }
        let mid = lo + (hi - lo) / 2;
        let q = quote_exact_in(pool, bin_arrays, mid, direction)?;
        if q.out_amount >= amount_out {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }

    let result = quote_exact_in(pool, bin_arrays, hi, direction)?;
    Ok(QuoteResult {
        in_amount: hi,
        out_amount: result.out_amount,
        fee_amount: result.fee_amount,
        bin_arrays_touched: result.bin_arrays_touched,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_at_bin_zero_is_one() {
        let p = price_from_bin(0, 10).unwrap();
        assert_eq!(p, Q64_RESOLUTION);
    }

    #[test]
    fn price_at_positive_bin_is_greater() {
        let p = price_from_bin(1, 100).unwrap(); // 1% step
        assert!(p > Q64_RESOLUTION);
        // Should be ~1.01 * 2^64
        let expected_approx = Q64_RESOLUTION + Q64_RESOLUTION / 100;
        let diff = if p > expected_approx {
            p - expected_approx
        } else {
            expected_approx - p
        };
        // Allow 1 unit of Q64 rounding
        assert!(diff <= 1, "price off by {diff}");
    }

    #[test]
    fn price_at_negative_bin_is_less() {
        let p = price_from_bin(-1, 100).unwrap();
        assert!(p < Q64_RESOLUTION);
    }

    #[test]
    fn fee_calculation() {
        // 1,000,000 amount * 25 bps = 2,500
        let fee = calculate_fee(1_000_000, 25).unwrap();
        assert_eq!(fee, 2_500);
    }

    #[test]
    fn effective_fee_static() {
        let mut pool: PoolState = bytemuck::Zeroable::zeroed();
        pool.base_fee_bps = 30;
        pool.dynamic_fee_enabled = 0;
        assert_eq!(effective_fee_bps(&pool).unwrap(), 30);
    }

    #[test]
    fn effective_fee_dynamic() {
        let mut pool: PoolState = bytemuck::Zeroable::zeroed();
        pool.base_fee_bps = 10;
        pool.dynamic_fee_enabled = 1;
        pool.volatility_accumulator = 100;
        pool.bin_step_bps = 10;
        pool.variable_fee_control = 100;
        pool.max_dynamic_fee_bps = 500;
        // fv = (va * s)^2 * C / 1e11
        // = (100 * 10)^2 * 100 / 1e11 = 1_000_000 * 100 / 1e11 = 0 (rounds to 0 bps)
        // total = base + fv = 10 + 0 = 10, capped at 500
        let fee = effective_fee_bps(&pool).unwrap();
        assert!(fee >= 10 && fee <= 500, "fee={fee}");
    }

    #[test]
    fn effective_fee_capped() {
        let mut pool: PoolState = bytemuck::Zeroable::zeroed();
        pool.base_fee_bps = 100;
        pool.dynamic_fee_enabled = 1;
        pool.volatility_accumulator = 50_000;
        pool.bin_step_bps = 100;
        pool.variable_fee_control = 1_000_000;
        pool.max_dynamic_fee_bps = 200;
        let fee = effective_fee_bps(&pool).unwrap();
        assert_eq!(fee, 200, "fee should be capped at max_dynamic_fee_bps");
    }

    #[test]
    fn price_symmetry() {
        // price(n) * price(-n) ≈ 1.0 in Q64.64
        let p_pos = price_from_bin(10, 50).unwrap();
        let p_neg = price_from_bin(-10, 50).unwrap();
        let product = mul_q64(p_pos, p_neg).unwrap();
        let diff = if product > Q64_RESOLUTION {
            product - Q64_RESOLUTION
        } else {
            Q64_RESOLUTION - product
        };
        // Allow small rounding error (< 0.0001%)
        let tolerance = Q64_RESOLUTION / 1_000_000;
        assert!(diff < tolerance, "product off by {diff}, tolerance {tolerance}");
    }

    #[test]
    fn price_monotonically_increasing() {
        let p0 = price_from_bin(0, 25).unwrap();
        let p1 = price_from_bin(1, 25).unwrap();
        let p2 = price_from_bin(2, 25).unwrap();
        assert!(p0 < p1);
        assert!(p1 < p2);

        let pn1 = price_from_bin(-1, 25).unwrap();
        let pn2 = price_from_bin(-2, 25).unwrap();
        assert!(pn1 < p0);
        assert!(pn2 < pn1);
    }

    /// Helper: create a minimal pool and bin_arrays for quote testing
    fn test_pool_and_bins(
        bin_step_bps: u16,
        base_fee_bps: u16,
        active_bin: i32,
        bins: Vec<(i32, u128, u128)>, // (bin_index, reserve_base, reserve_quote)
    ) -> (PoolState, ahash::HashMap<i32, BinArrayState>) {
        use crate::state::BIN_ARRAY_SIZE;

        let mut pool: PoolState = bytemuck::Zeroable::zeroed();
        pool.bin_step_bps = bin_step_bps;
        pool.base_fee_bps = base_fee_bps;
        pool.active_bin = active_bin;
        pool.dynamic_fee_enabled = 0;

        let mut bin_arrays: ahash::HashMap<i32, BinArrayState> = ahash::HashMap::default();

        for (idx, rb, rq) in &bins {
            let lbi = BinArrayState::lower_bin_index_from(*idx);
            let ba = bin_arrays
                .entry(lbi)
                .or_insert_with(|| {
                    let mut ba: BinArrayState = bytemuck::Zeroable::zeroed();
                    ba.lower_bin_index = lbi;
                    ba
                });

            let offset = (*idx - lbi) as usize;
            assert!(offset < BIN_ARRAY_SIZE);
            ba.bins[offset].reserve_base = *rb;
            ba.bins[offset].reserve_quote = *rq;
            ba.bins[offset].total_shares = 1; // non-empty
        }

        (pool, bin_arrays)
    }

    #[test]
    fn quote_exact_in_zero_amount() {
        let (pool, bins) = test_pool_and_bins(10, 30, 0, vec![]);
        let result = quote_exact_in(&pool, &bins, 0, SwapDirection::BaseToQuote).unwrap();
        assert_eq!(result.in_amount, 0);
        assert_eq!(result.out_amount, 0);
        assert_eq!(result.fee_amount, 0);
    }

    #[test]
    fn quote_exact_in_single_bin_b2q() {
        // Bin 0 with price = 1.0, lots of quote reserve
        let (pool, bins) = test_pool_and_bins(
            10, 100, 0,
            vec![(0, 0, 1_000_000_000)],
        );

        let result = quote_exact_in(&pool, &bins, 1_000_000, SwapDirection::BaseToQuote).unwrap();
        assert_eq!(result.in_amount, 1_000_000);
        // At price 1.0 (bin 0), 1M base → ~1M quote gross, minus 1% fee
        assert!(result.out_amount > 0, "should get some output");
        assert!(result.fee_amount > 0, "should charge fee");
        // Net output + fee ≈ gross output
        let gross = result.out_amount as u128 + result.fee_amount as u128;
        // At price 1.0, gross should be close to 1M
        assert!(
            (gross as i128 - 1_000_000i128).unsigned_abs() < 100,
            "gross={gross}, expected ~1000000"
        );
    }

    #[test]
    fn quote_exact_in_single_bin_q2b() {
        // Bin 0 with price = 1.0, lots of base reserve
        let (pool, bins) = test_pool_and_bins(
            10, 100, 0,
            vec![(0, 1_000_000_000, 0)],
        );

        let result = quote_exact_in(&pool, &bins, 1_000_000, SwapDirection::QuoteToBase).unwrap();
        assert_eq!(result.in_amount, 1_000_000);
        // Fee charged on input: 1% of 1M = 10K, remaining 990K → ~990K base
        assert!(result.out_amount > 0);
        assert!(result.fee_amount > 0);
        assert_eq!(result.fee_amount, 10_000, "1% of 1M = 10K");
    }

    #[test]
    fn quote_exact_out_roundtrip() {
        let (pool, bins) = test_pool_and_bins(
            10, 50, 0,
            vec![(0, 1_000_000_000, 1_000_000_000)],
        );

        let desired_out = 100_000u64;
        let result = quote_exact_out(&pool, &bins, desired_out, SwapDirection::BaseToQuote).unwrap();
        // out_amount should be >= desired_out
        assert!(
            result.out_amount >= desired_out,
            "out_amount={} < desired={}",
            result.out_amount, desired_out
        );
    }

    #[test]
    fn quote_multi_bin_traversal() {
        // Set up bins at indices -1, 0, 1 with reserves
        let (pool, bins) = test_pool_and_bins(
            100, 50, 0,
            vec![
                (0, 1_000, 1_000),
                (-1, 1_000, 1_000),
                (1, 1_000, 1_000),
            ],
        );

        // Large B2Q swap that should cross bins (walking DOWN)
        let result = quote_exact_in(&pool, &bins, 3_000, SwapDirection::BaseToQuote).unwrap();
        assert!(result.out_amount > 0);
        // Should touch bin 0 and potentially bin -1
        assert!(!result.bin_arrays_touched.is_empty());
    }
}
