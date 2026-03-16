//! Jupiter AMM adapter for CipherDLMM (Orbit Finance).
//!
//! Implements the `Amm` trait from `jupiter-amm-interface` (v0.6.0)
//! to enable Jupiter routing through CipherDLMM pools.
//!
//! Program ID: Fn3fA3fjsmpULNL7E9U79jKTe1KHxPtQeWdURCbJXCnM

pub mod accounts;
pub mod math;
pub mod pda;
pub mod state;

use std::collections::HashSet;

use anyhow::Result;
use jupiter_amm_interface::{
    AccountMap, Amm, AmmContext, AmmLabel, AmmProgramIdToLabel, KeyedAccount, Quote, QuoteParams, Swap, SwapAndAccountMetas,
    SwapMode, SwapParams,
};
use rust_decimal::Decimal;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

use crate::accounts::build_swap_account_metas;
use crate::math::{quote_exact_in, quote_exact_out, SwapDirection};
use crate::pda::{bin_array_pda, oracle_pda};
use crate::state::{deserialize_bin_array, deserialize_pool, BinArrayState, PoolState};

/// CipherDLMM on-chain program ID.
pub const PROGRAM_ID: Pubkey = pubkey!("Fn3fA3fjsmpULNL7E9U79jKTe1KHxPtQeWdURCbJXCnM");

/// Number of BinArrays to fetch on each side of the active bin.
const BIN_ARRAY_RANGE: i32 = 5;

/// The Jupiter AMM adapter for a single CipherDLMM pool.
#[derive(Clone)]
pub struct CipherDlmmAmm {
    pool_key: Pubkey,
    pool: PoolState,
    bin_arrays: ahash::HashMap<i32, BinArrayState>,
    oracle_key: Pubkey,
}

impl AmmProgramIdToLabel for CipherDlmmAmm {
    const PROGRAM_ID_TO_LABELS: &[(Pubkey, AmmLabel)] = &[
        (PROGRAM_ID, "CipherDLMM"),
    ];
}

impl Amm for CipherDlmmAmm {
    // ----- Construction -----

    fn from_keyed_account(keyed_account: &KeyedAccount, _amm_context: &AmmContext) -> Result<Self>
    where
        Self: Sized,
    {
        let pool = deserialize_pool(&keyed_account.account.data)?;
        let (oracle, _) = oracle_pda(&keyed_account.key);
        Ok(Self {
            pool_key: keyed_account.key,
            pool,
            bin_arrays: ahash::HashMap::default(),
            oracle_key: oracle,
        })
    }

    // ----- Identity -----

    fn label(&self) -> String {
        "CipherDLMM".to_string()
    }

    fn program_id(&self) -> Pubkey {
        PROGRAM_ID
    }

    fn key(&self) -> Pubkey {
        self.pool_key
    }

