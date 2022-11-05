use anyhow::Result;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// An abstraction in order to share reserve mints and necessary data
use solana_sdk::{account::Account, instruction::AccountMeta, pubkey::Pubkey};

use jupiter::jupiter_override::SwapLeg;

pub struct QuoteParams {
    pub in_amount: u64,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Quote {
    pub not_enough_liquidity: bool,
    pub min_in_amount: Option<u64>,
    pub min_out_amount: Option<u64>,
    pub in_amount: u64,
    pub out_amount: u64,
    pub fee_amount: u64,
    pub fee_mint: Pubkey,
    pub fee_pct: Decimal,
    pub price_impact_pct: Decimal,
}

pub type QuoteMintToReferrer = HashMap<Pubkey, Pubkey>;

pub struct SwapParams {
    pub source_mint: Pubkey,
    pub destination_mint: Pubkey,
    pub user_source_token_account: Pubkey,
    pub user_destination_token_account: Pubkey,
    pub user_transfer_authority: Pubkey,
    pub open_order_address: Option<Pubkey>,
    pub quote_mint_to_referrer: Option<QuoteMintToReferrer>,
    pub in_amount: u64,
}

pub struct SwapLegAndAccountMetas {
    pub swap_leg: SwapLeg,
    pub account_metas: Vec<AccountMeta>,
}

/// Amm might trigger a setup step for the user
#[derive(Clone)]
pub enum AmmUserSetup {
    SerumDexOpenOrdersSetup { market: Pubkey },
}

pub trait Amm {
    // Amm name
    fn label(&self) -> String;
    // Amm identifier, should be your pool address
    fn key(&self) -> Pubkey;
    // Token mints that the amm supports for swapping
    fn get_reserve_mints(&self) -> Vec<Pubkey>;
    // Accounts related for quoting and creating ix
    fn get_accounts_to_update(&self) -> Vec<Pubkey>;
    // Picks data necessary to update it's internal state
    fn update(&mut self, accounts_map: &HashMap<Pubkey, Vec<u8>>) -> Result<()>;
    // Returns quote for the given quote params
    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote>;

    // Just state how do we make a swap instruction dont have to implement this
    fn get_swap_leg_and_account_metas(
        &self,
        swap_params: &SwapParams,
    ) -> Result<SwapLegAndAccountMetas>;

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync>;

    fn has_dynamic_accounts(&self) -> bool {
        false
    }

    fn get_user_setup(&self) -> Option<AmmUserSetup> {
        None
    }
}

#[derive(Clone)]
pub struct KeyedAccount {
    pub key: Pubkey,
    pub account: Account,
    pub params: Option<serde_json::Value>,
}
