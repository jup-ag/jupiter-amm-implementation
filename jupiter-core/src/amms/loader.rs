use anyhow::Result;
use jupiter_amm_interface::{Amm, AmmContext, KeyedAccount};

use crate::amms::amm_program_id_to_labels::PROGRAM_ID_TO_AMM_LABEL_WITH_AMM_FROM_KEYED_ACCOUNT;

pub fn amm_factory(
    keyed_account: &KeyedAccount,
    amm_context: &AmmContext,
) -> Result<Option<Box<dyn Amm + Send + Sync>>> {
    let owner = keyed_account.account.owner;

    PROGRAM_ID_TO_AMM_LABEL_WITH_AMM_FROM_KEYED_ACCOUNT
        .get(&owner)
        .map(|(_, f)| f(keyed_account, amm_context))
        .transpose()
}
