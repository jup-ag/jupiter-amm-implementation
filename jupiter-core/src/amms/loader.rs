use std::collections::HashSet;

use anyhow::{anyhow, Result};
use jupiter_amm_interface::{Amm, AmmContext, KeyedAccount};
use solana_sdk::pubkey::Pubkey;

use super::spl_token_swap_amm::{SplTokenSwapAmm, SPL_TOKEN_SWAP_PROGRAMS};

pub fn amm_factory(
    keyed_account: &KeyedAccount,
    amm_context: &AmmContext,
    _saber_wrapper_mints: &mut HashSet<Pubkey>,
) -> Result<Box<dyn Amm + Send + Sync>> {
    let owner = keyed_account.account.owner;

    // Add your AMM here
    if SPL_TOKEN_SWAP_PROGRAMS.contains_key(&owner) {
        Ok(Box::new(SplTokenSwapAmm::from_keyed_account(
            keyed_account,
            amm_context,
        )?))
    } else {
        Err(anyhow!(
            "Unsupported pool {}, from owner {}",
            keyed_account.key,
            keyed_account.account.owner
        ))
    }
}
