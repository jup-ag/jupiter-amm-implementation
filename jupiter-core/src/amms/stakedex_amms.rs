use std::collections::HashMap;

use anyhow::{anyhow, Result};
use jupiter_amm_interface::KeyedAccount;
use solana_client::nonblocking::rpc_client::RpcClient;
use stakedex_sdk::test_utils::{
    marinade_program, spl_stake_pool, DepositSolWrapper, MarinadeStakedex, SplStakePoolStakedex,
};
use stakedex_sdk::Stakedex;

use crate::amm::Amm;
pub use stakedex_sdk::stakedex_program_id;

/// Scaffolding of the entire business
pub async fn initialize_stakedex_amms(
    client: &RpcClient,
) -> Result<Vec<Box<dyn Amm + Send + Sync>>> {
    let account_addresses = Stakedex::init_accounts();
    let account_map = client
        .get_multiple_accounts(&account_addresses)
        .await
        .unwrap()
        .into_iter()
        .zip(account_addresses)
        .fold(HashMap::new(), |mut m, (account, address)| {
            if let Some(account) = account {
                m.insert(address, account);
            }
            m
        });
    let (stakedex, errors) = Stakedex::from_fetched_accounts(&account_map);

    if !errors.is_empty() {
        return Err(anyhow!("Errors while initializing Stakedex: {:?}", errors));
    }

    // Create all possible Amms
    Ok(stakedex.get_amms())
}

pub fn init_stakedex_deposit_sol_wrapper_amm(
    keyed_account: &KeyedAccount,
) -> Result<Option<Box<dyn Amm + Send + Sync>>> {
    let owner = keyed_account.account.owner;
    let amm: Box<dyn Amm + Send + Sync> = if owner == spl_stake_pool::ID {
        Box::new(DepositSolWrapper::<SplStakePoolStakedex>::from_keyed_account(keyed_account)?)
    } else if owner == marinade_program::ID {
        Box::new(DepositSolWrapper::<MarinadeStakedex>::from_keyed_account(
            keyed_account,
        )?)
    } else {
        return Ok(None);
    };
    Ok(Some(amm))
}
