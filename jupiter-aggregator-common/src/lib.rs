use std::ops::Range;

use anchor_lang::{InstructionData, ToAccountMetas};
use jupiter_common::find_program_address::find_program_address;
use rand::Rng;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_system_interface::program as system_program;

pub const AUTHORITY_SEED: &[u8] = b"authority";
pub const TOKEN_ACCOUNT_SEED: &[u8] = b"token_account";

// Now, we only support up to 16 authorities. To create more authorities, we need to
// add them in the monorepo. We can use from 0 up to 255 in order to prevent hot accounts.
pub const AUTHORITY_COUNT: u8 = if cfg!(feature = "staging") { 4 } else { 16 };
pub const AUTHORITY_COUNT_USIZE: usize = AUTHORITY_COUNT as usize;

pub const fn compute_program_authorities(program_id: Pubkey) -> [Pubkey; AUTHORITY_COUNT_USIZE] {
    let mut program_authorities = [Pubkey::new_from_array([0; 32]); AUTHORITY_COUNT_USIZE];
    let mut authority_id = 0;
    while authority_id < AUTHORITY_COUNT_USIZE {
        program_authorities[authority_id] =
            find_program_address(&[AUTHORITY_SEED, &[authority_id as u8]], &program_id).0;
        authority_id += 1;
    }
    program_authorities
}
