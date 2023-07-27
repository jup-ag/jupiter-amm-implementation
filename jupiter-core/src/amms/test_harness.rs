use anyhow::Result;
use jupiter_amm_interface::{AccountMap, Amm, KeyedAccount};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

use crate::config;

pub struct AmmTestHarness {
    pub client: RpcClient,
}

impl AmmTestHarness {
    pub fn new() -> Self {
        Self {
            client: RpcClient::new(config::RPC_URL),
        }
    }

    pub fn get_keyed_account(&self, key: Pubkey) -> Result<KeyedAccount> {
        let account = self.client.get_account(&key)?;
        Ok(KeyedAccount {
            key,
            account,
            params: None,
        })
    }

    pub fn update_amm(&self, amm: &mut dyn Amm) {
        let accounts_to_update = amm.get_accounts_to_update();
        println!("{:?}", accounts_to_update);

        let accounts_map: AccountMap = self
            .client
            .get_multiple_accounts(&accounts_to_update)
            .unwrap()
            .iter()
            .enumerate()
            .fold(HashMap::new(), |mut m, (index, account)| {
                if let Some(account) = account {
                    m.insert(accounts_to_update[index], account.clone().into());
                }
                m
            });
        amm.update(&accounts_map).unwrap();
    }
}
