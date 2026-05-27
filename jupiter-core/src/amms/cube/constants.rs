//! Compile-time constants for Cube DEX (cubic-pool).

use solana_sdk::{pubkey, pubkey::Pubkey};

/// Mainnet / devnet / localnet cubic_pool program id.
pub const CUBIC_POOL_PROGRAM_ID: Pubkey = pubkey!("8iQtGj9mcUfFUGaiCpPy89swC3s8YTC8FhVZWfgeZhwu");

/// Human-readable label reported to Jupiter UI / routing.
pub const CUBE_LABEL: &str = "Cube";

/// 18-decimal fixed-point precision (1e18).
pub const FIXED_POINT_PRECISION: u128 = 1_000_000_000_000_000_000;
pub const ONE: u128 = FIXED_POINT_PRECISION;

/// Weights sum to 10_000 (= 100%).
pub const WEIGHT_SCALE: u64 = 10_000;

/// Denominator for `swap_fee_rate`. 1_000_000 = 100% (hundredths-of-bps).
pub const SWAP_FEE_PRECISION: u64 = 1_000_000;

/// Denominator for `protocol_fee_rate`. 10_000 = 100% (bps).
pub const PROTOCOL_FEE_PRECISION: u64 = 10_000;

/// Pool token-slot maximum.
pub const MAX_TOKENS: usize = 10;

/// `CubicPool` data length, v4 layout (includes 8-byte discriminator).
pub const POOL_V4_LEN: usize = 1683;

/// `CubicPool` legacy v3 data length (parallel-array layout).
pub const POOL_V3_LEN: usize = 1154;

/// Anchor account discriminator for `CubicPool`
/// (`sha256("account:CubicPool")[..8]`).
pub const POOL_DISCRIMINATOR: [u8; 8] = [137, 210, 42, 22, 209, 156, 43, 78];

/// Anchor instruction discriminator for `swap` (`sha256("global:swap")[..8]`).
pub const SWAP_IX_DISCRIMINATOR: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

pub const CUBIC_POOL_SEED: &[u8] = b"cubic_pool";
pub const BPT_MINT_SEED: &[u8] = b"bpt_mint";

/// Convert a basis-point weight (0..=10_000) to 18-decimal fixed-point.
#[inline]
pub fn weight_to_fixed_point(weight_bps: u64) -> u128 {
    (weight_bps as u128) * ONE / (WEIGHT_SCALE as u128)
}
