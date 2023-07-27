use rust_decimal::{prelude::FromPrimitive, Decimal};
use spl_token_swap::curve::fees::Fees as TokenSwapFees;

#[derive(Clone, Debug, Default)]
pub struct Fees(TokenSwapFees);

impl Fees {
    pub fn new(
        trade_fee_numerator: u64,
        trade_fee_denominator: u64,
        owner_trade_fee_numerator: u64,
        owner_trade_fee_denominator: u64,
    ) -> Self {
        Self(TokenSwapFees {
            trade_fee_numerator,
            trade_fee_denominator,
            owner_trade_fee_numerator,
            owner_trade_fee_denominator,
            ..Default::default()
        })
    }

    pub fn trading_fee(&self, amount: u128) -> Option<u128> {
        self.0.trading_fee(amount)
    }

    pub fn owner_trading_fee(&self, amount: u128) -> Option<u128> {
        self.0.owner_trading_fee(amount)
    }

    pub fn fee_pct(&self) -> Option<Decimal> {
        let trade_fee_pct = if self.0.trade_fee_denominator > 0 {
            Decimal::from_u64(self.0.trade_fee_numerator)?
                .checked_div(Decimal::from_u64(self.0.trade_fee_denominator)?)?
        } else {
            Decimal::ZERO
        };

        let owner_trade_fee_pct = if self.0.owner_trade_fee_denominator > 0 {
            Decimal::from_u64(self.0.owner_trade_fee_numerator)?
                .checked_div(Decimal::from_u64(self.0.owner_trade_fee_denominator)?)?
        } else {
            Decimal::ZERO
        };
        trade_fee_pct.checked_add(owner_trade_fee_pct)
    }
}
