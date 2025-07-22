use const_crypto::ed25519;
use solana_sdk::pubkey::Pubkey;

pub const fn find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    let (pda, bump) = ed25519::derive_program_address(seeds, &program_id.to_bytes());
    (Pubkey::new_from_array(pda), bump)
}
