use anchor_lang::{prelude::AccountMeta, ToAccountMetas};
pub use jupiter_amm_interface::{
    single_program_amm, try_get_account_data, AccountMap, Amm, AmmContext, AmmLabel,
    AmmProgramIdToLabel, KeyedAccount, KeyedUiAccount, Quote, QuoteParams, Side, SingleProgramAmm,
    Swap, SwapAndAccountMetas, SwapMode, SwapParams,
};
use solana_sdk::pubkey::Pubkey;

pub fn to_dex_account_metas(program_id: Pubkey, accounts: impl ToAccountMetas) -> Vec<AccountMeta> {
    let mut account_metas = vec![AccountMeta::new_readonly(program_id, false)];
    account_metas.extend(accounts.to_account_metas(None));
    account_metas
}
