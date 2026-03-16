//! On-chain state deserialization for CipherDLMM (Orbit Finance).
//!
//! Deserializes Pool and BinArray accounts from raw bytes using bytemuck
//! (zero-copy, matching the on-chain #[repr(C)] layout).
//!
//! Source of truth:
//!   backend_dlmm/programs/orbit_finance/src/state/pool.rs
//!   backend_dlmm/programs/orbit_finance/src/state/bin_array.rs

use bytemuck::{Pod, Zeroable};
use solana_sdk::pubkey::Pubkey;

/// Anchor discriminator length (8 bytes).
pub const ANCHOR_DISC_LEN: usize = 8;

/// Number of bins per BinArray account.
pub const BIN_ARRAY_SIZE: usize = 64;

// ---------------------------------------------------------------------------
// RewardIndexes (embedded in Pool, 32 bytes)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct RewardIndexes {
    pub holders_q128: u128,
    pub nft_q128: u128,
}

// SAFETY: #[repr(C)] with only u128 fields — trivially Pod.
unsafe impl Zeroable for RewardIndexes {}
unsafe impl Pod for RewardIndexes {}

// ---------------------------------------------------------------------------
// Pool account (604 bytes body, 612 with disc)
// ---------------------------------------------------------------------------

/// Pool account layout matching the on-chain `#[repr(C)] #[account(zero_copy)]`.
/// Field order exactly mirrors `backend_dlmm/programs/orbit_finance/src/state/pool.rs`.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PoolState {
    // 14 Pubkeys (14 * 32 = 448)
    pub admin: Pubkey,
    pub config_authority: Pubkey,
    pub pause_guardian: Pubkey,
    pub fee_withdraw_authority: Pubkey,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub creator_fee_vault: Pubkey,
    pub holders_fee_vault: Pubkey,
    pub nft_fee_vault: Pubkey,
    pub protocol_fee_vault: Pubkey,
    pub lp_mint: Pubkey,

    // 4 u128 fields (64 bytes)
    pub price_q64_64: u128,
    pub total_shares: u128,
    pub total_holder_units: u128,
    pub total_nft_units: u128,

    // RewardIndexes (32 bytes)
    pub reward_indexes: RewardIndexes,

    // 3 i64 fields (24 bytes)
    pub last_updated: i64,
    pub last_swap_time: i64,
    pub last_volatility_update: i64,

    // 4 i32 fields (16 bytes)
    pub initial_bin_id: i32,
    pub active_bin: i32,
    pub previous_bin: i32,
    pub reference_bin: i32,

    // 7 u32 fields (28 bytes) — FeeConfig flattened
    pub split_holders_microbps: u32,
    pub split_nft_microbps: u32,
    pub split_creator_extra_microbps: u32,
    pub variable_fee_control: u32,
    pub max_volatility_accumulator: u32,
    pub volatility_reference: u32,
    pub volatility_accumulator: u32,

    // 8 u16 fields (16 bytes)
    pub bin_step_bps: u16,
    pub base_fee_bps: u16,
    pub creator_cut_bps: u16,
    pub legacy_volatility_multiplier_bps: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor_bps: u16,
    pub max_dynamic_fee_bps: u16,

    // u8 fields + padding (12 bytes)
    pub version: u8,
    pub bump: u8,
    pub pause_bits: u8,
    pub accounting_mode: u8,
    pub dynamic_fee_enabled: u8,
    pub _fee_reserved: [u8; 5],
    pub _pad2: [u8; 2],
}

// SAFETY: #[repr(C)] struct with all Pod-safe fields (Pubkey is [u8; 32] internally).
unsafe impl Zeroable for PoolState {}
unsafe impl Pod for PoolState {}

impl PoolState {
    pub const LEN: usize = 640;

    /// PAUSE_SWAP bitmask (bit 0).
    const PAUSE_SWAP: u8 = 1;

    #[inline]
    pub fn is_swap_paused(&self) -> bool {
        (self.pause_bits & Self::PAUSE_SWAP) != 0
    }
}

// ---------------------------------------------------------------------------
// CompactBin (80 bytes)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct CompactBin {
    pub reserve_base: u128,
    pub reserve_quote: u128,
    pub total_shares: u128,
    pub fee_growth_base_q128: u128,
    pub fee_growth_quote_q128: u128,
}

