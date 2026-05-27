//! Cube DEX (cubic-pool) — Jupiter `Amm` trait implementation.
//!
//! Cubic-pool is a Balancer-V2-style weighted constant-product AMM with
//! virtual / actual balances, multi-token pools (2–10 tokens), and
//! Token-2022 support.
//!
//! On-chain source: <https://github.com/cubee-ee/contracts>
//! Program id (mainnet/devnet/localnet):
//!   `8iQtGj9mcUfFUGaiCpPy89swC3s8YTC8FhVZWfgeZhwu`
//!
//! Quote pipeline replicates `cubic-pool/src/instructions/swap.rs`:
//!   1. guard: pool_enabled && swaps_enabled && amount > 0
//!   2. max_selloff sliding-window check on GROSS amount_in
//!   3. fee subtraction with ZeroFeeAmount guard
//!   4. lp_actual_out = actual_balance - protocol_fees_owed (cap)
//!   5. raw virtual_balance into the weighted-CPMM curve
//!
//! When the LP-actual cap or selloff window blocks the full input the
//! quote falls back to a binary-search input cap so `quote.in_amount <=
//! quote_params.amount` (Jupiter convention).

pub mod constants;
pub mod ix;
pub mod math;
pub mod state;

use anyhow::{anyhow, Context, Result};
use jupiter_amm_interface::{
    AccountMap, Amm, AmmContext, ClockRef, KeyedAccount, Quote, QuoteParams, SingleProgramAmm,
    Swap, SwapAndAccountMetas, SwapParams,
};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashSet;
use std::sync::atomic::Ordering;

use self::constants::{CUBE_LABEL, CUBIC_POOL_PROGRAM_ID};
use spl_associated_token_account::get_associated_token_address_with_program_id;

fn derive_vault(pool: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(pool, mint, token_program)
}
use self::math::cubic_math::calc_out_given_in;
use self::math::fee::apply_swap_fee;
use self::math::max_selloff::{check as max_selloff_check, SelloffInputs};
use self::state::{PoolState, TokenSlot};

#[derive(Clone)]
pub struct CubeAmm {
    key: Pubkey,
    program_id: Pubkey,
    state: PoolState,
    reserve_mints: Vec<Pubkey>,
    clock_ref: ClockRef,
}

impl SingleProgramAmm for CubeAmm {
    const PROGRAM_ID: Pubkey = CUBIC_POOL_PROGRAM_ID;
    const LABEL: &'static str = CUBE_LABEL;
}

impl CubeAmm {
    fn try_quote(
        &self,
        in_slot: &TokenSlot,
        out_slot: &TokenSlot,
        lp_actual_out: u64,
        amount_in: u64,
    ) -> Result<u64> {
        // On-chain swap.rs runs max_selloff::check_and_advance BEFORE the
        // fee subtraction, using the gross amount_in. Replicate that
        // order so the sliding-window cap sees the same value.
        max_selloff_check(
            &SelloffInputs {
                max_selloff: in_slot.max_selloff,
                period_length: in_slot.max_selloff_period_length,
                previous_selloff: in_slot.previous_selloff,
                current_selloff: in_slot.current_selloff,
                window_start_timestamp: in_slot.window_start_timestamp,
            },
            amount_in,
            self.clock_ref.unix_timestamp.load(Ordering::Relaxed),
        )?;
        let (_fee, after) = apply_swap_fee(amount_in, self.state.swap_fee_rate)?;
        if after == 0 {
            return Err(anyhow!("amount_in_after_fee == 0"));
        }
        calc_out_given_in(
            in_slot.virtual_balance,
            in_slot.normalized_weight,
            out_slot.virtual_balance,
            out_slot.normalized_weight,
            after,
            lp_actual_out,
        )
    }
}

impl Amm for CubeAmm {
    fn from_keyed_account(keyed_account: &KeyedAccount, ctx: &AmmContext) -> Result<Self> {
        let state = PoolState::decode(&keyed_account.account.data).context("decode CubicPool")?;
        let reserve_mints = state
            .active_tokens()
            .iter()
            .map(|t| t.mint)
            .collect::<Vec<_>>();
        Ok(Self {
            key: keyed_account.key,
            program_id: keyed_account.account.owner,
            state,
            reserve_mints,
            clock_ref: ctx.clock_ref.clone(),
        })
    }

    fn label(&self) -> String {
        CUBE_LABEL.to_string()
    }

    fn program_id(&self) -> Pubkey {
        self.program_id
    }

    fn key(&self) -> Pubkey {
        self.key
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        self.reserve_mints.clone()
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        // The pool account carries virtual + actual balances + fees +
        // selloff accumulators + enable flags. Vaults are kept in sync
        // by the on-chain program; no need to re-fetch them.
        vec![self.key]
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        let acc = account_map
            .get(&self.key)
            .ok_or_else(|| anyhow!("CubeAmm: pool account missing from update map"))?;
        self.state = PoolState::decode(&acc.data)?;
        self.reserve_mints = self
            .state
            .active_tokens()
            .iter()
            .map(|t| t.mint)
            .collect();
        Ok(())
    }

