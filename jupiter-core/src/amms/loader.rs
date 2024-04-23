use std::collections::HashSet;

use anyhow::{anyhow, Result};
use jupiter_amm_interface::{Amm, KeyedAccount};
use s_jup_interface::SPool;
use solana_sdk::pubkey::Pubkey;

use super::spl_token_swap_amm::{SplTokenSwapAmm, SPL_TOKEN_SWAP_PROGRAMS};

pub fn amm_factory(
    keyed_account: &KeyedAccount,
    _saber_wrapper_mints: &mut HashSet<Pubkey>,
) -> Result<Box<dyn Amm + Send + Sync>> {
    let owner = keyed_account.account.owner;

    // Add your AMM here
    if SPL_TOKEN_SWAP_PROGRAMS.contains_key(&owner) {
        Ok(Box::new(SplTokenSwapAmm::from_keyed_account(
            keyed_account,
        )?))
    } else if keyed_account.account.owner
        == solana_sdk::pubkey!("5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx")
    {
        Ok(Box::new(SPool::from_keyed_account(keyed_account)?))
    } else {
        Err(anyhow!(
            "Unsupported pool {}, from owner {}",
            keyed_account.key,
            keyed_account.account.owner
        ))
    }
}
