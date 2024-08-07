use anchor_lang::prelude::*;

#[account]
#[derive(Default, Debug)]
pub struct PoolState {
    pub auth_bump: u8,
    pub status: u8,

    pub lp_mint_decimals: u8,
    pub mint_0_decimals: u8,
    pub mint_1_decimals: u8,

    pub amm_config: Pubkey,
    pub pool_creator: Pubkey,
    pub token_0_vault: Pubkey,
    pub token_1_vault: Pubkey,

    pub lp_mint: Pubkey,
    pub token_0_mint: Pubkey,
    pub token_1_mint: Pubkey,

    pub token_0_program: Pubkey,
    pub token_1_program: Pubkey,

    pub lp_supply: u64,
    pub protocol_fees_token_0: u64,
    pub protocol_fees_token_1: u64,

    pub fund_fees_token_0: u64,
    pub fund_fees_token_1: u64,

    pub open_time: u64,

    pub tax_mint: Pubkey,
    pub tax_authority: Pubkey,
    pub in_tax_rate: u64,
    pub out_tax_rate: u64,
    pub tax_amount_0: u64,
    pub tax_amount_1: u64,
    pub tax_disabled: bool,

    pub lp_fee_rate: u64,

    pub padding: [u64; 31],
}

impl PoolState {
    pub const LEN: usize = 8 + 1 * 5 + 9 * 32 + 8 * 6 + 32 * 2 + 8 * 4 + 1 + 8 + 8 * 31;

    pub fn vault_amount_without_fee(&self, vault_0: u64, vault_1: u64) -> (u64, u64) {
        (
            vault_0
                .checked_sub(
                    self.protocol_fees_token_0 + self.fund_fees_token_0 + self.tax_amount_0,
                )
                .unwrap(),
            vault_1
                .checked_sub(
                    self.protocol_fees_token_1 + self.fund_fees_token_1 + self.tax_amount_1,
                )
                .unwrap(),
        )
    }
}

#[account]
#[derive(Default, Debug)]
pub struct AmmConfig {
    pub bump: u8,
    pub disable_create_pool: bool,
    pub index: u16,
    pub trade_fee_rate: u64,
    pub protocol_fee_rate: u64,
    pub fund_fee_rate: u64,
    pub create_pool_fee: u64,
    pub protocol_owner: Pubkey,
    pub fund_owner: Pubkey,

    pub padding: [u64; 16],
}

impl AmmConfig {
    pub const LEN: usize = 8 + 1 + 1 + 2 + 4 * 8 + 32 * 2 + 8 * 16;
}
