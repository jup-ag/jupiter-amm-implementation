//! Off-chain swap simulation engine for MnM DLMM.
//!
//! Exact mirror of on-chain swap logic (sans token transfers).

use crate::state::{BinArray, PoolState, BINS_PER_ARRAY, MAX_BIN_CROSSINGS};

// ─── Q64.64 fixed-point math ────────────────────────────────────────────────

const Q64_ONE: u128 = 1u128 << 64;

fn get_price_from_bin_id(bin_id: i32, bin_step: u16) -> u128 {
    let max_bin = if bin_step == 0 { i32::MAX } else {
        (440000i64 / bin_step as i64).min(100_000) as i32
    };
    if bin_id > max_bin { return u128::MAX; }
    if bin_id < -max_bin { return 1; }
    if bin_id == 0 { return Q64_ONE; }

    let base = Q64_ONE + (Q64_ONE * bin_step as u128) / 10000;
    let result = pow_q64(base, bin_id.unsigned_abs());
    if bin_id < 0 { q64_div(Q64_ONE, result) } else { result }
}

fn pow_q64(base: u128, exp: u32) -> u128 {
    if exp == 0 { return Q64_ONE; }
    let (mut result, mut b, mut e) = (Q64_ONE, base, exp);
    while e > 0 {
        if e & 1 == 1 { result = q64_mul(result, b); }
        b = q64_mul(b, b);
        e >>= 1;
    }
    result
}

fn q64_mul(a: u128, b: u128) -> u128 {
    let (a_lo, a_hi) = (a & 0xFFFFFFFFFFFFFFFF, a >> 64);
    let (b_lo, b_hi) = (b & 0xFFFFFFFFFFFFFFFF, b >> 64);
    let mid = (a_lo * b_hi).saturating_add(a_hi * b_lo).saturating_add((a_lo * b_lo) >> 64);
    (a_hi * b_hi).checked_shl(64).unwrap_or(0).saturating_add(mid)
}

fn q64_div(a: u128, b: u128) -> u128 {
    if b == 0 { return u128::MAX; }
    let (a_hi, a_lo) = (a >> 64, a & 0xFFFFFFFFFFFFFFFF);
    if a_hi == 0 { return (a << 64) / b; }
    let q_unit = u128::MAX / b;
    let r_unit = u128::MAX % b;
    let hi_q = a_hi.checked_mul(q_unit).unwrap_or(u128::MAX);
    let hi_r = a_hi.checked_mul(r_unit.saturating_add(1)).unwrap_or(u128::MAX);
    let lo_result = (a_lo << 64) / b;
    let lo_carry = (a_lo << 64) % b;
    let carry = match hi_r.checked_add(lo_carry) {
        Some(sum) => sum / b,
        None => {
            let q1 = hi_r / b;
            let r1 = hi_r % b;
            q1.saturating_add(r1.saturating_add(lo_carry) / b)
        }
    };
    hi_q.saturating_add(lo_result).saturating_add(carry)
}

fn compute_swap_amount(
    amount_in: u64, liq_x: u64, liq_y: u64, price: u128, swap_x_to_y: bool,
) -> (u64, u64) {
    if swap_x_to_y {
        let out_full = q64_mul(amount_in as u128, price);
        let out = out_full.min(liq_y as u128) as u64;
        let consumed = if out == liq_y {
            (q64_div(liq_y as u128, price) as u64).min(amount_in)
        } else { amount_in };
        (out, consumed)
    } else {
        let out_full = q64_div(amount_in as u128, price) as u64;
        let out = out_full.min(liq_x);
        let consumed = if out == liq_x {
            (q64_mul(liq_x as u128, price) as u64).min(amount_in)
        } else { amount_in };
        (out, consumed)
    }
}

fn compute_fee(amount: u64, fee_bps: u16) -> u64 {
    ((amount as u128 * fee_bps as u128) / 10000) as u64
}

fn next_set_bit_u64(bitmap: u64, from: u32, direction: bool) -> Option<u32> {
    if from >= 64 { return None; }
    if direction {
        let masked = bitmap & (!0u64 << from);
        if masked != 0 { Some(masked.trailing_zeros()) } else { None }
    } else {
        let mask = if from == 63 { !0u64 } else { (1u64 << (from + 1)) - 1 };
        let masked = bitmap & mask;
        if masked != 0 { Some(63 - masked.leading_zeros()) } else { None }
    }
}

// ─── Swap simulation ────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwapDirection {
    XtoY,
    YtoX,
}

#[derive(Clone, Debug)]
pub struct QuoteInput {
    pub amount_in: u64,
    pub direction: SwapDirection,
}

