use anchor_lang::{InstructionData, ToAccountMetas};
use anyhow::{anyhow, ensure, Result};
use jupiter_amm_interface::SwapMode;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

use crate::{
    aggregator_version::AggregatorVersion, route::route_plan_with_metadata::JupiterRoutePlanStep,
    swap_transaction::transaction_config::FeeMint,
};

#[derive(Clone)]
pub struct BuildSwapInstructionDataParams {
    pub use_shared_accounts: bool,
    pub use_token_ledger: bool,
    pub program_authority_id: u8,
    pub route_plan: Vec<JupiterRoutePlanStep>,
    pub amount: u64,
    pub quoted_amount: u64,
    pub swap_mode: SwapMode,
    pub slippage_bps: u16,
    pub platform_fee_bps: u16,
}

pub fn build_swap_instruction_data(
    aggregator_version: AggregatorVersion,
    build_swap_instruction_data_params: BuildSwapInstructionDataParams,
) -> Result<Vec<u8>> {
    match aggregator_version {
        AggregatorVersion::V6 => {
            build_jupiter_aggregator_v6_swap_instruction_data(build_swap_instruction_data_params)
        }
    }
}

fn build_jupiter_aggregator_v6_swap_instruction_data(
    BuildSwapInstructionDataParams {
        use_shared_accounts,
        use_token_ledger,
        program_authority_id,
        route_plan,
        amount,
        quoted_amount,
        swap_mode,
        slippage_bps,
        platform_fee_bps,
    }: BuildSwapInstructionDataParams,
) -> Result<Vec<u8>> {
    let platform_fee_bps = platform_fee_bps.try_into()?;
    let route_plan = route_plan
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>>>()?;

    Ok(match swap_mode {
        SwapMode::ExactIn => match (use_shared_accounts, use_token_ledger) {
            (true, true) => {
                jupiter_aggregator_v6::client::args::SharedAccountsRouteWithTokenLedger {
                    id: program_authority_id,
                    route_plan,
                    quoted_out_amount: quoted_amount,
                    slippage_bps,
                    platform_fee_bps,
                }
                .data()
            }
            (true, false) => jupiter_aggregator_v6::client::args::SharedAccountsRoute {
                id: program_authority_id,
                route_plan,
                in_amount: amount,
                quoted_out_amount: quoted_amount,
                slippage_bps,
                platform_fee_bps,
            }
            .data(),
            (false, true) => jupiter_aggregator_v6::client::args::RouteWithTokenLedger {
                route_plan,
                quoted_out_amount: quoted_amount,
                slippage_bps,
                platform_fee_bps,
            }
            .data(),
            (false, false) => jupiter_aggregator_v6::client::args::Route {
                route_plan,
                in_amount: amount,
                quoted_out_amount: quoted_amount,
                slippage_bps,
                platform_fee_bps,
            }
            .data(),
        },
        SwapMode::ExactOut => match use_shared_accounts {
            true => jupiter_aggregator_v6::client::args::SharedAccountsExactOutRoute {
                id: program_authority_id,
                route_plan,
                out_amount: amount,
                quoted_in_amount: quoted_amount,
                slippage_bps,
                platform_fee_bps,
            }
            .data(),
            false => jupiter_aggregator_v6::client::args::ExactOutRoute {
                route_plan,
                out_amount: amount,
                quoted_in_amount: quoted_amount,
                slippage_bps,
                platform_fee_bps,
            }
            .data(),
        },
    })
}

pub struct BuildSwapAccountsParams<'a> {
    pub use_shared_accounts: bool,
    pub swap_mode: SwapMode,
    pub user_transfer_authority: &'a Pubkey,
    pub program_authority: &'a Pubkey,
    pub user_source_token_account: &'a Pubkey,
    pub source_token_account: &'a Pubkey,
    pub source_token_program: &'a Pubkey,
    pub user_destination_token_account: &'a Pubkey,
    pub destination_token_account: &'a Pubkey,
    pub destination_token_program: &'a Pubkey,
    pub input_mint: &'a Pubkey,
    pub output_mint: &'a Pubkey,
    pub token_ledger: Option<Pubkey>,
    pub platform_fee_account: Option<Pubkey>,
    pub optional_destination_token_account: Option<Pubkey>,
    pub token_2022_program: Option<Pubkey>,
    pub user_transfer_authority_as_writable: bool,
    pub fee_mint: FeeMint,
}

pub struct SwapAccounts {
    pub accounts: Vec<AccountMeta>,
}

pub fn build_swap_accounts(
    aggregator_version: AggregatorVersion,
    build_swap_accounts_params: BuildSwapAccountsParams,
) -> Result<SwapAccounts> {
    match aggregator_version {
        AggregatorVersion::V6 => {
            build_jupiter_aggregator_v6_swap_accounts(build_swap_accounts_params)
        }
    }
}

