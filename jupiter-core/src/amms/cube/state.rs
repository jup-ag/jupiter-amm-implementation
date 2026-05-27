//! Manual little-endian decoder for the `CubicPool` v4 account.
//!
//! Layout reference: contracts-research §"CubicPool (v4 layout — current source)".
//! Offsets are cumulative from start of `data` (i.e. discriminator is at 0..8).
//!
//! We do NOT depend on cubic-pool's crate because:
//!   1. solana-program 3.0.0 (cubic-pool) vs solana-sdk 2.3.* (dflow-amm-
//!      interface) are incompatible.
//!   2. Pulling anchor-lang into a connector crate is too heavy.
//!
//! v3 (1154-byte) pools are not supported here; callers receive an error.
//! The migrate_pool_v4 instruction one-shot upgrades them; un-migrated pools
//! are a vanishing set in practice.

use super::constants::{MAX_TOKENS, POOL_DISCRIMINATOR, POOL_V3_LEN, POOL_V4_LEN};
use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Copy)]
pub struct TokenSlot {
    pub mint: Pubkey,
    pub token_program: Pubkey,
    pub normalized_weight: u64,
    /// Sliding-window cap on `amount_in` for THIS token. `0` disables.
    pub max_selloff: u64,
    /// Sliding-window length in seconds. Must be `> 0` whenever
    /// `max_selloff > 0`.
    pub max_selloff_period_length: u32,
    pub virtual_balance: u64,
    pub actual_balance: u64,
    pub protocol_fees_owed: u64,
    /// `amount_in` summed over the PREVIOUS sliding-window bucket.
    pub previous_selloff: u64,
    /// `amount_in` summed over the CURRENT bucket.
    pub current_selloff: u64,
    /// Start of the current bucket; `Clock::unix_timestamp` at the last
    /// window rotation.
    pub window_start_timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct PoolState {
    pub config: Pubkey,
    pub bump: u8,
    pub token_count: u8,
    pub pool_id: u64,
    pub swap_fee_rate: u32,
    pub protocol_fee_rate: u16,
    pub pool_enabled: bool,
    pub swaps_enabled: bool,
    pub tokens: [TokenSlot; MAX_TOKENS],
}

impl PoolState {
    /// Decode a `CubicPool` account `data` buffer.
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() == POOL_V3_LEN {
            return Err(anyhow!(
                "cube-amm: pool is v3 (1154 bytes); migrate via migrate_pool_v4 first"
            ));
        }
        if data.len() != POOL_V4_LEN {
            return Err(anyhow!(
                "cube-amm: unexpected pool data length {} (want {})",
                data.len(),
                POOL_V4_LEN
            ));
        }
        if &data[0..8] != POOL_DISCRIMINATOR {
            return Err(anyhow!("cube-amm: pool discriminator mismatch"));
        }

        let config = read_pubkey(data, 8);
        let bump = data[40];
        let token_count = data[41];
        let pool_id = read_u64(data, 42);
        let swap_fee_rate = read_u32(data, 50);
        let protocol_fee_rate = read_u16(data, 54);
        // skip created_at (8B @ 56)
        let pool_enabled = data[64] != 0;
        let swaps_enabled = data[65] != 0;
        // skip pool_admin/pending_pool_admin/range_manager block: 66..179

        let mut tokens: [TokenSlot; MAX_TOKENS] = [TokenSlot {
            mint: Pubkey::default(),
            token_program: Pubkey::default(),
            normalized_weight: 0,
            max_selloff: 0,
            max_selloff_period_length: 0,
            virtual_balance: 0,
            actual_balance: 0,
            protocol_fees_owed: 0,
            previous_selloff: 0,
            current_selloff: 0,
            window_start_timestamp: 0,
        }; MAX_TOKENS];

        let token_base = 179usize;
        let slot_size = 144usize;
        for i in 0..MAX_TOKENS {
            let off = token_base + i * slot_size;
            // AssetConfig (88 B): mint(32) + token_program(32) +
            //   normalized_weight(8) + max_selloff(8) +
            //   max_selloff_period_length(4) + reserved[4].
            let mint = read_pubkey(data, off);
            let token_program = read_pubkey(data, off + 32);
            let normalized_weight = read_u64(data, off + 64);
            let max_selloff = read_u64(data, off + 72);
            let max_selloff_period_length = read_u32(data, off + 80);
            // AssetDynamics (56 B) starts at off + 88: virtual_balance(8) +
            //   actual_balance(8) + protocol_fees_owed(8) +
            //   previous_selloff(8) + current_selloff(8) +
            //   window_start_timestamp(8) + reserved[8].
            let virtual_balance = read_u64(data, off + 88);
            let actual_balance = read_u64(data, off + 96);
            let protocol_fees_owed = read_u64(data, off + 104);
            let previous_selloff = read_u64(data, off + 112);
            let current_selloff = read_u64(data, off + 120);
            let window_start_timestamp = read_i64(data, off + 128);

            tokens[i] = TokenSlot {
                mint,
                token_program,
                normalized_weight,
                max_selloff,
                max_selloff_period_length,
                virtual_balance,
                actual_balance,
                protocol_fees_owed,
                previous_selloff,
                current_selloff,
                window_start_timestamp,
            };
        }

        Ok(PoolState {
            config,
            bump,
            token_count,
            pool_id,
            swap_fee_rate,
            protocol_fee_rate,
            pool_enabled,
            swaps_enabled,
            tokens,
        })
    }

    /// Active token slots (indices 0..token_count).
    pub fn active_tokens(&self) -> &[TokenSlot] {
        &self.tokens[..self.token_count as usize]
    }

    /// Find slot index for the given mint, if any.
    pub fn index_of_mint(&self, mint: &Pubkey) -> Option<usize> {
        self.active_tokens().iter().position(|t| &t.mint == mint)
    }
}

#[inline]
fn read_pubkey(data: &[u8], off: usize) -> Pubkey {
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&data[off..off + 32]);
    Pubkey::new_from_array(bytes)
}

#[inline]
fn read_u64(data: &[u8], off: usize) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&data[off..off + 8]);
    u64::from_le_bytes(bytes)
}

#[inline]
fn read_u32(data: &[u8], off: usize) -> u32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&data[off..off + 4]);
    u32::from_le_bytes(bytes)
}

#[inline]
fn read_i64(data: &[u8], off: usize) -> i64 {
    read_u64(data, off) as i64
}

#[inline]
fn read_u16(data: &[u8], off: usize) -> u16 {
    let mut bytes = [0u8; 2];
    bytes.copy_from_slice(&data[off..off + 2]);
    u16::from_le_bytes(bytes)
}
