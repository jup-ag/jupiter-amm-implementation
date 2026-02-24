//! On-chain account layout types for MnM DLMM.
//!
//! Zero-copy deserialization of PoolState and BinArray accounts.

pub const DISC: usize = 8;
pub const BINS_PER_ARRAY: usize = 64;
pub const MAX_BIN_CROSSINGS: u8 = 4;

/// PoolState — 384 bytes (including alignment padding).
#[derive(Clone, Copy)]
#[repr(C)]
pub struct PoolState {
    pub authority: [u8; 32],
    pub token_mint_x: [u8; 32],
    pub token_mint_y: [u8; 32],
    pub token_vault_x: [u8; 32],
    pub token_vault_y: [u8; 32],
    pub active_bin_id: i32,
    pub bin_step: u16,
    pub protocol_fee_bps: u16,
    pub lp_fee_bps: u16,
    pub _padding1: [u8; 6],
    pub total_liquidity_x: u64,
    pub total_liquidity_y: u64,
    pub fee_growth_global_x: u128,
    pub fee_growth_global_y: u128,
    pub protocol_fees_x: u64,
    pub protocol_fees_y: u64,
    pub is_paused: u8,
    pub bump: u8,
    pub _padding2: [u8; 6],
    pub shift_enabled: u8,
    pub shift_default_mode: u8,
    pub max_shift_bins: u8,
    pub _shift_padding: [u8; 5],
    pub shift_cooldown_slots: u64,
    pub last_shift_slot: u64,
    pub token_program_x: [u8; 32],
    pub token_program_y: [u8; 32],
    pub _reserved: [u8; 40],
}

unsafe impl bytemuck::Pod for PoolState {}
unsafe impl bytemuck::Zeroable for PoolState {}

/// Single bin — 64 bytes.
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Bin {
    pub liquidity_x: u64,
    pub liquidity_y: u64,
    pub fee_growth_x: u128,
    pub fee_growth_y: u128,
    pub shift_liquidity_x: u64,
    pub shift_liquidity_y: u64,
}

/// BinArray — 4208 bytes (including alignment padding).
#[derive(Clone, Copy)]
#[repr(C)]
pub struct BinArray {
    pub pool: [u8; 32],
    pub index: i32,
    pub bump: u8,
    pub _padding: [u8; 3],
    pub bins: [Bin; BINS_PER_ARRAY],
    pub active_bins: u64,
    pub _reserved: [u8; 56],
}

unsafe impl bytemuck::Pod for BinArray {}
unsafe impl bytemuck::Zeroable for BinArray {}

/// Deserialize a zero-copy Anchor account (skip 8-byte discriminator).
///
/// Copies data into a zeroed `T` to guarantee alignment. This avoids
/// bytemuck `from_bytes` alignment panics when account data arrives from
/// RPC with arbitrary pointer alignment (u128 fields require 16-byte).
pub fn deserialize<T: bytemuck::Pod>(data: &[u8]) -> Option<T> {
    let size = core::mem::size_of::<T>();
    if data.len() < DISC { return None; }
    let available = data.len() - DISC;
    let copy_len = available.min(size);
    let mut val = T::zeroed();
    let dst = bytemuck::bytes_of_mut(&mut val);
    dst[..copy_len].copy_from_slice(&data[DISC..DISC + copy_len]);
    Some(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_state_deserialize_unaligned() {
        let size = DISC + core::mem::size_of::<PoolState>();
        let mut buf = vec![0u8; size + 1];
        let unaligned = &mut buf[1..1 + size];
        unaligned[..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        let bin_id_offset = DISC + 160;
        unaligned[bin_id_offset..bin_id_offset + 4].copy_from_slice(&42i32.to_le_bytes());
        let pool: PoolState = deserialize(unaligned).expect("deserialize should succeed");
        assert_eq!(pool.active_bin_id, 42);
    }

    #[test]
    fn bin_array_deserialize_unaligned() {
        let size = DISC + core::mem::size_of::<BinArray>();
        let mut buf = vec![0u8; size + 1];
        let unaligned = &mut buf[1..1 + size];
        unaligned[..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        let index_offset = DISC + 32;
        unaligned[index_offset..index_offset + 4].copy_from_slice(&(-7i32).to_le_bytes());
        let arr: BinArray = deserialize(unaligned).expect("deserialize should succeed");
        assert_eq!(arr.index, -7);
    }
}
