use std::num::NonZeroUsize;

use itertools::Itertools;
use solana_client::rpc_client::RpcClient;
use solana_rpc_client_api::client_error::Error as ClientError;
use solana_sdk::{account::Account, pubkey::Pubkey};

pub trait ExtendedSolanaRpcClient {
    fn get_multiple_accounts_chunked(
        &self,
        accounts: &[Pubkey],
        chunk_size: NonZeroUsize,
    ) -> Result<Vec<Option<Account>>, ClientError>;
}

impl ExtendedSolanaRpcClient for RpcClient {
    fn get_multiple_accounts_chunked(
        &self,
        accounts: &[Pubkey],
        chunk_size: NonZeroUsize,
    ) -> Result<Vec<Option<Account>>, ClientError> {
        let chunk_size = chunk_size.get();
        accounts
            .chunks(chunk_size)
            .into_iter()
            .map(|pk_chunk| self.get_multiple_accounts(pk_chunk))
            .flatten_ok()
            .collect()
    }
}
