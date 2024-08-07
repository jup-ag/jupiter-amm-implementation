//! Swap calculations

use crate::curve::{constant_product::ConstantProductCurve, fees::Fees};
use std::fmt::Debug;

/// The direction to round.  Used for pool token to trading token conversions to
/// avoid losing value on any deposit or withdrawal.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RoundDirection {
    /// Floor the value, ie. 1.9 => 1, 1.1 => 1, 1.5 => 1
    Floor,
    /// Ceiling the value, ie. 1.9 => 2, 1.1 => 2, 1.5 => 2
    Ceiling,
}

/// Encodes results of depositing both sides at once
#[derive(Debug, PartialEq)]
pub struct TradingTokenResult {
    /// Amount of token A
    pub token_0_amount: u128,
    /// Amount of token B
    pub token_1_amount: u128,
}

/// Encodes all results of swapping from a source token to a destination token
#[derive(Debug, PartialEq)]
pub struct SwapResult {
    /// New amount of source token
    pub new_swap_source_amount: u128,
    /// New amount of destination token
    pub new_swap_destination_amount: u128,
    /// Amount of source token swapped (includes fees)
    pub source_amount_swapped: u128,
    /// Amount of destination token swapped
    pub destination_amount_swapped: u128,
    /// Amount of source tokens going to pool holders
    pub trade_fee: u128,
    /// Amount of source tokens going to protocol
    pub protocol_fee: u128,
    /// Amount of source tokens going to protocol team
    pub fund_fee: u128,
    /// Amount of source tokens going to lp
    pub lp_fee: u128,
}

/// Concrete struct to wrap around the trait object which performs calculation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurveCalculator {}

impl CurveCalculator {
    /// Subtract fees and calculate how much destination token will be provided
    /// given an amount of source token.
    pub fn swap_base_input(
        source_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
        trade_fee_rate: u64,
        protocol_fee_rate: u64,
        fund_fee_rate: u64,
        lp_fee_rate: u64,
    ) -> Option<SwapResult> {
        let trade_fee_with_lp_fee_rate = trade_fee_rate.checked_add(lp_fee_rate).unwrap();
        // debit the fee to calculate the amount swapped
        let trade_fee_with_lp_fee = Fees::trading_fee(source_amount, trade_fee_with_lp_fee_rate)?;
        let trade_fee = Fees::trading_fee(source_amount, trade_fee_rate)?;
        let protocol_fee = Fees::protocol_fee(trade_fee, protocol_fee_rate)?;
        let fund_fee = Fees::fund_fee(trade_fee, fund_fee_rate)?;

        let lp_fee = trade_fee_with_lp_fee.checked_sub(trade_fee)?;

        let source_amount_less_fees = source_amount.checked_sub(trade_fee_with_lp_fee)?;

        let destination_amount_swapped = ConstantProductCurve::swap_base_input_without_fees(
            source_amount_less_fees,
            swap_source_amount,
            swap_destination_amount,
        );

        Some(SwapResult {
            new_swap_source_amount: swap_source_amount.checked_add(source_amount)?,
            new_swap_destination_amount: swap_destination_amount
                .checked_sub(destination_amount_swapped)?,
            source_amount_swapped: source_amount,
            destination_amount_swapped,
            trade_fee,
            protocol_fee,
            fund_fee,
            lp_fee,
        })
    }

    pub fn swap_base_output(
        destinsation_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
        trade_fee_rate: u64,
        protocol_fee_rate: u64,
        fund_fee_rate: u64,
        lp_fee_rate: u64,
    ) -> Option<SwapResult> {
        let source_amount_swapped = ConstantProductCurve::swap_base_output_without_fees(
            destinsation_amount,
            swap_source_amount,
            swap_destination_amount,
        );

        let trade_fee_with_lp_fee_rate = trade_fee_rate.checked_add(lp_fee_rate).unwrap();
        let source_amount =
            Fees::calculate_pre_fee_amount(source_amount_swapped, trade_fee_with_lp_fee_rate)
                .unwrap();

        let trade_fee_with_lp_fee = Fees::trading_fee(source_amount, trade_fee_with_lp_fee_rate)?;
        let trade_fee = Fees::trading_fee(source_amount, trade_fee_rate)?;
        let protocol_fee = Fees::protocol_fee(trade_fee, protocol_fee_rate)?;
        let fund_fee = Fees::fund_fee(trade_fee, fund_fee_rate)?;

        let lp_fee = trade_fee_with_lp_fee.checked_sub(trade_fee)?;

        Some(SwapResult {
            new_swap_source_amount: swap_source_amount.checked_add(source_amount)?,
            new_swap_destination_amount: swap_destination_amount
                .checked_sub(destinsation_amount)?,
            source_amount_swapped: source_amount,
            destination_amount_swapped: destinsation_amount,
            trade_fee,
            protocol_fee,
            fund_fee,
            lp_fee,
        })
    }

    /// Get the amount of trading tokens for the given amount of pool tokens,
    /// provided the total trading tokens and supply of pool tokens.
    pub fn lp_tokens_to_trading_tokens(
        lp_token_amount: u128,
        lp_token_supply: u128,
        swap_token_0_amount: u128,
        swap_token_1_amount: u128,
        round_direction: RoundDirection,
    ) -> Option<TradingTokenResult> {
        ConstantProductCurve::lp_tokens_to_trading_tokens(
            lp_token_amount,
            lp_token_supply,
            swap_token_0_amount,
            swap_token_1_amount,
            round_direction,
        )
    }
}
