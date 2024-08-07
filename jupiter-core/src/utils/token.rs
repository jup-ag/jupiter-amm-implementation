use anchor_lang::prelude::*;
use anchor_spl::{
    token::Token,
    token_2022::spl_token_2022::{
        self,
        extension::{
            transfer_fee::{TransferFeeConfig, MAX_FEE_BASIS_POINTS},
            StateWithExtensions,
        },
    },
    token_interface::spl_token_2022::extension::BaseStateWithExtensions,
};
use solana_sdk::account::{Account, ReadableAccount};

/// Calculate the fee for output amount
pub fn get_transfer_inverse_fee(mint_info: &Account, post_fee_amount: u64) -> u64 {
    if mint_info.owner == Token::id() {
        return 0;
    }

    assert!(
        post_fee_amount > 0,
        "post_fee_amount must be greater than 0"
    );

    let mint_data = mint_info.data();
    let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data).unwrap();

    let fee = if let Ok(transfer_fee_config) = mint.get_extension::<TransferFeeConfig>() {
        let epoch = Clock::get().unwrap().epoch;

        let transfer_fee = transfer_fee_config.get_epoch_fee(epoch);
        if u16::from(transfer_fee.transfer_fee_basis_points) == MAX_FEE_BASIS_POINTS {
            u64::from(transfer_fee.maximum_fee)
        } else {
            transfer_fee_config
                .calculate_inverse_epoch_fee(epoch, post_fee_amount)
                .unwrap()
        }
    } else {
        0
    };

    fee
}

pub fn get_transfer_fee(mint_info: &Account, pre_fee_amount: u64) -> Result<u64> {
    if mint_info.owner == Token::id() {
        return Ok(0);
    }

    let mint_data = mint_info.data();
    let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)?;

    let fee = if let Ok(transfer_fee_config) = mint.get_extension::<TransferFeeConfig>() {
        transfer_fee_config
            .calculate_epoch_fee(Clock::get()?.epoch, pre_fee_amount)
            .unwrap()
    } else {
        0
    };
    Ok(fee)
}
