//! Swap instruction data + account-meta builder. Mirrors
//! cubic-pool/src/instructions/swap.rs (account list lines 15-56,
//! args lines 159-166).

use super::constants::SWAP_IX_DISCRIMINATOR;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

/// 26-byte swap instruction data:
/// `disc(8) || amount_in_le(8) || minimum_amount_out_le(8) || in_idx(1) || out_idx(1)`.
pub fn encode_swap_ix_data(
    amount_in: u64,
    minimum_amount_out: u64,
    token_in_index: u8,
    token_out_index: u8,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(26);
    out.extend_from_slice(&SWAP_IX_DISCRIMINATOR);
    out.extend_from_slice(&amount_in.to_le_bytes());
    out.extend_from_slice(&minimum_amount_out.to_le_bytes());
    out.push(token_in_index);
    out.push(token_out_index);
    out
}

/// All inputs the connector needs to assemble the 10-account meta list.
pub struct SwapAccounts {
    pub program_id: Pubkey,
    pub pool: Pubkey,
    pub token_mint_in: Pubkey,
    pub token_mint_out: Pubkey,
    pub user_token_in: Pubkey,
    pub user_token_out: Pubkey,
    pub vault_in: Pubkey,
    pub vault_out: Pubkey,
    pub user: Pubkey,
    pub token_program_in: Pubkey,
    pub token_program_out: Pubkey,
}

/// Build the AccountMeta list for the cubic_pool::swap instruction.
///
/// Order taken verbatim from swap.rs and from
/// idl/cubic_pool.json["instructions"]["swap"]["accounts"]:
///   0. pool                 (writable)
///   1. token_mint_in        (read)
///   2. token_mint_out       (read)
///   3. user_token_in        (writable)
///   4. user_token_out       (writable)
///   5. vault_in             (writable)
///   6. vault_out            (writable)
///   7. user                 (signer, writable)
///   8. token_program_in     (read)
///   9. token_program_out    (read)
///
/// For dflow we prepend the program id as a read-only meta at index 0 — this
/// is the convention the runtime uses to identify the target program, and
/// the value reported by `get_accounts_len` includes it.
pub fn build_swap_account_metas(a: &SwapAccounts) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(a.program_id, false),
        AccountMeta::new(a.pool, false),
        AccountMeta::new_readonly(a.token_mint_in, false),
        AccountMeta::new_readonly(a.token_mint_out, false),
        AccountMeta::new(a.user_token_in, false),
        AccountMeta::new(a.user_token_out, false),
        AccountMeta::new(a.vault_in, false),
        AccountMeta::new(a.vault_out, false),
        AccountMeta::new(a.user, true),
        AccountMeta::new_readonly(a.token_program_in, false),
        AccountMeta::new_readonly(a.token_program_out, false),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ix_data_layout() {
        let data = encode_swap_ix_data(1_000_000_000, 990_000_000, 0, 1);
        assert_eq!(data.len(), 26);
        assert_eq!(&data[..8], &SWAP_IX_DISCRIMINATOR);
        assert_eq!(u64::from_le_bytes(data[8..16].try_into().unwrap()), 1_000_000_000);
        assert_eq!(u64::from_le_bytes(data[16..24].try_into().unwrap()), 990_000_000);
        assert_eq!(data[24], 0);
        assert_eq!(data[25], 1);
    }
}
