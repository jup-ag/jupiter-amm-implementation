use anyhow::Result;
use spl_token::state::Account as TokenAccount;
use std::{collections::HashMap, convert::TryInto};

use crate::{
    curve::{swap_curve::get_swap_curve_result, Fees},
    utils::{get_transfer_fee, get_transfer_inverse_fee},
};
use lazy_static::lazy_static;
use solana_sdk::{program_pack::Pack, pubkey, pubkey::Pubkey};

use jupiter_amm_interface::{
    try_get_account_data, AccountMap, Amm, KeyedAccount, Quote, QuoteParams, Swap,
    SwapAndAccountMetas, SwapParams,
};

use super::{
    account_meta_from_goat_swap::GoatTokenSwap, state::{AmmConfig, PoolState}
};
use anchor_lang::prelude::*;

mod goat_swap_programs {
    use super::*;
    pub const GOAT: Pubkey = pubkey!("GoatAFSqACoMvJqvgW7aFACFkkArv69ezTJhS8xdEr5H");
}

lazy_static! {
    pub static ref GOAT_SWAP_PROGRAMS: HashMap<Pubkey, String> = {
        let mut m = HashMap::new();
        m.insert(goat_swap_programs::GOAT, "Goat".into());
        m
    };
}

pub struct GoatSwapAmm {
    key: Pubkey,
    label: String,
    state: PoolState,
    amm_config: AmmConfig,
    reserve_mints: [Pubkey; 2],
    mint_accounts: [solana_sdk::account::Account; 2],
    reserves: [u128; 2],
    program_id: Pubkey,
}

impl GoatSwapAmm {
    fn get_authority(&self) -> Pubkey {
        Pubkey::find_program_address(&[b"vault_and_lp_mint_auth_seed"], &self.program_id).0
    }

    fn get_quote_swap_base_input(&self, quote_params: &QuoteParams) -> Result<Quote> {
        let input_mint = quote_params.input_mint;
        let output_mint = quote_params.output_mint;

        let (
            input_mint_account,
            output_mint_account,
            swap_source_amount,
            swap_destination_amount,
        ) = {
            let mint_0_account = self.mint_accounts[0].clone();
            let mint_1_account = self.mint_accounts[1].clone();

            let (vault_0_amount, vault_1_amount) = self.state.vault_amount_without_fee(
                u64::try_from(self.reserves[0]).unwrap(),
                u64::try_from(self.reserves[1]).unwrap(),
            );

            if input_mint == self.state.token_0_mint {
                (
                    mint_0_account,
                    mint_1_account,
                    u128::try_from(vault_0_amount).unwrap(),
                    u128::try_from(vault_1_amount).unwrap(),
                )
            } else {
                (
                    mint_1_account,
                    mint_0_account,
                    u128::try_from(vault_1_amount).unwrap(),
                    u128::try_from(vault_0_amount).unwrap(),
                )
            }
        };

        let transfer_fee = get_transfer_fee(&input_mint_account, quote_params.amount)?;
        let amount_in_without_transfer_fee = quote_params.amount.saturating_sub(transfer_fee);

        // check tax
        let has_out_tax = !self.state.tax_disabled
            && self.state.out_tax_rate > 0
            && self.state.tax_mint.eq(&output_mint);
        let has_in_tax = !self.state.tax_disabled
            && self.state.in_tax_rate > 0
            && self.state.tax_mint.eq(&input_mint);

        let in_tax = if has_in_tax {
            let in_tax =
                Fees::tax_amount(amount_in_without_transfer_fee, self.state.in_tax_rate).unwrap();
            u64::try_from(in_tax).unwrap()
        } else {
            0
        };

        let actual_amount_in = amount_in_without_transfer_fee.saturating_sub(in_tax);

        assert!(
            actual_amount_in > 0,
            "actual_amount_in must be greater than 0"
        );

        let swap_result = get_swap_curve_result(
            actual_amount_in,
            swap_source_amount,
            swap_destination_amount,
            self.amm_config.trade_fee_rate,
            self.amm_config.protocol_fee_rate,
            self.amm_config.fund_fee_rate,
            self.state.lp_fee_rate,
            quote_params.swap_mode,
        )?;

        // calculate amount out with tax
        let out_tax = if has_out_tax {
            Fees::tax_amount(
                u64::try_from(swap_result.expected_output_amount).unwrap(),
                self.state.out_tax_rate,
            )
            .unwrap()
        } else {
            0
        };

        let actual_account_out = {
            let destination_amount_swapped_post_tax = swap_result
                .expected_output_amount
                .checked_sub(out_tax)
                .unwrap();

            let amount_out: u64 = u64::try_from(destination_amount_swapped_post_tax).unwrap();

            let transfer_fee = get_transfer_fee(&output_mint_account, amount_out).unwrap();
            let amount_received = amount_out.checked_sub(transfer_fee).unwrap();

            assert!(
                amount_received > 0,
                "amount_received must be greater than 0"
            );

            amount_received
        };

        let fee_amount = swap_result.fees.try_into().unwrap();

        Ok(Quote {
            fee_pct: swap_result.fee_pct,
            in_amount: quote_params.amount,
            not_enough_liquidity: swap_result.not_enough_liquidity,
            out_amount: actual_account_out,
            fee_amount: fee_amount,
            fee_mint: quote_params.input_mint,
            ..Quote::default()
        })
    }

