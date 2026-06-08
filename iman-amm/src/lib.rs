//! IMÁN AMM adapter for Jupiter — TRAMPA PROPINA concentrated-liquidity pools.
//!
//! TrampaPool on-chain layout (after 8-byte discriminator):
//!   [0..32]    token_a_mint
//!   [32..64]   token_b_mint
//!   [64..96]   token_a_vault
//!   [96..128]  token_b_vault
//!   [128..160] oracle_pubkey
//!   [160..192] authority
//!   [192..224] fee_vault
//!   [224..226] propina_pct          u16  (7500 = 75%)
//!   [226..228] concentrador_bps     u16  (20000 = ±200bps)
//!   [228..232] _pad0                [u8;4]
//!   [232..240] latido_interval_min  u64
//!   [240..248] latido_interval_max  u64
//!   [248..256] latido_window        u64
//!   [256..264] last_latido_slot     u64
//!   [264..266] incentivo_pct        u16
//!   [266..272] _pad1                [u8;6]
//!   [272..280] total_fees_collected u64
//!   [280..288] reserve_a            u64
//!   [288..296] reserve_b            u64
//!   [296..808] price_history        [u64;64]
//!   [808]      price_history_idx    u8
//!   [809]      is_active            u8
//!   [810]      bump                 u8
//!   [811..816] _pad2                [u8;5]
//! Total (body, no discriminator): 816 bytes

use anyhow::{anyhow, Result};
use bytemuck::{Pod, Zeroable};
use jupiter_amm_interface::{
    AccountMap, Amm, AmmContext, AmmLabel, AmmProgramIdToLabel, KeyedAccount, Quote, QuoteParams,
    SwapAndAccountMetas, SwapMode, SwapParams,
};
use rust_decimal::prelude::*;
use solana_sdk::{instruction::AccountMeta, pubkey, pubkey::Pubkey};

pub const TRAMPA_PROGRAM_ID: Pubkey =
    pubkey!("FpFXNWCm5qM4t9GKttp9Jkx8YpYfxgW5Cu37T8pdr8oE");

const TOKEN_PROGRAM: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const CLOCK_SYSVAR: Pubkey  = pubkey!("SysvarC1ock11111111111111111111111111111111");

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct TrampaPoolState {
    pub token_a_mint:          [u8; 32],
    pub token_b_mint:          [u8; 32],
    pub token_a_vault:         [u8; 32],
    pub token_b_vault:         [u8; 32],
    pub oracle_pubkey:         [u8; 32],
    pub authority:             [u8; 32],
    pub fee_vault:             [u8; 32],
    pub propina_pct:           u16,
    pub concentrador_bps:      u16,
    pub _pad0:                 [u8; 4],
    pub latido_interval_min:   u64,
    pub latido_interval_max:   u64,
    pub latido_window:         u64,
    pub last_latido_slot:      u64,
    pub incentivo_pct:         u16,
    pub _pad1:                 [u8; 6],
    pub total_fees_collected:  u64,
    pub reserve_a:             u64,
    pub reserve_b:             u64,
    pub price_history:         [u64; 64],
    pub price_history_idx:     u8,
    pub is_active:             u8,
    pub bump:                  u8,
    pub _pad2:                 [u8; 5],
}

const _: () = assert!(std::mem::size_of::<TrampaPoolState>() == 816);

fn pk(bytes: &[u8; 32]) -> Pubkey {
    Pubkey::from(*bytes)
}

#[derive(Clone)]
pub struct ImanAmm {
    pool_key:     Pubkey,
    state:        TrampaPoolState,
    reserve_a:    u64,
    reserve_b:    u64,
    oracle_price: u64,
    current_slot: u64,
}

impl ImanAmm {
    /// PROPINA fee bps: 75% × price divergence from oracle.
    fn propina_fee_bps(&self) -> u64 {
        if self.oracle_price == 0 || self.reserve_a == 0 {
            return 0;
        }
        // SOL(9dec) / USDC(6dec): scale reserve_b × 1e9 to get 6-decimal price
        let pool_price = self.reserve_b.saturating_mul(1_000_000_000) / self.reserve_a;
        let divergence = if self.oracle_price > pool_price {
            (self.oracle_price - pool_price) * 10_000 / self.oracle_price
        } else {
            (pool_price - self.oracle_price) * 10_000 / self.oracle_price
        };
        divergence * self.state.propina_pct as u64 / 10_000
    }

    /// True when inside LATIDO free-fee window.
    fn is_latido_window(&self) -> bool {
        let min = self.state.latido_interval_min;
        let max = self.state.latido_interval_max;
        if max <= min { return false; }
        let entropy = self.current_slot.wrapping_rem(max - min);
        let interval = min.saturating_add(entropy);
        let since = self.current_slot.saturating_sub(self.state.last_latido_slot);
        since > interval && since <= interval.saturating_add(self.state.latido_window)
    }

    fn compute_out(&self, in_amount: u64, a_to_b: bool) -> Result<(u64, u64)> {
        let (reserve_in, reserve_out) = if a_to_b {
            (self.reserve_a, self.reserve_b)
        } else {
            (self.reserve_b, self.reserve_a)
        };
        if reserve_in == 0 || reserve_out == 0 {
            return Err(anyhow!("no liquidity"));
        }
        let k = reserve_in as u128 * reserve_out as u128;
        let new_in = reserve_in as u128 + in_amount as u128;
        let gross_out = reserve_out.saturating_sub((k / new_in) as u64);
        let fee_bps = if self.is_latido_window() { 0 } else { self.propina_fee_bps() };
        let fee_amount = gross_out * fee_bps / 10_000;
        Ok((gross_out.saturating_sub(fee_amount), fee_amount))
    }
}

