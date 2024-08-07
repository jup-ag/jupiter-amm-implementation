use anyhow::{Context, Result};
use jupiter_amm_interface::SwapMode;

use crate::{curve::{CurveCalculator, Fees}, math::token_swap::SwapResult};

pub fn get_swap_curve_result(
    amount: u64,
    swap_source_amount: u128,
    swap_destination_amount: u128,
    trade_fee_rate: u64,
    protocol_fee_rate: u64,
    fund_fee_rate: u64,
    lp_fee_rate: u64,
    swap_mode: SwapMode,
) -> Result<SwapResult> {
    if swap_mode == SwapMode::ExactIn {
        let curve_result = CurveCalculator::swap_base_input(
            u128::from(amount),
            swap_source_amount,
            swap_destination_amount,
            trade_fee_rate,
            protocol_fee_rate,
            fund_fee_rate,
            lp_fee_rate,
        )
        .unwrap();

        let fee_pct =
            Fees::fee_pct(trade_fee_rate + lp_fee_rate).context("failed to get fee pct")?;

        return Ok(SwapResult {
            expected_output_amount: curve_result.destination_amount_swapped,
            fees: curve_result
                .trade_fee
                .checked_add(curve_result.lp_fee)
                .unwrap(),
            input_amount: curve_result.source_amount_swapped,
            fee_pct,
            ..Default::default()
        });
    }

    let curve_result = CurveCalculator::swap_base_output(
        u128::from(amount),
        swap_source_amount,
        swap_destination_amount,
        trade_fee_rate,
        protocol_fee_rate,
        fund_fee_rate,
        lp_fee_rate,
    )
    .unwrap();

    let fee_pct = Fees::fee_pct(trade_fee_rate + lp_fee_rate).context("failed to get fee pct")?;

    Ok(SwapResult {
        expected_output_amount: curve_result.destination_amount_swapped,
        fees: curve_result
            .trade_fee
            .checked_add(curve_result.lp_fee)
            .unwrap(),
        input_amount: curve_result.source_amount_swapped,
        fee_pct,
        ..Default::default()
    })
}