    fn get_quote_swap_base_output(&self, quote_params: &QuoteParams) -> Result<Quote> {
        let input_mint = quote_params.input_mint;
        let output_mint = quote_params.output_mint;

        let (
            input_mint_account,
            output_mint_account,
            swap_source_amount,
            swap_destination_amount,
        ) = {
            let mint_0_account = self.mint_accounts[0].clone();
            let mint_1_account = self.mint_accounts[1].clone();

            let (vault_0_amount, vault_1_amount) = self.state.vault_amount_without_fee(
                u64::try_from(self.reserves[0]).unwrap(),
                u64::try_from(self.reserves[1]).unwrap(),
            );

            if input_mint == self.state.token_0_mint {
                (
                    mint_0_account,
                    mint_1_account,
                    u128::try_from(vault_0_amount).unwrap(),
                    u128::try_from(vault_1_amount).unwrap(),
                )
            } else {
                (
                    mint_1_account,
                    mint_0_account,
                    u128::try_from(vault_1_amount).unwrap(),
                    u128::try_from(vault_0_amount).unwrap(),
                )
            }
        };

        let out_transfer_fee = get_transfer_inverse_fee(&output_mint_account, quote_params.amount);

        // check tax
        let has_out_tax = !self.state.tax_disabled
            && self.state.out_tax_rate > 0
            && self.state.tax_mint.eq(&output_mint);
        let has_in_tax = !self.state.tax_disabled
            && self.state.in_tax_rate > 0
            && self.state.tax_mint.eq(&input_mint);

        let amount_out_with_tax = if has_out_tax {
            let amount_out_with_tax = Fees::calculate_pre_fee_amount(
                u128::try_from(quote_params.amount).unwrap(),
                self.state.out_tax_rate,
            )
            .unwrap();

            let amount_out_with_tax = u64::try_from(amount_out_with_tax).unwrap();

            amount_out_with_tax
        } else {
            quote_params.amount
        };

        let actual_amount_out = amount_out_with_tax.checked_add(out_transfer_fee).unwrap();

        let swap_result = get_swap_curve_result(
            actual_amount_out,
            swap_source_amount,
            swap_destination_amount,
            self.amm_config.trade_fee_rate,
            self.amm_config.protocol_fee_rate,
            self.amm_config.fund_fee_rate,
            self.state.lp_fee_rate,
            quote_params.swap_mode,
        )?;

        // calculate amount out with tax
        let amount_in_with_tax = if has_in_tax {
            Fees::calculate_pre_fee_amount(swap_result.input_amount, self.state.in_tax_rate)
                .unwrap()
        } else {
            swap_result.input_amount
        };

        assert!(
            amount_in_with_tax > 0,
            "amount_in_with_tax must be greater than 0"
        );

        let actual_in_amount = {
            let amount_in_with_tax = u64::try_from(amount_in_with_tax).unwrap();
            let transfer_fee = get_transfer_inverse_fee(&input_mint_account, amount_in_with_tax);
            let actual_in_amount = amount_in_with_tax.checked_add(transfer_fee).unwrap();

            actual_in_amount
        };

        let fee_amount = swap_result.fees.try_into().unwrap();

        Ok(Quote {
            fee_pct: swap_result.fee_pct,
            in_amount: actual_in_amount,
            not_enough_liquidity: swap_result.not_enough_liquidity,
            out_amount: quote_params.amount,
            fee_amount: fee_amount,
            fee_mint: quote_params.input_mint,
            ..Quote::default()
        })
    }
}