#[derive(Clone, Debug, Default)]
pub struct QuoteOutput {
    pub in_amount: u64,
    pub out_amount: u64,
    pub fee_amount: u64,
    pub fee_pct: f64,
    #[allow(dead_code)]
    pub not_enough_liquidity: bool,
}

/// Simulate a swap across one or more BinArrays.
pub fn simulate_swap_multi(pool: &PoolState, bin_arrays: &[&BinArray], input: &QuoteInput) -> Option<QuoteOutput> {
    let swap_x_to_y = input.direction == SwapDirection::XtoY;
    let total_fee_bps = pool.lp_fee_bps + pool.protocol_fee_bps;

    let mut remaining = input.amount_in;
    let mut total_out: u64 = 0;
    let mut total_fee: u64 = 0;
    let mut crossings: u8 = 0;
    let mut cur_bin = pool.active_bin_id;

    struct SimArray {
        bins: [crate::state::Bin; BINS_PER_ARRAY],
        active_bins: u64,
    }
    let mut sim_arrays: Vec<(i32, SimArray)> = bin_arrays.iter().map(|ba| {
        (ba.index, SimArray { bins: ba.bins, active_bins: ba.active_bins })
    }).collect();

    fn find_array(arrays: &mut [(i32, SimArray)], idx: i32) -> Option<&mut SimArray> {
        arrays.iter_mut().find(|(i, _)| *i == idx).map(|(_, a)| a)
    }

    while remaining > 0 && crossings <= MAX_BIN_CROSSINGS {
        let arr_idx = cur_bin.div_euclid(BINS_PER_ARRAY as i32);
        let arr = match find_array(&mut sim_arrays, arr_idx) {
            Some(a) => a as *mut SimArray,
            None => break,
        };
        let arr = unsafe { &mut *arr };

        let inner = cur_bin.rem_euclid(BINS_PER_ARRAY as i32) as usize;
        let bin = &mut arr.bins[inner];
        let price = get_price_from_bin_id(cur_bin, pool.bin_step);

        let fee = compute_fee(remaining, total_fee_bps);
        let after_fee = remaining.saturating_sub(fee);

        let (out, consumed) = compute_swap_amount(
            after_fee, bin.liquidity_x, bin.liquidity_y, price, swap_x_to_y,
        );

        if swap_x_to_y {
            bin.liquidity_x = bin.liquidity_x.saturating_add(consumed);
            bin.liquidity_y = bin.liquidity_y.saturating_sub(out);
        } else {
            bin.liquidity_y = bin.liquidity_y.saturating_add(consumed);
            bin.liquidity_x = bin.liquidity_x.saturating_sub(out);
        }

        let (fee_actual, consumed_with_fee) = if consumed >= after_fee {
            (fee, remaining)
        } else {
            let f = compute_fee(consumed, total_fee_bps);
            (f, consumed.saturating_add(f))
        };

        total_fee = total_fee.saturating_add(fee_actual);
        total_out = total_out.saturating_add(out);
        remaining = remaining.saturating_sub(consumed_with_fee);

        let exhausted = if swap_x_to_y { bin.liquidity_y == 0 } else { bin.liquidity_x == 0 };
        if exhausted && remaining > 0 {
            crossings += 1;
            if crossings > MAX_BIN_CROSSINGS { break; }

            let idx = cur_bin.rem_euclid(BINS_PER_ARRAY as i32) as u32;
            let dir = !swap_x_to_y;

            let next = if dir {
                next_set_bit_u64(arr.active_bins, idx.saturating_add(1), true)
            } else if idx == 0 {
                None
            } else {
                next_set_bit_u64(arr.active_bins, idx - 1, false)
            };

            match next {
                Some(n) => cur_bin = arr_idx * BINS_PER_ARRAY as i32 + n as i32,
                None => {
                    let next_arr_idx = if dir { arr_idx + 1 } else { arr_idx - 1 };
                    let next_arr = match find_array(&mut sim_arrays, next_arr_idx) {
                        Some(a) => a,
                        None => break,
                    };
                    let start = if dir { 0 } else { 63 };
                    match next_set_bit_u64(next_arr.active_bins, start, dir) {
                        Some(n) => cur_bin = next_arr_idx * BINS_PER_ARRAY as i32 + n as i32,
                        None => break,
                    }
                }
            }
        } else {
            break;
        }
    }

    let in_amount = input.amount_in.saturating_sub(remaining);
    Some(QuoteOutput {
        in_amount,
        out_amount: total_out,
        fee_amount: total_fee,
        not_enough_liquidity: remaining > 0,
        fee_pct: if input.amount_in > 0 { total_fee as f64 / input.amount_in as f64 } else { 0.0 },
    })
}