fn build_jupiter_aggregator_v6_swap_accounts(
    BuildSwapAccountsParams {
        use_shared_accounts,
        swap_mode,
        user_transfer_authority,
        program_authority,
        user_source_token_account,
        source_token_account,
        source_token_program,
        user_destination_token_account,
        destination_token_account,
        destination_token_program,
        input_mint,
        output_mint,
        token_ledger,
        platform_fee_account,
        optional_destination_token_account,
        token_2022_program,
        user_transfer_authority_as_writable,
        fee_mint,
    }: BuildSwapAccountsParams,
) -> Result<SwapAccounts> {
    let program = jupiter_aggregator_v6::ID;
    let event_authority = jupiter_aggregator_v6::EVENT_AUTHORITY;

    let mut accounts = match (use_shared_accounts, swap_mode, token_ledger) {
        (true, SwapMode::ExactIn, Some(token_ledger)) => {
            verify_exact_in_fee_mint_compatibility(&fee_mint, source_token_program)?;
            jupiter_aggregator_v6::client::accounts::SharedAccountsRouteWithTokenLedger {
                token_program: spl_token::ID,
                program_authority: *program_authority,
                user_transfer_authority: *user_transfer_authority,
                source_token_account: *user_source_token_account,
                program_source_token_account: *source_token_account,
                program_destination_token_account: *destination_token_account,
                destination_token_account: optional_destination_token_account
                    .unwrap_or(*user_destination_token_account),
                source_mint: *input_mint,
                destination_mint: *output_mint,
                platform_fee_account,
                token_2022_program,
                token_ledger,
                event_authority,
                program,
            }
            .to_account_metas(None)
        }
        (true, SwapMode::ExactIn, None) => {
            verify_exact_in_fee_mint_compatibility(&fee_mint, source_token_program)?;
            jupiter_aggregator_v6::client::accounts::SharedAccountsRoute {
                token_program: spl_token::ID,
                program_authority: *program_authority,
                user_transfer_authority: *user_transfer_authority,
                source_token_account: *user_source_token_account,
                program_source_token_account: *source_token_account,
                program_destination_token_account: *destination_token_account,
                destination_token_account: optional_destination_token_account
                    .unwrap_or(*user_destination_token_account),
                source_mint: *input_mint,
                destination_mint: *output_mint,
                platform_fee_account,
                token_2022_program,
                event_authority,
                program,
            }
            .to_account_metas(None)
        }
        (true, SwapMode::ExactOut, None) => {
            ensure!(
                fee_mint == FeeMint::InputMint,
                "ExactOut only supports input mint fee"
            );
            jupiter_aggregator_v6::client::accounts::SharedAccountsExactOutRoute {
                token_program: spl_token::ID,
                program_authority: *program_authority,
                user_transfer_authority: *user_transfer_authority,
                source_token_account: *user_source_token_account,
                program_source_token_account: *source_token_account,
                program_destination_token_account: *destination_token_account,
                destination_token_account: optional_destination_token_account
                    .unwrap_or(*user_destination_token_account),
                source_mint: *input_mint,
                destination_mint: *output_mint,
                platform_fee_account,
                token_2022_program,
                event_authority,
                program,
            }
            .to_account_metas(None)
        }
        (false, SwapMode::ExactIn, Some(token_ledger)) => {
            verify_exact_in_fee_mint_compatibility(&fee_mint, source_token_program)?;
            let fee_token_program =
                get_fee_token_program(&fee_mint, source_token_program, destination_token_program);

            jupiter_aggregator_v6::client::accounts::RouteWithTokenLedger {
                token_program: *fee_token_program,
                user_transfer_authority: *user_transfer_authority,
                user_source_token_account: *user_source_token_account,
                user_destination_token_account: *user_destination_token_account,
                destination_mint: *output_mint,
                platform_fee_account,
                destination_token_account: optional_destination_token_account,
                token_ledger,
                event_authority,
                program,
            }
            .to_account_metas(None)
        }
        (false, SwapMode::ExactIn, None) => {
            verify_exact_in_fee_mint_compatibility(&fee_mint, source_token_program)?;
            let fee_token_program =
                get_fee_token_program(&fee_mint, source_token_program, destination_token_program);

            jupiter_aggregator_v6::client::accounts::Route {
                token_program: *fee_token_program,
                user_transfer_authority: *user_transfer_authority,
                user_source_token_account: *user_source_token_account,
                user_destination_token_account: *user_destination_token_account,
                destination_mint: *output_mint,
                platform_fee_account,
                destination_token_account: optional_destination_token_account,
                event_authority,
                program,
            }
            .to_account_metas(None)
        }
        (false, SwapMode::ExactOut, None) => {
            ensure!(
                fee_mint == FeeMint::InputMint,
                "ExactOut only supports input mint fee"
            );
            jupiter_aggregator_v6::client::accounts::ExactOutRoute {
                token_program: *destination_token_program,
                user_transfer_authority: *user_transfer_authority,
                user_source_token_account: *user_source_token_account,
                user_destination_token_account: *user_destination_token_account,
                source_mint: *input_mint,
                destination_mint: *output_mint,
                platform_fee_account,
                destination_token_account: optional_destination_token_account,
                token_2022_program,
                event_authority,
                program,
            }
            .to_account_metas(None)
        }
        (_, SwapMode::ExactOut, _) => {
            return Err(anyhow!("SwapMode::ExactOut is only supported with shared accounts and without token ledger"));
        }
    };

    if user_transfer_authority_as_writable {
        for account_meta in accounts.iter_mut() {
            if &account_meta.pubkey == user_transfer_authority {
                account_meta.is_writable = true;
            }
        }
    }

    Ok(SwapAccounts { accounts })
}

fn get_fee_token_program<'a>(
    fee_mint: &FeeMint,
    source_token_program: &'a Pubkey,
    destination_token_program: &'a Pubkey,
) -> &'a Pubkey {
    match fee_mint {
        FeeMint::InputMint => source_token_program,
        FeeMint::OutputMint => destination_token_program,
    }
}

fn verify_exact_in_fee_mint_compatibility(
    fee_mint: &FeeMint,
    source_token_program: &Pubkey,
) -> Result<()> {
    if fee_mint == &FeeMint::InputMint && source_token_program == &spl_token_2022::ID {
        return Err(anyhow!(
            "Cannot use input mint fee with the token 2022 program"
        ));
    }
    Ok(())
}
