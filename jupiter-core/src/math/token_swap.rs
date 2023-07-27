use rust_decimal::Decimal;

pub trait TokenSwap {
    fn exchange(
        &self,
        token_amounts: &[u128],
        in_amount: u128,
        input_index: usize,
        output_index: Option<usize>,
    ) -> Option<SwapResult>;
}

#[derive(Debug, Clone, Default)]
pub struct SwapResult {
    pub fee_pct: Decimal,
    pub fees: u128,
    pub input_amount: u128,
    pub expected_output_amount: u128,
    pub not_enough_liquidity: bool,
}