impl Amm for ImanAmm {
    fn from_keyed_account(keyed_account: &KeyedAccount, _ctx: &AmmContext) -> Result<Self> {
        let data = keyed_account.account.data();
        let body_size = std::mem::size_of::<TrampaPoolState>();
        if data.len() < 8 + body_size {
            return Err(anyhow!("account data too small"));
        }
        let state: TrampaPoolState =
            *bytemuck::try_from_bytes::<TrampaPoolState>(&data[8..8 + body_size])
                .map_err(|e| anyhow!("deserialize: {e:?}"))?;
        if state.is_active == 0 {
            return Err(anyhow!("pool inactive"));
        }
        Ok(ImanAmm { pool_key: keyed_account.key, state, reserve_a: 0, reserve_b: 0, oracle_price: 0, current_slot: 0 })
    }

    fn label(&self) -> String { "IMÁN".to_string() }
    fn program_id(&self) -> Pubkey { TRAMPA_PROGRAM_ID }
    fn key(&self) -> Pubkey { self.pool_key }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        vec![pk(&self.state.token_a_mint), pk(&self.state.token_b_mint)]
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        vec![
            self.pool_key,
            pk(&self.state.token_a_vault),
            pk(&self.state.token_b_vault),
            pk(&self.state.oracle_pubkey),
        ]
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        let body_size = std::mem::size_of::<TrampaPoolState>();
        if let Some(acct) = account_map.get(&self.pool_key) {
            let d = acct.data();
            if d.len() >= 8 + body_size {
                self.state = *bytemuck::try_from_bytes::<TrampaPoolState>(&d[8..8 + body_size])
                    .map_err(|e| anyhow!("{e:?}"))?;
            }
        }
        if let Some(acct) = account_map.get(&pk(&self.state.token_a_vault)) {
            self.reserve_a = read_token_amount(acct.data());
        }
        if let Some(acct) = account_map.get(&pk(&self.state.token_b_vault)) {
            self.reserve_b = read_token_amount(acct.data());
        }
        // Pyth V1 PriceAccount: exponent i32 @ offset 20, price i64 @ offset 208
        if let Some(acct) = account_map.get(&pk(&self.state.oracle_pubkey)) {
            let d = acct.data();
            if d.len() >= 228 {
                let price_raw = i64::from_le_bytes(d[208..216].try_into().unwrap_or([0; 8]));
                let exponent  = i32::from_le_bytes(d[20..24].try_into().unwrap_or([0; 4]));
                if price_raw > 0 {
                    let adj = 6i32 + exponent;
                    self.oracle_price = if adj >= 0 {
                        price_raw as u64 * 10u64.pow(adj as u32)
                    } else {
                        price_raw as u64 / 10u64.pow((-adj) as u32)
                    };
                }
            }
        }
        Ok(())
    }

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote> {
        if self.state.is_active == 0 { return Err(anyhow!("pool inactive")); }
        if quote_params.swap_mode != SwapMode::ExactIn { return Err(anyhow!("ExactIn only")); }
        let a_to_b = quote_params.input_mint == pk(&self.state.token_a_mint);
        let (out_amount, fee_amount) = self.compute_out(quote_params.amount, a_to_b)?;
        let fee_pct = if out_amount + fee_amount > 0 {
            Decimal::from(fee_amount) / Decimal::from(out_amount + fee_amount)
        } else {
            Decimal::ZERO
        };
        Ok(Quote {
            in_amount: quote_params.amount,
            out_amount,
            fee_amount,
            fee_mint: if a_to_b { pk(&self.state.token_b_mint) } else { pk(&self.state.token_a_mint) },
            fee_pct,
        })
    }

    fn get_swap_and_account_metas(&self, swap_params: &SwapParams) -> Result<SwapAndAccountMetas> {
        use jupiter_amm_interface::Swap;
        let a_to_b = swap_params.source_mint == pk(&self.state.token_a_mint);
        let (user_in, user_out) = if a_to_b {
            (swap_params.source_token_account, swap_params.destination_token_account)
        } else {
            (swap_params.destination_token_account, swap_params.source_token_account)
        };
        // TRAMPA swap account order:
        //   [0] user (signer)  [1] pool (writable)  [2] user_in (writable)  [3] user_out (writable)
        //   [4] vault_a (writable)  [5] vault_b (writable)  [6] oracle (readonly)
        //   [7] fee_vault (writable)  [8] token_program  [9] clock_sysvar
        Ok(SwapAndAccountMetas {
            swap: Swap::ImanTrampa { a_to_b },
            account_metas: vec![
                AccountMeta::new_readonly(swap_params.token_transfer_authority, true),
                AccountMeta::new(self.pool_key, false),
                AccountMeta::new(user_in, false),
                AccountMeta::new(user_out, false),
                AccountMeta::new(pk(&self.state.token_a_vault), false),
                AccountMeta::new(pk(&self.state.token_b_vault), false),
                AccountMeta::new_readonly(pk(&self.state.oracle_pubkey), false),
                AccountMeta::new(pk(&self.state.fee_vault), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM, false),
                AccountMeta::new_readonly(CLOCK_SYSVAR, false),
            ],
        })
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> { Box::new(self.clone()) }
    fn get_accounts_len(&self) -> usize { 10 }
    fn is_active(&self) -> bool { self.state.is_active != 0 }
    fn supports_exact_out(&self) -> bool { false }
}

impl AmmProgramIdToLabel for ImanAmm {
    const PROGRAM_ID_TO_LABELS: &[(Pubkey, AmmLabel)] = &[(TRAMPA_PROGRAM_ID, "IMÁN")];
}

fn read_token_amount(data: &[u8]) -> u64 {
    if data.len() >= 72 {
        u64::from_le_bytes(data[64..72].try_into().unwrap_or([0u8; 8]))
    } else {
        0
    }
}