    fn quote(&self, qp: &QuoteParams) -> Result<Quote> {
        if !self.state.pool_enabled || !self.state.swaps_enabled {
            return Err(anyhow!("CubeAmm: pool/swaps disabled"));
        }
        if qp.amount == 0 {
            return Err(anyhow!("CubeAmm: zero amount"));
        }
        let in_idx = self
            .state
            .index_of_mint(&qp.input_mint)
            .ok_or_else(|| anyhow!("CubeAmm: input mint not in pool"))?;
        let out_idx = self
            .state
            .index_of_mint(&qp.output_mint)
            .ok_or_else(|| anyhow!("CubeAmm: output mint not in pool"))?;
        if in_idx == out_idx {
            return Err(anyhow!("CubeAmm: input == output mint"));
        }

        let in_slot = &self.state.tokens[in_idx];
        let out_slot = &self.state.tokens[out_idx];
        let lp_actual_out = out_slot
            .actual_balance
            .saturating_sub(out_slot.protocol_fees_owed);

        // Fast path: full input fits.
        if let Ok(out_amount) = self.try_quote(in_slot, out_slot, lp_actual_out, qp.amount) {
            let (fee_amount, _) = apply_swap_fee(qp.amount, self.state.swap_fee_rate)
                .unwrap_or((0, qp.amount));
            return Ok(Quote {
                in_amount: qp.amount,
                out_amount,
                fee_amount,
                fee_mint: in_slot.mint,
                fee_pct: rust_decimal::Decimal::new(self.state.swap_fee_rate as i64, 6),
            });
        }

        // Slow path: binary-search the largest viable in_amount so
        // `quote.in_amount <= quote_params.amount`.
        let mut lo: u64 = 0;
        let mut hi: u64 = qp.amount;
        let mut best: Option<(u64, u64)> = None;
        for _ in 0..64 {
            if hi <= lo + 1 {
                break;
            }
            let mid = lo + (hi - lo) / 2;
            match self.try_quote(in_slot, out_slot, lp_actual_out, mid) {
                Ok(out) => {
                    best = Some((mid, out));
                    lo = mid;
                }
                Err(_) => {
                    hi = mid;
                }
            }
        }
        let (in_amount, out_amount) =
            best.ok_or_else(|| anyhow!("CubeAmm: no viable quote at any sub-amount"))?;
        let (fee_amount, _) =
            apply_swap_fee(in_amount, self.state.swap_fee_rate).unwrap_or((0, in_amount));
        Ok(Quote {
            in_amount,
            out_amount,
            fee_amount,
            fee_mint: in_slot.mint,
            fee_pct: rust_decimal::Decimal::new(self.state.swap_fee_rate as i64, 6),
        })
    }

    fn get_swap_and_account_metas(&self, sp: &SwapParams) -> Result<SwapAndAccountMetas> {
        use solana_sdk::instruction::AccountMeta;

        let in_idx = self
            .state
            .index_of_mint(&sp.source_mint)
            .ok_or_else(|| anyhow!("CubeAmm: source mint not in pool"))?;
        let out_idx = self
            .state
            .index_of_mint(&sp.destination_mint)
            .ok_or_else(|| anyhow!("CubeAmm: destination mint not in pool"))?;
        let in_slot = &self.state.tokens[in_idx];
        let out_slot = &self.state.tokens[out_idx];

        let vault_in = derive_vault(&self.key, &in_slot.mint, &in_slot.token_program);
        let vault_out = derive_vault(&self.key, &out_slot.mint, &out_slot.token_program);

        // Jupiter convention: leading program-id meta (non-signer / non-
        // writable), all other accounts have is_signer=false in the
        // inner ix — the aggregator's CPI machinery propagates the
        // user's signer flag from the outer tx. Matches the layout used
        // by spl_token_swap_amm.rs::to_dex_account_metas.
        let account_metas = vec![
            AccountMeta::new_readonly(self.program_id, false),
            AccountMeta::new(self.key, false),
            AccountMeta::new_readonly(in_slot.mint, false),
            AccountMeta::new_readonly(out_slot.mint, false),
            AccountMeta::new(sp.source_token_account, false),
            AccountMeta::new(sp.destination_token_account, false),
            AccountMeta::new(vault_in, false),
            AccountMeta::new(vault_out, false),
            AccountMeta::new(sp.token_transfer_authority, false),
            AccountMeta::new_readonly(in_slot.token_program, false),
            AccountMeta::new_readonly(out_slot.token_program, false),
        ];
        // TODO upstream: companion PR adding a `Cube` variant to
        // jup-ag/jupiter-amm-interface. Until it lands, `TokenSwap` is
        // the closest generic placeholder.
        Ok(SwapAndAccountMetas {
            swap: Swap::TokenSwap,
            account_metas,
        })
    }

    fn get_accounts_len(&self) -> usize {
        // 10 swap accounts (pool, mint_in, mint_out, user_in, user_out,
        // vault_in, vault_out, user, tp_in, tp_out) + 1 leading
        // program-id meta.
        11
    }

    fn is_active(&self) -> bool {
        if !self.state.pool_enabled || !self.state.swaps_enabled {
            return false;
        }
        self.state
            .active_tokens()
            .iter()
            .any(|t| t.actual_balance.saturating_sub(t.protocol_fees_owed) > 0)
    }

    fn underlying_liquidities(&self) -> Option<HashSet<Pubkey>> {
        None
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}
