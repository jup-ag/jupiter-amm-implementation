//! PDA derivation for CipherDLMM accounts.
//!
//! Source of truth: backend_dlmm/programs/orbit_finance/src/seeds.rs

use solana_sdk::pubkey::Pubkey;

use crate::state::BinArrayState;
use crate::PROGRAM_ID;

/// BinArray PDA: [b"bin_array", pool, lower_bin_index.to_le_bytes()]
pub fn bin_array_pda(pool: &Pubkey, bin_index: i32) -> (Pubkey, u8) {
    let lower = BinArrayState::lower_bin_index_from(bin_index);
    Pubkey::find_program_address(
        &[b"bin_array", pool.as_ref(), &lower.to_le_bytes()],
        &PROGRAM_ID,
    )
}

/// Oracle PDA: [b"oracle", pool]
pub fn oracle_pda(pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"oracle", pool.as_ref()], &PROGRAM_ID)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn bin_array_pda_is_deterministic() {
        let pool = Pubkey::from_str("11111111111111111111111111111111").unwrap();
        let (a, _) = bin_array_pda(&pool, 100);
        let (b, _) = bin_array_pda(&pool, 100);
        assert_eq!(a, b);

        // Different bin in same array → same PDA
        let (c, _) = bin_array_pda(&pool, 65);
        let (d, _) = bin_array_pda(&pool, 127);
        assert_eq!(c, d);

        // Different array → different PDA
        let (e, _) = bin_array_pda(&pool, 64);
        let (f, _) = bin_array_pda(&pool, 128);
        assert_ne!(e, f);
    }
}
