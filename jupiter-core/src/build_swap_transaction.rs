use itertools::Itertools;

use anchor_lang::{prelude::*, InstructionData};
use anyhow::{anyhow, Result};
use solana_sdk::{
    address_lookup_table_account::AddressLookupTableAccount, instruction::Instruction,
};

use jupiter::{
    self, find_event_authority, jupiter_override::RoutePlanStep as JupiterRoutePlanStep,
};
use jupiter_amm_interface::SwapMode;

/// All necessary parts to build a `VersionedTransaction`
#[derive(Clone)]
pub struct SwapInstructions {
    pub compute_budget_instructions: Vec<Instruction>,
    pub setup_instructions: Vec<Instruction>,
    pub token_ledger_instruction: Option<Instruction>,
    /// Instruction performing the action of swapping
    pub swap_instruction: Instruction,
    pub cleanup_instruction: Option<Instruction>,
    pub address_lookup_table_addresses: Vec<Pubkey>,
}

impl From<SwapInstructions> for Vec<Instruction> {
    fn from(
        SwapInstructions {
            compute_budget_instructions,
            setup_instructions,
            token_ledger_instruction: _,
            swap_instruction,
            cleanup_instruction,
            address_lookup_table_addresses: _,
        }: SwapInstructions,
    ) -> Vec<Instruction> {
        // We don't use `token_ledger_instruction` to build the transaction. `token_ledger_instruction` is
        // only available in instructions mode.
        compute_budget_instructions
            .into_iter()
            .chain(setup_instructions)
            .chain([swap_instruction])
            .chain(cleanup_instruction)
            .collect_vec()
    }
}

pub struct BuildSwapAccountsParams<'a> {
    pub use_shared_accounts: bool,
    pub swap_mode: SwapMode,
    pub user: &'a Pubkey,
    pub program_authority: &'a Pubkey,
    pub user_source_token_account: &'a Pubkey,
    pub source_token_account: &'a Pubkey,
    pub user_destination_token_account: &'a Pubkey,
    pub destination_token_account: &'a Pubkey,
    pub destination_token_program: &'a Pubkey,
    pub input_mint: &'a Pubkey,
    pub output_mint: &'a Pubkey,
    pub token_ledger: Option<Pubkey>,
    pub platform_fee_account: Option<Pubkey>,
    pub optional_destination_token_account: Option<Pubkey>,
    pub token2022_program: Option<Pubkey>,
}

