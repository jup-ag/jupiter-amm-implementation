use anyhow::Result;
use jupiter_amm_interface::{Amm, AmmContext, AmmProgramIdToLabel, KeyedAccount};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::amms::spl_token_swap_amm::SplTokenSwapAmm;

type AmmFromKeyedAccount =
    Box<dyn Fn(&KeyedAccount, &AmmContext) -> Result<Box<dyn Amm + Send + Sync>> + Send + Sync>;

fn wrap_from_keyed_account<T: Amm + Send + Sync + 'static>() -> AmmFromKeyedAccount {
    Box::new(|keyed_account, amm_context| {
        T::from_keyed_account(keyed_account, amm_context)
            .map(|amm| Box::new(amm) as Box<dyn Amm + Send + Sync>)
    })
}

fn create_entries_for_amm<T: Amm + AmmProgramIdToLabel + Send + Sync + 'static>(
) -> Vec<(Pubkey, (&'static str, AmmFromKeyedAccount))> {
    T::PROGRAM_ID_TO_LABELS
        .iter()
        .map(|(program_id, label)| (*program_id, (*label, wrap_from_keyed_account::<T>())))
        .collect::<Vec<_>>()
}

pub static PROGRAM_ID_TO_AMM_LABEL_WITH_AMM_FROM_KEYED_ACCOUNT: LazyLock<
    HashMap<Pubkey, (&'static str, AmmFromKeyedAccount)>,
> = LazyLock::new(|| {
    let mut m = HashMap::new();

    m.extend(create_entries_for_amm::<SplTokenSwapAmm>());
    m
});

pub static PROGRAM_ID_TO_LABEL: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let program_id_to_label = HashMap::from_iter(
        PROGRAM_ID_TO_AMM_LABEL_WITH_AMM_FROM_KEYED_ACCOUNT
            .iter()
            .map(|(program_id, (label, _))| (program_id.to_string(), (*label).into())),
    );
    program_id_to_label
});
