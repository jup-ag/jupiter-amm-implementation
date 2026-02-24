//! Jupiter AMM adapter for MnM DLMM (Dynamic Liquidity Market Maker).
//!
//! MnM DLMM is a concentrated liquidity AMM with bin shifting — bins
//! automatically reposition with trades to keep liquidity around the active price.
//!
//! Program ID: MnMRzPXhhuFFzfXvffkMGeodBqY7hnaqNpQcYGeREi5

mod pda;
mod quote;
mod state;

use anyhow::{Result, anyhow};
use borsh::BorshSerialize;
use jupiter_amm_interface::{
    Amm, AccountMap, AmmContext, AmmLabel, KeyedAccount, Quote, QuoteParams,
    SingleProgramAmm, Swap, SwapAndAccountMetas, SwapMode, SwapParams,
    try_get_account_data,
};
use rust_decimal::Decimal;
use solana_sdk::{
    instruction::AccountMeta,
    pubkey::Pubkey,
    pubkey,
};

use pda::{find_tick_map, find_bin_array, find_vault_x, find_vault_y};
use quote::{QuoteInput, SwapDirection, simulate_swap_multi};
use state::{BinArray, PoolState, BINS_PER_ARRAY};

// ─── Constants ──────────────────────────────────────────────────────────────

/// MnM DLMM program ID (mainnet).
pub const MNM_PROGRAM_ID: Pubkey = pubkey!("MnMRzPXhhuFFzfXvffkMGeodBqY7hnaqNpQcYGeREi5");

/// SPL Token program ID.
const SPL_TOKEN_PROGRAM: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

/// SPL Token-2022 program ID.
const SPL_TOKEN_2022_PROGRAM: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

/// Adjacent BinArrays to fetch for quoting (active ± 1).
const BIN_ARRAYS_RADIUS: i32 = 1;

// ─── Swap instruction args (for reference/future use) ───────────────────────

/// Anchor sighash for `global:swap`.
/// SHA-256("global:swap")[0..8] = [248, 198, 158, 145, 225, 117, 135, 200]
#[allow(dead_code)]
const SWAP_DISCRIMINATOR: [u8; 8] = [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8];

#[derive(Clone, Debug, BorshSerialize)]
#[allow(dead_code)]
struct SwapArgs {
    amount_in: u64,
    min_amount_out: u64,
    swap_x_to_y: bool,
}

// ─── MnM DLMM AMM ──────────────────────────────────────────────────────────

/// MnM DLMM pool adapter for Jupiter's routing engine.
#[derive(Clone)]
pub struct MnmDlmmAmm {
    key: Pubkey,
    pool: PoolState,
    mint_x: Pubkey,
    mint_y: Pubkey,
    vault_x: Pubkey,
    vault_y: Pubkey,
    tick_map: Pubkey,
    token_program_x: Pubkey,
    token_program_y: Pubkey,
    bin_array_keys: Vec<Pubkey>,
    bin_arrays: Vec<BinArray>,
    is_paused: bool,
}

impl SingleProgramAmm for MnmDlmmAmm {
    const PROGRAM_ID: Pubkey = MNM_PROGRAM_ID;
    const LABEL: AmmLabel = "MnM DLMM";
}

impl MnmDlmmAmm {
    fn bytes_to_pubkey(bytes: &[u8; 32]) -> Pubkey {
        Pubkey::new_from_array(*bytes)
    }

    fn compute_bin_array_keys(pool_key: &Pubkey, active_bin_id: i32) -> Vec<Pubkey> {
        let active_array_idx = active_bin_id.div_euclid(BINS_PER_ARRAY as i32);
        let mut keys = Vec::with_capacity((2 * BIN_ARRAYS_RADIUS + 1) as usize);
        for offset in -BIN_ARRAYS_RADIUS..=BIN_ARRAYS_RADIUS {
            let (pda, _) = find_bin_array(pool_key, active_array_idx + offset);
            keys.push(pda);
        }
        keys
    }

    fn resolve_token_program(bytes: &[u8; 32]) -> Pubkey {
        let pk = Self::bytes_to_pubkey(bytes);
        if pk == SPL_TOKEN_2022_PROGRAM {
            SPL_TOKEN_2022_PROGRAM
        } else {
            SPL_TOKEN_PROGRAM
        }
    }
}

impl Amm for MnmDlmmAmm {
    fn from_keyed_account(keyed_account: &KeyedAccount, _amm_context: &AmmContext) -> Result<Self> {
        let data = &keyed_account.account.data;

        let pool: PoolState = state::deserialize(data)
            .ok_or_else(|| anyhow!("Failed to deserialize MnM PoolState (data len: {})", data.len()))?;

        let mint_x = Self::bytes_to_pubkey(&pool.token_mint_x);
        let mint_y = Self::bytes_to_pubkey(&pool.token_mint_y);
        let (vault_x, _) = find_vault_x(&keyed_account.key);
        let (vault_y, _) = find_vault_y(&keyed_account.key);
        let (tick_map, _) = find_tick_map(&keyed_account.key);
        let token_program_x = Self::resolve_token_program(&pool.token_program_x);
        let token_program_y = Self::resolve_token_program(&pool.token_program_y);
        let bin_array_keys = Self::compute_bin_array_keys(&keyed_account.key, pool.active_bin_id);
        let is_paused = pool.is_paused != 0;

        Ok(Self {
            key: keyed_account.key,
            pool,
            mint_x,
            mint_y,
            vault_x,
            vault_y,
            tick_map,
            token_program_x,
            token_program_y,
            bin_array_keys,
            bin_arrays: Vec::new(),
            is_paused,
        })
    }

