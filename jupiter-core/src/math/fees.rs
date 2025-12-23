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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fee_pct_sums_trade_and_owner_fees() {
        // 0.1% trade fee + 0.2% owner fee = 0.3% total
        let fees = Fees::new(1, 1000, 2, 1000);
        let pct = fees.fee_pct().expect("fee_pct should be Some");

        let expected = Decimal::from_f64(0.003).expect("decimal construction should succeed");
        assert_eq!(pct, expected);
    }

    #[test]
    fn fee_pct_handles_zero_denominators_as_zero() {
        // When denominators are zero, the corresponding fee components are treated as 0.
        let fees = Fees::new(1, 0, 2, 0);
        let pct = fees.fee_pct().expect("fee_pct should be Some");

        assert_eq!(pct, Decimal::ZERO);
    }

    #[test]
    fn trading_and_owner_trading_fee_delegate_to_underlying() {
        let amount: u128 = 1_000_000;
        let trade_fee_numerator = 1;
        let trade_fee_denominator = 1_000;
        let owner_trade_fee_numerator = 2;
        let owner_trade_fee_denominator = 1_000;

        let fees = Fees::new(
            trade_fee_numerator,
            trade_fee_denominator,
            owner_trade_fee_numerator,
            owner_trade_fee_denominator,
        );

        let trade_fee = fees
            .trading_fee(amount)
            .expect("trading_fee should be computable");
        let owner_trade_fee = fees
            .owner_trading_fee(amount)
            .expect("owner_trading_fee should be computable");

        // With 0.1% and 0.2% fees respectively, total fee should be amount * 0.003
        let expected_total_fee = amount * 3 / 1_000;
        assert_eq!(trade_fee + owner_trade_fee, expected_total_fee);
    }
}
