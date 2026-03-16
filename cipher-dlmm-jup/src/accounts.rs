//! Swap instruction AccountMeta builder.
//!
//! Produces the exact account ordering required by the on-chain `swap` instruction.
//!
//! Source of truth: backend_dlmm/programs/orbit_finance/src/instructions/swap.rs

use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

use crate::math::{QuoteResult, SwapDirection};
use crate::pda::{bin_array_pda, oracle_pda};
use crate::state::PoolState;

/// CIPHER token mint (hardcoded constant matching on-chain).
/// Address: Ciphern9cCXtms66s8Mm6wCFC27b2JProRQLYmiLMH3N
pub const CIPHER_MINT: Pubkey = Pubkey::new_from_array([
    174, 39, 78, 17, 82, 201, 47, 182, 137, 13, 178, 98, 249, 89, 219, 202, 80, 217, 137, 158,
    98, 49, 191, 2, 12, 224, 28, 24, 82, 210, 14, 57,
]);

/// SPL Token program ID.
const TOKEN_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

/// Build the full list of AccountMetas for the swap instruction.
///
/// Account order (must match on-chain Swap context exactly):
///   0.  pool               (writable)
///   1.  user               (signer)
///   2.  user_source        (writable)
///   3.  user_destination   (writable)
///   4.  base_vault         (writable)
///   5.  quote_vault        (writable)
///   6.  protocol_fee_vault (writable)
///   7.  creator_fee_vault  (writable)
///   8.  holders_fee_vault  (writable)
///   9.  nft_fee_vault      (writable)
///  10.  cipher_mint        (read-only)
///  11.  token_program      (read-only)
///  -- remaining_accounts --
///  12+. BinArray PDAs      (writable, ordered by traversal direction)
///   N.  Oracle PDA         (writable, optional)
pub fn build_swap_account_metas(
    pool_key: &Pubkey,
    pool: &PoolState,
    user: &Pubkey,
    user_source: &Pubkey,
    user_destination: &Pubkey,
    quote_result: &QuoteResult,
    direction: SwapDirection,
    include_oracle: bool,
) -> Vec<AccountMeta> {
    let mut metas = Vec::with_capacity(20);

    // Fixed accounts (12)
    metas.push(AccountMeta::new(*pool_key, false));
    metas.push(AccountMeta::new_readonly(*user, true));
    metas.push(AccountMeta::new(*user_source, false));
    metas.push(AccountMeta::new(*user_destination, false));
    metas.push(AccountMeta::new(pool.base_vault, false));
    metas.push(AccountMeta::new(pool.quote_vault, false));
    metas.push(AccountMeta::new(pool.protocol_fee_vault, false));
    metas.push(AccountMeta::new(pool.creator_fee_vault, false));
    metas.push(AccountMeta::new(pool.holders_fee_vault, false));
    metas.push(AccountMeta::new(pool.nft_fee_vault, false));
    metas.push(AccountMeta::new_readonly(CIPHER_MINT, false));
    metas.push(AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false));

    // Remaining accounts: BinArrays (ordered by traversal direction)
    let mut bin_array_lbis = quote_result.bin_arrays_touched.clone();
    match direction {
        SwapDirection::BaseToQuote => bin_array_lbis.sort_unstable_by(|a, b| b.cmp(a)), // descending
        SwapDirection::QuoteToBase => bin_array_lbis.sort_unstable(), // ascending
    }

    for lbi in &bin_array_lbis {
        let (pda, _) = bin_array_pda(pool_key, *lbi);
        metas.push(AccountMeta::new(pda, false));
    }

    // Optional: Oracle
    if include_oracle {
        let (oracle, _) = oracle_pda(pool_key);
        metas.push(AccountMeta::new(oracle, false));
    }

    metas
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::QuoteResult;

    #[test]
    fn account_meta_count_no_oracle() {
        let pool_key = Pubkey::default();
        let mut pool: PoolState = bytemuck::Zeroable::zeroed();
        pool.base_vault = Pubkey::new_unique();
        pool.quote_vault = Pubkey::new_unique();
        pool.protocol_fee_vault = Pubkey::new_unique();
        pool.creator_fee_vault = Pubkey::new_unique();
        pool.holders_fee_vault = Pubkey::new_unique();
        pool.nft_fee_vault = Pubkey::new_unique();

        let quote = QuoteResult {
            in_amount: 100,
            out_amount: 99,
            fee_amount: 1,
            bin_arrays_touched: vec![0, 64],
        };

        let metas = build_swap_account_metas(
            &pool_key, &pool,
            &Pubkey::new_unique(), &Pubkey::new_unique(), &Pubkey::new_unique(),
            &quote, SwapDirection::BaseToQuote, false,
        );

        // 12 fixed + 2 bin arrays = 14
        assert_eq!(metas.len(), 14);
    }

    #[test]
    fn account_meta_count_with_oracle() {
        let pool_key = Pubkey::default();
        let pool: PoolState = bytemuck::Zeroable::zeroed();

        let quote = QuoteResult {
            in_amount: 100,
            out_amount: 99,
            fee_amount: 1,
            bin_arrays_touched: vec![0],
        };

        let metas = build_swap_account_metas(
            &pool_key, &pool,
            &Pubkey::new_unique(), &Pubkey::new_unique(), &Pubkey::new_unique(),
            &quote, SwapDirection::QuoteToBase, true,
        );

        // 12 fixed + 1 bin array + 1 oracle = 14
        assert_eq!(metas.len(), 14);
    }

    #[test]
    fn user_is_signer() {
        let pool_key = Pubkey::default();
        let pool: PoolState = bytemuck::Zeroable::zeroed();

        let quote = QuoteResult {
            in_amount: 0, out_amount: 0, fee_amount: 0,
            bin_arrays_touched: vec![],
        };

        let metas = build_swap_account_metas(
            &pool_key, &pool,
            &Pubkey::new_unique(), &Pubkey::new_unique(), &Pubkey::new_unique(),
            &quote, SwapDirection::BaseToQuote, false,
        );

        // Account at index 1 should be signer (user)
        assert!(metas[1].is_signer, "user account should be signer");
        // Pool should be writable
        assert!(metas[0].is_writable, "pool should be writable");
        // cipher_mint (index 10) should be read-only
        assert!(!metas[10].is_writable, "cipher_mint should be read-only");
        // token_program (index 11) should be read-only
        assert!(!metas[11].is_writable, "token_program should be read-only");
    }

    #[test]
    fn bin_arrays_sorted_by_direction() {
        let pool_key = Pubkey::default();
        let pool: PoolState = bytemuck::Zeroable::zeroed();

        let quote = QuoteResult {
            in_amount: 100, out_amount: 99, fee_amount: 1,
            bin_arrays_touched: vec![0, 64, -64],
        };

        // B2Q: descending order
        let metas_b2q = build_swap_account_metas(
            &pool_key, &pool,
            &Pubkey::new_unique(), &Pubkey::new_unique(), &Pubkey::new_unique(),
            &quote, SwapDirection::BaseToQuote, false,
        );

        // The bin array accounts start at index 12
        // For B2Q, lbi order should be 64, 0, -64 (descending)
        let (pda_64, _) = bin_array_pda(&pool_key, 64);
        let (pda_0, _) = bin_array_pda(&pool_key, 0);
        let (pda_n64, _) = bin_array_pda(&pool_key, -64);

        assert_eq!(metas_b2q[12].pubkey, pda_64);
        assert_eq!(metas_b2q[13].pubkey, pda_0);
        assert_eq!(metas_b2q[14].pubkey, pda_n64);

        // Q2B: ascending order
        let metas_q2b = build_swap_account_metas(
            &pool_key, &pool,
            &Pubkey::new_unique(), &Pubkey::new_unique(), &Pubkey::new_unique(),
            &quote, SwapDirection::QuoteToBase, false,
        );

        assert_eq!(metas_q2b[12].pubkey, pda_n64);
        assert_eq!(metas_q2b[13].pubkey, pda_0);
        assert_eq!(metas_q2b[14].pubkey, pda_64);
    }
}
