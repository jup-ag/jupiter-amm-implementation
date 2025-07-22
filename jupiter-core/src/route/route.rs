use itertools::Itertools;
use jupiter_amm_interface::Amm;
use solana_sdk::pubkey::Pubkey;

fn get_two_permutations<T: PartialEq + Clone>(items: &[T]) -> Vec<(T, T)> {
    items
        .iter()
        .permutations(2)
        .map(|permutation| (permutation[0].clone(), permutation[1].clone()))
        .collect()
}

pub fn get_token_mints_permutations(
    amm: &dyn Amm,
    allow_unidirectional_restriction: bool,
) -> Vec<(Pubkey, Pubkey)> {
    if allow_unidirectional_restriction && amm.unidirectional() {
        vec![amm.get_reserve_mints().into_iter().collect_tuple().unwrap()]
    } else {
        get_two_permutations(&amm.get_reserve_mints())
    }
}
