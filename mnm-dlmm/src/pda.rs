//! PDA derivation for MnM DLMM accounts.

use solana_sdk::pubkey::Pubkey;
use super::MNM_PROGRAM_ID;

pub fn find_tick_map(pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"tick_map", pool.as_ref()], &MNM_PROGRAM_ID)
}

pub fn find_bin_array(pool: &Pubkey, index: i32) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"bin_array", pool.as_ref(), &index.to_le_bytes()],
        &MNM_PROGRAM_ID,
    )
}

pub fn find_vault_x(pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault_x", pool.as_ref()], &MNM_PROGRAM_ID)
}

pub fn find_vault_y(pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault_y", pool.as_ref()], &MNM_PROGRAM_ID)
}