    fn label(&self) -> String {
        "MnM DLMM".to_string()
    }

    fn program_id(&self) -> Pubkey {
        MNM_PROGRAM_ID
    }

    fn key(&self) -> Pubkey {
        self.key
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        vec![self.mint_x, self.mint_y]
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        let mut accounts = Vec::with_capacity(1 + self.bin_array_keys.len());
        accounts.push(self.key);
        accounts.extend_from_slice(&self.bin_array_keys);
        accounts
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        // Refresh pool state (active bin may have shifted)
        let pool_data = try_get_account_data(account_map, &self.key)?;
        if let Some(updated_pool) = state::deserialize::<PoolState>(pool_data) {
            if updated_pool.active_bin_id != self.pool.active_bin_id {
                self.bin_array_keys = Self::compute_bin_array_keys(&self.key, updated_pool.active_bin_id);
            }
            self.pool = updated_pool;
            self.is_paused = updated_pool.is_paused != 0;
        }

        // Load BinArrays for quoting
        self.bin_arrays.clear();
        for ba_key in &self.bin_array_keys {
            if let Ok(ba_data) = try_get_account_data(account_map, ba_key) {
                if let Some(ba) = state::deserialize::<BinArray>(ba_data) {
                    self.bin_arrays.push(ba);
                }
            }
            // Missing BinArrays are OK — pool may not have liquidity in those ranges
        }

        Ok(())
    }

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote> {
        if self.is_paused {
            return Err(anyhow!("MnM pool is paused"));
        }
        if quote_params.swap_mode == SwapMode::ExactOut {
            return Err(anyhow!("MnM DLMM does not support ExactOut mode"));
        }
        if self.bin_arrays.is_empty() {
            return Err(anyhow!("No BinArrays loaded — call update() first"));
        }

        let direction = if quote_params.input_mint == self.mint_x {
            SwapDirection::XtoY
        } else if quote_params.input_mint == self.mint_y {
            SwapDirection::YtoX
        } else {
            return Err(anyhow!(
                "Input mint {} doesn't match pool mints ({} or {})",
                quote_params.input_mint, self.mint_x, self.mint_y
            ));
        };

        let input = QuoteInput {
            amount_in: quote_params.amount,
            direction,
        };

        let ba_refs: Vec<&BinArray> = self.bin_arrays.iter().collect();
        let result = simulate_swap_multi(&self.pool, &ba_refs, &input)
            .ok_or_else(|| anyhow!("Swap simulation returned None"))?;

        if result.out_amount == 0 {
            return Err(anyhow!("No liquidity available for this swap"));
        }

        Ok(Quote {
            in_amount: result.in_amount,
            out_amount: result.out_amount,
            fee_amount: result.fee_amount,
            fee_mint: quote_params.input_mint,
            fee_pct: Decimal::from_f64_retain(result.fee_pct).unwrap_or(Decimal::ZERO),
        })
    }

    fn get_swap_and_account_metas(&self, swap_params: &SwapParams) -> Result<SwapAndAccountMetas> {
        let swap_x_to_y = swap_params.source_mint == self.mint_x;

        let active_array_idx = self.pool.active_bin_id.div_euclid(BINS_PER_ARRAY as i32);
        let (active_bin_array_pda, _) = find_bin_array(&self.key, active_array_idx);

        let (token_program_in, token_program_out) = if swap_x_to_y {
            (self.token_program_x, self.token_program_y)
        } else {
            (self.token_program_y, self.token_program_x)
        };

        //   0. pool          (mut)
        //   1. tick_map       (mut)
        //   2. bin_array      (mut)
        //   3. token_mint_x   (read)
        //   4. token_mint_y   (read)
        //   5. token_vault_x  (mut)
        //   6. token_vault_y  (mut)
        //   7. user_token_in  (mut)     ← source_token_account
        //   8. user_token_out (mut)     ← destination_token_account
        //   9. user           (signer)  ← token_transfer_authority
        //  10. token_program_in  (read)
        //  11. token_program_out (read)
        let account_metas = vec![
            AccountMeta::new(self.key, false),
            AccountMeta::new(self.tick_map, false),
            AccountMeta::new(active_bin_array_pda, false),
            AccountMeta::new_readonly(self.mint_x, false),
            AccountMeta::new_readonly(self.mint_y, false),
            AccountMeta::new(self.vault_x, false),
            AccountMeta::new(self.vault_y, false),
            AccountMeta::new(swap_params.source_token_account, false),
            AccountMeta::new(swap_params.destination_token_account, false),
            AccountMeta::new_readonly(swap_params.token_transfer_authority, true),
            AccountMeta::new_readonly(token_program_in, false),
            AccountMeta::new_readonly(token_program_out, false),
        ];

        // TODO: Requires new Swap::MnmDlmm variant in jupiter-amm-interface.
        // Using MeteoraDlmm as structural placeholder — account layout is compatible.
        Ok(SwapAndAccountMetas {
            swap: Swap::MeteoraDlmm,
            account_metas,
        })
    }

    fn has_dynamic_accounts(&self) -> bool {
        true // BinArray PDAs change as active bin moves
    }

    fn supports_exact_out(&self) -> bool {
        false
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }

    fn get_accounts_len(&self) -> usize {
        12
    }

    fn is_active(&self) -> bool {
        !self.is_paused
    }
}