pub fn build_swap_accounts(
    BuildSwapAccountsParams {
        use_shared_accounts,
        swap_mode,
        user,
        program_authority,
        user_source_token_account,
        source_token_account,
        user_destination_token_account,
        destination_token_account,
        destination_token_program,
        input_mint,
        output_mint,
        token_ledger,
        platform_fee_account,
        optional_destination_token_account,
        token2022_program,
    }: BuildSwapAccountsParams,
) -> Result<Vec<AccountMeta>> {
    let program = jupiter::ID;
    let event_authority = find_event_authority();
    Ok(match (use_shared_accounts, swap_mode, token_ledger) {
        (true, SwapMode::ExactIn, Some(token_ledger)) => {
            jupiter::accounts::SharedAccountsRouteWithTokenLedger {
                token_program: spl_token::ID,
                program_authority: *program_authority,
                user_transfer_authority: *user,
                source_token_account: *user_source_token_account,
                program_source_token_account: *source_token_account,
                program_destination_token_account: *destination_token_account,
                destination_token_account: optional_destination_token_account
                    .unwrap_or(*user_destination_token_account),
                source_mint: *input_mint,
                destination_mint: *output_mint,
                platform_fee_account,
                token2022_program,
                token_ledger,
                event_authority,
                program,
            }
            .to_account_metas(None)
        }
        (true, SwapMode::ExactIn, None) => jupiter::accounts::SharedAccountsRoute {
            token_program: spl_token::ID,
            program_authority: *program_authority,
            user_transfer_authority: *user,
            source_token_account: *user_source_token_account,
            program_source_token_account: *source_token_account,
            program_destination_token_account: *destination_token_account,
            destination_token_account: optional_destination_token_account
                .unwrap_or(*user_destination_token_account),
            source_mint: *input_mint,
            destination_mint: *output_mint,
            platform_fee_account,
            token2022_program,
            event_authority,
            program,
        }
        .to_account_metas(None),
        (true, SwapMode::ExactOut, None) => jupiter::accounts::SharedAccountsExactOutRoute {
            token_program: spl_token::ID,
            program_authority: *program_authority,
            user_transfer_authority: *user,
            source_token_account: *user_source_token_account,
            program_source_token_account: *source_token_account,
            program_destination_token_account: *destination_token_account,
            destination_token_account: optional_destination_token_account
                .unwrap_or(*user_destination_token_account),
            source_mint: *input_mint,
            destination_mint: *output_mint,
            platform_fee_account,
            token2022_program,
            event_authority,
            program,
        }
        .to_account_metas(None),
        (false, SwapMode::ExactIn, Some(token_ledger)) => jupiter::accounts::RouteWithTokenLedger {
            token_program: *destination_token_program, // This depends on the user_destination_token_account
            user_transfer_authority: *user,
            user_source_token_account: *user_source_token_account,
            user_destination_token_account: *user_destination_token_account,
            destination_mint: *output_mint,
            platform_fee_account,
            destination_token_account: optional_destination_token_account,
            token_ledger,
            event_authority,
            program,
        }
        .to_account_metas(None),
        (false, SwapMode::ExactIn, None) => jupiter::accounts::Route {
            token_program: *destination_token_program, // This depends on the user_destination_token_account
            user_transfer_authority: *user,
            user_source_token_account: *user_source_token_account,
            user_destination_token_account: *user_destination_token_account,
            destination_mint: *output_mint,
            platform_fee_account,
            destination_token_account: optional_destination_token_account,
            event_authority,
            program,
        }
        .to_account_metas(None),
        (false, SwapMode::ExactOut, None) => jupiter::accounts::ExactOutRoute {
            token_program: spl_token::ID,
            user_transfer_authority: *user,
            user_source_token_account: *user_source_token_account,
            user_destination_token_account: *user_destination_token_account,
            source_mint: *input_mint,
            destination_mint: *output_mint,
            platform_fee_account,
            destination_token_account: optional_destination_token_account,
            token2022_program,
            event_authority,
            program,
        }
        .to_account_metas(None),
        (_, SwapMode::ExactOut, _) => {
            return Err(anyhow!("SwapMode::ExactOut is only supported with shared accounts and without token ledger"));
        }
    })
}

pub struct BuildSwapInstructionDataParams {
    pub use_shared_accounts: bool,
    pub use_token_ledger: bool,
    pub program_authority_id: u8,
    pub route_plan: Vec<JupiterRoutePlanStep>,
    pub amount: u64,
    pub quoted_amount: u64,
    pub swap_mode: SwapMode,
    pub slippage_bps: u16,
    pub platform_fee_bps: u8,
}

pub fn build_swap_instruction_data(
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
    Ok(match swap_mode {
        SwapMode::ExactIn => match (use_shared_accounts, use_token_ledger) {
            (true, true) => jupiter::jupiter_override::SharedAccountsRouteWithTokenLedger {
                id: program_authority_id,
                route_plan,
                quoted_out_amount: quoted_amount,
                slippage_bps,
                platform_fee_bps,
            }
            .data(),
            (true, false) => jupiter::jupiter_override::SharedAccountsRoute {
                id: program_authority_id,
                route_plan,
                in_amount: amount,
                quoted_out_amount: quoted_amount,
                slippage_bps,
                platform_fee_bps,
            }
            .data(),
            (false, true) => jupiter::jupiter_override::RouteWithTokenLedger {
                route_plan,
                quoted_out_amount: quoted_amount,
                slippage_bps,
                platform_fee_bps,
            }
            .data(),
            (false, false) => jupiter::jupiter_override::Route {
                route_plan,
                in_amount: amount,
                quoted_out_amount: quoted_amount,
                slippage_bps,
                platform_fee_bps,
            }
            .data(),
        },
        SwapMode::ExactOut => match use_shared_accounts {
            true => jupiter::jupiter_override::SharedAccountsExactOutRoute {
                id: program_authority_id,
                route_plan,
                out_amount: amount,
                quoted_in_amount: quoted_amount,
                slippage_bps,
                platform_fee_bps,
            }
            .data(),
            false => jupiter::jupiter_override::ExactOutRoute {
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

pub struct SwapInstructionsWithMetadata {
    pub swap_instructions: SwapInstructions,
    pub address_lookup_table_accounts: Vec<AddressLookupTableAccount>,
    pub prioritization_fee_lamports: u64,
}
