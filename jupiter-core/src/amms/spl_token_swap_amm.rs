use anyhow::{ensure, Context, Result};
use program_interfaces::jupiter_dex_interfaces::client::accounts::TokenSwap;
use spl_token::state::Account as TokenAccount;
use std::{collections::HashMap, convert::TryInto, sync::LazyLock};

use crate::amm::*;
use crate::math::swap_curve_info::get_swap_curve_result;
use jupiter_amm_interface::{
    try_get_account_data, AccountMap, AmmContext, AmmLabel, AmmProgramIdToLabel,
};
use solana_sdk::{program_pack::Pack, pubkey, pubkey::Pubkey};
use spl_token_swap::{
    curve::{
        base::{CurveType, SwapCurve},
        calculator::TradeDirection,
    },
    state::SwapV1,
};

pub mod spl_token_swap_programs {
    use super::*;
    pub const ORCA_V1: Pubkey = pubkey!("DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1");
    pub const ORCA_V2: Pubkey = pubkey!("9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP");
    pub const STEPN: Pubkey = pubkey!("Dooar9JkhdZ7J3LHN3A7YCuoGRUggXhQaG4kijfLGU2j");
    pub const SAROS: Pubkey = pubkey!("SSwapUtytfBdBn1b9NUGG6foMVPtcWgpRU32HToDUZr");
    pub const PENGUIN: Pubkey = pubkey!("PSwapMdSai8tjrEXcxFeQth87xC4rRsa4VA5mhGhXkP");
    pub const SPL_TOKEN_SWAP: Pubkey = pubkey!("SwaPpA9LAaLfeLi3a68M4DjnLqgtticKg6CnyNwgAC8");
}

pub struct SplTokenSwapAmm {
    key: Pubkey,
    authority: Pubkey,
    label: String,
    state: SwapV1,
    reserve_mints: [Pubkey; 2],
    reserves: [u128; 2],
    program_id: Pubkey,
}

impl AmmProgramIdToLabel for SplTokenSwapAmm {
    const PROGRAM_ID_TO_LABELS: &[(Pubkey, AmmLabel)] = &[
        (spl_token_swap_programs::ORCA_V1, "Orca V1"),
        (spl_token_swap_programs::ORCA_V2, "Orca V2"),
        (spl_token_swap_programs::STEPN, "StepN"),
        (spl_token_swap_programs::SAROS, "Saros"),
        (spl_token_swap_programs::PENGUIN, "Penguin"),
        (spl_token_swap_programs::SPL_TOKEN_SWAP, "Token Swap"),
    ];
}

pub static SPL_TOKEN_SWAP_PROGRAMS: LazyLock<HashMap<Pubkey, String>> = LazyLock::new(|| {
    HashMap::from_iter(
        SplTokenSwapAmm::PROGRAM_ID_TO_LABELS
            .iter()
            .map(|(program_id, amm_label)| (*program_id, (*amm_label).into())),
    )
});

impl Clone for SplTokenSwapAmm {
    fn clone(&self) -> Self {
        SplTokenSwapAmm {
            key: self.key,
            authority: self.authority,
            label: self.label.clone(),
            state: SwapV1 {
                is_initialized: self.state.is_initialized,
                bump_seed: self.state.bump_seed,
                token_program_id: self.state.token_program_id,
                token_a: self.state.token_a,
                token_b: self.state.token_b,
                pool_mint: self.state.pool_mint,
                token_a_mint: self.state.token_a_mint,
                token_b_mint: self.state.token_b_mint,
                pool_fee_account: self.state.pool_fee_account,
                fees: self.state.fees.clone(),
                swap_curve: SwapCurve {
                    curve_type: self.state.swap_curve.curve_type,
                    calculator: self.state.swap_curve.calculator.clone(),
                },
            },
            reserve_mints: self.reserve_mints,
            program_id: self.program_id,
            reserves: self.reserves,
        }
    }
}

impl Amm for SplTokenSwapAmm {
    fn from_keyed_account(keyed_account: &KeyedAccount, _amm_context: &AmmContext) -> Result<Self> {
        // Skip the first byte which is version
        let state = SwapV1::unpack(&keyed_account.account.data[1..])?;

        // Support only the most common non exotic curves
        ensure!(matches!(
            state.swap_curve.curve_type,
            CurveType::ConstantProduct | CurveType::Stable
        ));

        let reserve_mints = [state.token_a_mint, state.token_b_mint];

        let label = SPL_TOKEN_SWAP_PROGRAMS
            .get(&keyed_account.account.owner)
            .cloned()
            .context("Label not found")?;
        let program_id = keyed_account.account.owner;
        Ok(Self {
            key: keyed_account.key,
            authority: Pubkey::find_program_address(&[&keyed_account.key.to_bytes()], &program_id)
                .0,
            label,
            state,
            reserve_mints,
            program_id,
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
        vec![self.state.token_a, self.state.token_b]
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        let token_a_account = try_get_account_data(account_map, &self.state.token_a)?;
        let token_a_token_account = TokenAccount::unpack(token_a_account)?;

        let token_b_account = try_get_account_data(account_map, &self.state.token_b)?;
        let token_b_token_account = TokenAccount::unpack(token_b_account)?;

        self.reserves = [
            token_a_token_account.amount.into(),
            token_b_token_account.amount.into(),
        ];

        Ok(())
    }

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote> {
        let (trade_direction, swap_source_amount, swap_destination_amount) =
            if quote_params.input_mint == self.reserve_mints[0] {
                (TradeDirection::AtoB, self.reserves[0], self.reserves[1])
            } else {
                (TradeDirection::BtoA, self.reserves[1], self.reserves[0])
            };

        let swap_result = get_swap_curve_result(
            &self.state.swap_curve,
            quote_params.amount,
            swap_source_amount,
            swap_destination_amount,
            trade_direction,
            &self.state.fees,
        )?;

        Ok(Quote {
            fee_pct: swap_result.fee_pct,
            in_amount: swap_result.input_amount.try_into()?,
            out_amount: swap_result.expected_output_amount.try_into()?,
            fee_amount: swap_result.fees.try_into()?,
            fee_mint: quote_params.input_mint,
        })
    }

    fn get_accounts_len(&self) -> usize {
        11
    }

    fn get_swap_and_account_metas(&self, swap_params: &SwapParams) -> Result<SwapAndAccountMetas> {
        let SwapParams {
            source_mint,
            destination_token_account,
            source_token_account,
            token_transfer_authority,
            ..
        } = swap_params;

        let (swap_source, swap_destination) = if *source_mint == self.state.token_a_mint {
            (self.state.token_a, self.state.token_b)
        } else {
            (self.state.token_b, self.state.token_a)
        };

        Ok(SwapAndAccountMetas {
            swap: Swap::TokenSwap,
            account_metas: to_dex_account_metas(
                self.program_id,
                TokenSwap {
                    token_program: spl_token::ID,
                    swap: self.key,
                    authority: self.authority,
                    user_transfer_authority: *token_transfer_authority,
                    source: *source_token_account,
                    swap_source,
                    swap_destination,
                    destination: *destination_token_account,
                    pool_mint: self.state.pool_mint,
                    pool_fee: self.state.pool_fee_account,
                },
            ),
        })
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}