    // ----- Mints -----

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        vec![self.pool.base_mint, self.pool.quote_mint]
    }

    // ----- Account fetching -----

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        let mut keys = Vec::with_capacity(2 + (BIN_ARRAY_RANGE as usize * 2 + 1));

        // Pool itself
        keys.push(self.pool_key);

        // BinArrays around the active bin
        let center = BinArrayState::lower_bin_index_from(self.pool.active_bin);
        for i in -BIN_ARRAY_RANGE..=BIN_ARRAY_RANGE {
            let lbi = center + i * 64;
            let (pda, _) = bin_array_pda(&self.pool_key, lbi);
            keys.push(pda);
        }

        // Oracle
        keys.push(self.oracle_key);

        keys
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        // Re-deserialize pool if available
        if let Some(pool_acc) = account_map.get(&self.pool_key) {
            self.pool = deserialize_pool(&pool_acc.data)?;
        }

        // Rebuild bin_arrays from cached accounts
        self.bin_arrays.clear();
        let center = BinArrayState::lower_bin_index_from(self.pool.active_bin);
        for i in -BIN_ARRAY_RANGE..=BIN_ARRAY_RANGE {
            let lbi = center + i * 64;
            let (pda, _) = bin_array_pda(&self.pool_key, lbi);
            if let Some(acc) = account_map.get(&pda) {
                // Gracefully skip accounts that don't exist or can't be deserialized
                if let Ok(ba) = deserialize_bin_array(&acc.data) {
                    self.bin_arrays.insert(ba.lower_bin_index, ba);
                }
            }
        }

        Ok(())
    }

    // ----- Quoting -----

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote> {
        let direction = self.resolve_direction(&quote_params.input_mint)?;
        let fee_bps = math::effective_fee_bps(&self.pool)?;

        let result = match quote_params.swap_mode {
            SwapMode::ExactIn => {
                quote_exact_in(&self.pool, &self.bin_arrays, quote_params.amount, direction)?
            }
            SwapMode::ExactOut => {
                quote_exact_out(&self.pool, &self.bin_arrays, quote_params.amount, direction)?
            }
        };

        // Fee mint is always quote token (fees are in quote domain)
        let fee_mint = self.pool.quote_mint;
        let fee_pct = Decimal::from(fee_bps) / Decimal::from(10_000u32);

        Ok(Quote {
            in_amount: result.in_amount,
            out_amount: result.out_amount,
            fee_amount: result.fee_amount,
            fee_mint,
            fee_pct,
            ..Quote::default()
        })
    }

    // ----- Swap instruction building -----

    fn get_swap_and_account_metas(&self, swap_params: &SwapParams) -> Result<SwapAndAccountMetas> {
        let direction = self.resolve_direction(&swap_params.source_mint)?;

        // Run quote to determine which BinArrays are touched
        let amount = match swap_params.swap_mode {
            SwapMode::ExactIn => swap_params.in_amount,
            SwapMode::ExactOut => swap_params.out_amount,
        };
        let quote_result = match swap_params.swap_mode {
            SwapMode::ExactIn => {
                quote_exact_in(&self.pool, &self.bin_arrays, amount, direction)?
            }
            SwapMode::ExactOut => {
                quote_exact_out(&self.pool, &self.bin_arrays, amount, direction)?
            }
        };

        let account_metas = build_swap_account_metas(
            &self.pool_key,
            &self.pool,
            &swap_params.token_transfer_authority,
            &swap_params.source_token_account,
            &swap_params.destination_token_account,
            &quote_result,
            direction,
            true, // include oracle
        );

        Ok(SwapAndAccountMetas {
            // Use MeteoraDlmm as placeholder — Jupiter team will add CipherDlmm variant
            swap: Swap::MeteoraDlmm,
            account_metas,
        })
    }

    // ----- Cloning -----

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }

    // ----- Optional overrides -----

    fn has_dynamic_accounts(&self) -> bool {
        true
    }

    fn supports_exact_out(&self) -> bool {
        true
    }

    fn get_accounts_len(&self) -> usize {
        // 12 fixed + up to 11 BinArrays + 1 oracle = 24
        24
    }

    fn is_active(&self) -> bool {
        !self.pool.is_swap_paused()
    }

    fn requires_update_for_reserve_mints(&self) -> bool {
        false
    }

    fn unidirectional(&self) -> bool {
        false
    }

    fn program_dependencies(&self) -> Vec<(Pubkey, String)> {
        vec![]
    }

    fn underlying_liquidities(&self) -> Option<HashSet<Pubkey>> {
        None
    }
}

impl CipherDlmmAmm {
    /// Determine swap direction from the input mint.
    fn resolve_direction(&self, input_mint: &Pubkey) -> Result<SwapDirection> {
        if *input_mint == self.pool.base_mint {
            Ok(SwapDirection::BaseToQuote)
        } else if *input_mint == self.pool.quote_mint {
            Ok(SwapDirection::QuoteToBase)
        } else {
            anyhow::bail!(
                "Input mint {} does not match pool base ({}) or quote ({})",
                input_mint,
                self.pool.base_mint,
                self.pool.quote_mint
            )
        }
    }
}
