use anchor_lang::prelude::{AccountMeta, Pubkey};

#[derive(Copy, Clone, Debug)]
pub struct GoatTokenSwap {
    pub token_swap_program: Pubkey,
    pub token_program: Pubkey,
    pub swap: Pubkey,
    pub authority: Pubkey,
    pub user_transfer_authority: Pubkey,
    pub source: Pubkey,
    pub swap_source: Pubkey,
    pub swap_destination: Pubkey,
    pub destination: Pubkey,
    pub pool_mint: Pubkey,
    pub pool_fee: Pubkey,
    /// other accounts
    // pub authority: Pubkey,
    pub amm_config: Pubkey,
    // pub pool_state: Pubkey,
    // pub input_token_account: Pubkey,
    // pub output_token_account: Pubkey,
    // pub input_vault: Pubkey,
    // pub output_vault: Pubkey,
    // pub input_token_program: Pubkey,
    pub output_token_program: Pubkey,
    // pub input_token_mint: Pubkey,
    // pub output_token_mint: Pubkey,
}

impl From<GoatTokenSwap> for Vec<AccountMeta> {
    fn from(accounts: GoatTokenSwap) -> Self {
        vec![
            AccountMeta::new_readonly(accounts.token_swap_program, false),
            AccountMeta::new_readonly(accounts.token_program, false),
            AccountMeta::new_readonly(accounts.swap, false),
            AccountMeta::new_readonly(accounts.authority, false),
            AccountMeta::new_readonly(accounts.user_transfer_authority, false),
            AccountMeta::new(accounts.source, false),
            AccountMeta::new(accounts.swap_source, false),
            AccountMeta::new(accounts.swap_destination, false),
            AccountMeta::new(accounts.destination, false),
            AccountMeta::new(accounts.pool_mint, false),
            AccountMeta::new(accounts.pool_fee, false),
            // other accounts
            AccountMeta::new_readonly(accounts.amm_config, false),
            AccountMeta::new(accounts.output_token_program, false),
        ]
    }
}