unsafe impl Zeroable for CompactBin {}
unsafe impl Pod for CompactBin {}

impl CompactBin {
    pub const LEN: usize = 80;

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.total_shares == 0 && self.reserve_base == 0 && self.reserve_quote == 0
    }
}

// ---------------------------------------------------------------------------
// BinArray account (5168 bytes body, 5176 with disc)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BinArrayState {
    pub pool: Pubkey,
    pub bins: [CompactBin; BIN_ARRAY_SIZE],
    pub lower_bin_index: i32,
    pub bump: u8,
    pub _reserved: [u8; 11],
}

unsafe impl Zeroable for BinArrayState {}
unsafe impl Pod for BinArrayState {}

impl BinArrayState {
    pub const LEN: usize = 32 + (CompactBin::LEN * BIN_ARRAY_SIZE) + 4 + 1 + 11;

    /// Calculate the lower_bin_index (array boundary) for a given bin_index.
    /// Must match the on-chain floor division logic.
    #[inline]
    pub fn lower_bin_index_from(bin_index: i32) -> i32 {
        let size = BIN_ARRAY_SIZE as i32;
        let q = bin_index / size;
        let r = bin_index % size;
        let array_number = if r < 0 { q - 1 } else { q };
        array_number * size
    }

    /// Get a bin by its absolute bin_index.
    #[inline]
    pub fn get_bin(&self, bin_index: i32) -> Option<&CompactBin> {
        let offset = bin_index.checked_sub(self.lower_bin_index)?;
        if offset < 0 || offset >= BIN_ARRAY_SIZE as i32 {
            return None;
        }
        Some(&self.bins[offset as usize])
    }
}

// ---------------------------------------------------------------------------
// Deserialization helpers
// ---------------------------------------------------------------------------

/// Deserialize a Pool from raw account data (includes 8-byte Anchor discriminator).
pub fn deserialize_pool(data: &[u8]) -> anyhow::Result<PoolState> {
    if data.len() < ANCHOR_DISC_LEN + PoolState::LEN {
        anyhow::bail!(
            "Pool account too small: {} bytes (need {})",
            data.len(),
            ANCHOR_DISC_LEN + PoolState::LEN
        );
    }
    let body = &data[ANCHOR_DISC_LEN..ANCHOR_DISC_LEN + PoolState::LEN];
    Ok(*bytemuck::from_bytes::<PoolState>(body))
}

/// Deserialize a BinArray from raw account data (includes 8-byte Anchor discriminator).
pub fn deserialize_bin_array(data: &[u8]) -> anyhow::Result<BinArrayState> {
    if data.len() < ANCHOR_DISC_LEN + BinArrayState::LEN {
        anyhow::bail!(
            "BinArray account too small: {} bytes (need {})",
            data.len(),
            ANCHOR_DISC_LEN + BinArrayState::LEN
        );
    }
    let body = &data[ANCHOR_DISC_LEN..ANCHOR_DISC_LEN + BinArrayState::LEN];
    Ok(*bytemuck::from_bytes::<BinArrayState>(body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_size_matches() {
        assert_eq!(
            std::mem::size_of::<PoolState>(),
            PoolState::LEN,
            "PoolState struct size mismatch — check field order and alignment"
        );
    }

    #[test]
    fn bin_array_size_matches() {
        assert_eq!(
            std::mem::size_of::<BinArrayState>(),
            BinArrayState::LEN,
            "BinArrayState struct size mismatch"
        );
    }

    #[test]
    fn compact_bin_size() {
        assert_eq!(std::mem::size_of::<CompactBin>(), 80);
    }

    #[test]
    fn lower_bin_index_from_positive() {
        assert_eq!(BinArrayState::lower_bin_index_from(150), 128);
        assert_eq!(BinArrayState::lower_bin_index_from(128), 128);
        assert_eq!(BinArrayState::lower_bin_index_from(0), 0);
        assert_eq!(BinArrayState::lower_bin_index_from(63), 0);
    }

    #[test]
    fn lower_bin_index_from_negative() {
        assert_eq!(BinArrayState::lower_bin_index_from(-1), -64);
        assert_eq!(BinArrayState::lower_bin_index_from(-64), -64);
        assert_eq!(BinArrayState::lower_bin_index_from(-65), -128);
    }
}