impl Clone for GoatSwapAmm {
    fn clone(&self) -> Self {
        GoatSwapAmm {
            key: self.key,
            label: self.label.clone(),
            state: PoolState {
                amm_config: self.state.amm_config,
                auth_bump: self.state.auth_bump,
                token_0_mint: self.state.token_0_mint,
                token_1_mint: self.state.token_1_mint,
                lp_mint: self.state.lp_mint,
                token_0_vault: self.state.token_0_vault,
                token_1_vault: self.state.token_1_vault,
                protocol_fees_token_0: self.state.protocol_fees_token_0,
                protocol_fees_token_1: self.state.protocol_fees_token_1,
                token_0_program: self.state.token_0_program,
                token_1_program: self.state.token_1_program,
                pool_creator: self.state.pool_creator,
                fund_fees_token_0: self.state.fund_fees_token_0,
                fund_fees_token_1: self.state.fund_fees_token_1,
                lp_mint_decimals: self.state.lp_mint_decimals,
                lp_supply: self.state.lp_supply,
                mint_0_decimals: self.state.mint_0_decimals,
                mint_1_decimals: self.state.mint_1_decimals,
                open_time: self.state.open_time,
                status: self.state.status,
                tax_mint: self.state.tax_mint,
                in_tax_rate: self.state.in_tax_rate,
                out_tax_rate: self.state.out_tax_rate,
                tax_amount_0: self.state.tax_amount_0,
                tax_amount_1: self.state.tax_amount_1,
                tax_authority: self.state.tax_authority,
                tax_disabled: self.state.tax_disabled,
                lp_fee_rate: self.state.lp_fee_rate,
                padding: self.state.padding,
            },
            amm_config: AmmConfig {
                bump: self.amm_config.bump,
                create_pool_fee: self.amm_config.create_pool_fee,
                disable_create_pool: self.amm_config.disable_create_pool,
                fund_fee_rate: self.amm_config.fund_fee_rate,
                protocol_fee_rate: self.amm_config.protocol_fee_rate,
                trade_fee_rate: self.amm_config.trade_fee_rate,
                fund_owner: self.amm_config.fund_owner,
                index: self.amm_config.index,
                protocol_owner: self.amm_config.protocol_owner,
                padding: self.amm_config.padding,
            },
            reserve_mints: self.reserve_mints,
            mint_accounts: self.mint_accounts.clone(),
            program_id: self.program_id,
            reserves: self.reserves,
        }
    }
}

impl Amm for GoatSwapAmm {
    fn from_keyed_account(keyed_account: &KeyedAccount) -> Result<Self> {
        let state: PoolState = PoolState::try_from_slice(&keyed_account.account.data[8..])?;

        let reserve_mints = [state.token_0_mint, state.token_1_mint];

        let label = GOAT_SWAP_PROGRAMS
            .get(&keyed_account.account.owner)
            .unwrap()
            .clone();

        Ok(Self {
            key: keyed_account.key,
            label,
            state,
            amm_config: AmmConfig::default(),
            reserve_mints,
            mint_accounts: [
                solana_sdk::account::Account::default(),
                solana_sdk::account::Account::default(),
            ],
            program_id: keyed_account.account.owner,
            reserves: Default::default(),
        })
    }

    fn label(&self) -> String {
        self.label.clone()
    }

    fn program_id(&self) -> Pubkey {
        self.program_id
    }

    fn key(&self) -> Pubkey {
        self.key
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        self.reserve_mints.to_vec()
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        vec![
            self.state.token_0_vault,
            self.state.token_1_vault,
            self.state.amm_config,
            self.state.token_0_mint,
            self.state.token_1_mint,
        ]
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        let token_0_vault = try_get_account_data(account_map, &self.state.token_0_vault)?;
        let token_0_vault_token_account = TokenAccount::unpack(token_0_vault)?;

        let token_1_vault = try_get_account_data(account_map, &self.state.token_1_vault)?;
        let token_1_vault_token_account = TokenAccount::unpack(token_1_vault)?;

        self.reserves = [
            token_0_vault_token_account.amount.into(),
            token_1_vault_token_account.amount.into(),
        ];

        let amm_config_data = try_get_account_data(account_map, &self.state.amm_config)?;
        self.amm_config = AmmConfig::try_from_slice(&amm_config_data[8..])?;

        let token_0_mint_account = account_map.get(&self.state.token_0_mint).unwrap();
        let token_1_mint_account = account_map.get(&self.state.token_1_mint).unwrap();

        self.mint_accounts = [token_0_mint_account.clone(), token_1_mint_account.clone()];

        Ok(())
    }

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote> {
        match quote_params.swap_mode {
            jupiter_amm_interface::SwapMode::ExactIn => {
                self.get_quote_swap_base_input(quote_params)
            }
            jupiter_amm_interface::SwapMode::ExactOut => {
                self.get_quote_swap_base_output(quote_params)
            }
        }
    }

    fn get_swap_and_account_metas(&self, swap_params: &SwapParams) -> Result<SwapAndAccountMetas> {
        let SwapParams {
            token_transfer_authority,
            source_token_account,
            destination_token_account,
            source_mint,
            ..
        } = swap_params;

        let (swap_source, swap_destination) = if *source_mint == self.state.token_0_mint {
            (self.state.token_0_vault, self.state.token_1_vault)
        } else {
            (self.state.token_1_vault, self.state.token_0_vault)
        };

        Ok(SwapAndAccountMetas {
            swap: Swap::TokenSwap,
            account_metas: GoatTokenSwap {
                token_swap_program: self.program_id,
                token_program: spl_token::id(),
                swap: self.key,
                authority: self.get_authority(),
                user_transfer_authority: *token_transfer_authority,
                source: *source_token_account,
                destination: *destination_token_account,
                pool_mint: self.state.lp_mint,
                swap_destination,
                swap_source,
                pool_fee: swap_source,
                amm_config: self.state.amm_config,
                output_token_program: spl_token::id(),
            }
            .into(),
        })
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}
