use anchor_lang::ToAccountMetas;
use anyhow::{Context, Result};
use spl_token::native_mint;
use spl_token::state::Account as TokenAccount;
use std::{collections::HashMap, convert::TryInto};

use crate::amms::amm::{Amm, KeyedAccount};
use lazy_static::lazy_static;
use solana_sdk::{program_pack::Pack, pubkey, pubkey::Pubkey};
use spl_token_swap::curve::base::SwapCurve;
use spl_token_swap::{curve::calculator::TradeDirection, state::SwapV1};

use super::amm::{Quote, QuoteParams, SwapLegAndAccountMetas, SwapParams};
use jupiter::{
    accounts::TokenSwap,
    jupiter_override::{Swap, SwapLeg},
};

mod spl_token_swap_programs {
    use super::*;
    pub const ORCA_V1: Pubkey = pubkey!("DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1");
    pub const ORCA_V2: Pubkey = pubkey!("9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP");
    pub const STEPN: Pubkey = pubkey!("Dooar9JkhdZ7J3LHN3A7YCuoGRUggXhQaG4kijfLGU2j");
    pub const SAROS: Pubkey = pubkey!("SSwapUtytfBdBn1b9NUGG6foMVPtcWgpRU32HToDUZr");
    pub const PENGUIN: Pubkey = pubkey!("PSwapMdSai8tjrEXcxFeQth87xC4rRsa4VA5mhGhXkP");
}

// export const PROGRAM_ID_TO_LABEL = new Map<string, string>([
//   ["DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1", "Orca v1"],
//   ["9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP", "Orca"],
//   [STEP_TOKEN_SWAP_PROGRAM_ID.toBase58(), "Step"],
//   ["PSwapMdSai8tjrEXcxFeQth87xC4rRsa4VA5mhGhXkP", "Penguin"],
//   ["SSwapUtytfBdBn1b9NUGG6foMVPtcWgpRU32HToDUZr", "Saros"],
//   ["Dooar9JkhdZ7J3LHN3A7YCuoGRUggXhQaG4kijfLGU2j", "StepN"],
// ]);

lazy_static! {
    pub static ref SPL_TOKEN_SWAP_PROGRAMS: HashMap<Pubkey, String> = {
        let mut m = HashMap::new();
        m.insert(spl_token_swap_programs::ORCA_V1, "Orca v1".into());
        m.insert(spl_token_swap_programs::ORCA_V2, "Orca v2".into());
        // m.insert(spl_token_swap_programs::STEP, "Step".into()); We need to support the STEP state
        m.insert(spl_token_swap_programs::STEPN, "StepN".into());
        m.insert(spl_token_swap_programs::SAROS, "Saros".into());
        m.insert(spl_token_swap_programs::PENGUIN, "Penguin".into());
        m
    };
}

pub struct SplTokenSwapAmm {
    key: Pubkey,
    label: String,
    state: SwapV1,
    reserve_mints: [Pubkey; 2],
    reserves: [u128; 2],
    program_id: Pubkey,
}

impl SplTokenSwapAmm {
    pub fn from_keyed_account(keyed_account: &KeyedAccount) -> Result<Self> {
        // Skip the first byte which is version
        let state = SwapV1::unpack(&keyed_account.account.data[1..])?;
        let reserve_mints = [state.token_a_mint.clone(), state.token_b_mint.clone()];

        let label = SPL_TOKEN_SWAP_PROGRAMS
            .get(&keyed_account.account.owner)
            .unwrap()
            .clone();
        Ok(Self {
            key: keyed_account.key,
            label,
            state,
            reserve_mints,
            program_id: keyed_account.account.owner,
            reserves: Default::default(),
        })
    }

    fn get_authority(&self) -> Pubkey {
        Pubkey::find_program_address(&[&self.key.to_bytes()], &self.program_id).0
    }

    fn clone(&self) -> SplTokenSwapAmm {
        SplTokenSwapAmm {
            key: self.key,
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
    fn label(&self) -> String {
        self.label.clone()
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

    fn update(&mut self, accounts_map: &HashMap<Pubkey, Vec<u8>>) -> Result<()> {
        let token_a_account = accounts_map.get(&self.state.token_a).unwrap();
        let token_a_token_account = TokenAccount::unpack(token_a_account).unwrap();

        let token_b_account = accounts_map.get(&self.state.token_b).unwrap();
        let token_b_token_account = TokenAccount::unpack(token_b_account).unwrap();

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

        let swap_result = self
            .state
            .swap_curve
            .swap(
                quote_params.in_amount.into(),
                swap_source_amount,
                swap_destination_amount,
                trade_direction,
                &self.state.fees,
            )
            .context("quote failed")?;

        Ok(Quote {
            out_amount: swap_result.destination_amount_swapped.try_into().unwrap(),
            ..Quote::default()
        })
    }

    fn get_swap_leg_and_account_metas(
        &self,
        swap_params: &SwapParams,
    ) -> Result<SwapLegAndAccountMetas> {
        let SwapParams {
            destination_mint,
            in_amount,
            source_mint,
            user_destination_token_account,
            user_source_token_account,
            user_transfer_authority,
            open_order_address,
            quote_mint_to_referrer,
        } = swap_params;

        let (swap_source, swap_destination) = if *source_mint == self.state.token_a_mint {
            (self.state.token_a, self.state.token_b)
        } else {
            (self.state.token_b, self.state.token_a)
        };

        let account_metas = TokenSwap {
            destination: *user_destination_token_account,
            source: *user_source_token_account,
            user_transfer_authority: *user_transfer_authority,
            authority: self.get_authority(),
            token_swap_program: self.program_id,
            token_program: spl_token::ID,
            swap: self.key,
            pool_mint: self.state.pool_mint,
            pool_fee: self.state.pool_fee_account,
            swap_destination,
            swap_source,
        }
        .to_account_metas(None);

        Ok(SwapLegAndAccountMetas {
            swap_leg: SwapLeg::Swap {
                swap: Swap::TokenSwap,
            },
            account_metas,
        })
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}

#[test]
fn test_new_spl_token_swap() {
    use crate::amms::test_harness::AmmTestHarness;
    use crate::constants::USDC_MINT;

    const SOL_USDC_POOL: Pubkey = pubkey!("EGZ7tiLeH62TPV1gL8WwbXGzEPa9zmcpVnnkPKKnrE2U");

    let test_harness = AmmTestHarness::new();
    let keyed_account = test_harness.get_keyed_account(SOL_USDC_POOL).unwrap();
    let mut amm = SplTokenSwapAmm::from_keyed_account(&keyed_account).unwrap();

    test_harness.update_amm(&mut amm);

    let quote = amm
        .quote(&QuoteParams {
            input_mint: native_mint::id(),
            in_amount: 1000000000,
            output_mint: USDC_MINT,
        })
        .unwrap();

    println!("Token mints: {:?}", amm.reserve_mints);
    println!("Quote result: {:?}", quote);
}
