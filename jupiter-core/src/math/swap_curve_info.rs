use anyhow::{Context, Result};
use spl_token_swap::curve::{
    base::SwapCurve, calculator::TradeDirection, fees::Fees as TokenSwapFees,
};

use super::{fees::Fees, token_swap::SwapResult};

pub fn get_swap_curve_result(
    swap_curve: &SwapCurve,
    amount: u64,
    swap_source_amount: u128,
    swap_destination_amount: u128,
    trade_direction: TradeDirection,
    fees: &TokenSwapFees,
) -> Result<SwapResult> {
    let curve_result = swap_curve
        .swap(
            amount.into(),
            swap_source_amount,
            swap_destination_amount,
            trade_direction,
            fees,
        )
        .context("quote failed")?;

    let fees = Fees::new(
        fees.trade_fee_numerator,
        fees.trade_fee_denominator,
        fees.owner_trade_fee_numerator,
        fees.owner_trade_fee_denominator,
    );
    let fee_pct = fees.fee_pct().context("failed to get fee pct")?;

    Ok(SwapResult {
        expected_output_amount: curve_result.destination_amount_swapped,
        fees: curve_result.trade_fee + curve_result.owner_fee,
        input_amount: curve_result.source_amount_swapped,
        fee_pct,
        ..Default::default()
    })
}
