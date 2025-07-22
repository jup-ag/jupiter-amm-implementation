use agave_feature_set::FeatureSet;
use ahash::HashSet;
use anyhow::{Context, Error, Result};
use assert_matches::assert_matches;
use glob::glob;
use jupiter_common::client_extension::get_amm_context;
use litesvm::{types::TransactionResult, LiteSVM};
use regex::Regex;
use serde_json::Value;
use solana_account_decoder::{encode_ui_account, UiAccountEncoding};
use solana_rpc_client::{nonblocking, rpc_client::RpcClient};
use solana_sdk::{
    account::Account,
    clock::Clock,
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    inner_instruction::InnerInstruction,
    instruction::Instruction,
    message::SanitizedMessage,
    native_token::LAMPORTS_PER_SOL,
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    sysvar::{
        self,
        instructions::{BorrowedAccountMeta, BorrowedInstruction},
    },
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::extension::StateWithExtensions;
use std::hint::black_box;
use std::str::FromStr;
use std::time::Instant;
use std::{
    collections::HashMap,
    fs::{create_dir, File},
    io::Write,
    path::Path,
};
use std::{
    fs::{remove_dir_all, OpenOptions},
    sync::LazyLock,
};

use crate::{
    active_features::MAINNET_ACTIVE_FEATURES,
    aggregator_version::AggregatorVersion,
    amm::*,
    amms::loader::amm_factory,
    constants,
    route::route::get_token_mints_permutations,
    route::route_plan_with_metadata::JupiterRoutePlanStep,
    solana_rpc_utils::ExtendedSolanaRpcClient,
    swap_transaction::{
        build_swap_instruction::{
            build_swap_accounts, build_swap_instruction_data, BuildSwapAccountsParams,
            BuildSwapInstructionDataParams, SwapAccounts,
        },
        transaction_config::FeeMint,
    },
};
use jupiter_amm_interface::{KeyedUiAccount, SwapMode};
use solana_sdk::pubkey;
use solana_sdk_ids::bpf_loader_upgradeable;

pub static SPL_TOKEN_MINT_TO_IN_AMOUNT: LazyLock<HashMap<Pubkey, u64>> = LazyLock::new(|| {
    HashMap::from([
        (spl_token::native_mint::ID, 25_000_000_000),
        (constants::USDC_MINT, 1_110_000_000),
        (
            pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
            1_110_000_000,
        ),
    ])
});

pub static TOKEN2022_MINT_TO_IN_AMOUNT: LazyLock<HashMap<Pubkey, u64>> =
    LazyLock::new(|| HashMap::from([]));

pub static TOKEN_MINT_TO_IN_AMOUNT: LazyLock<HashMap<Pubkey, u64>> = LazyLock::new(|| {
    let mut m = SPL_TOKEN_MINT_TO_IN_AMOUNT.clone();
    m.extend(TOKEN2022_MINT_TO_IN_AMOUNT.iter());
    m
});

pub static TOKEN_MINT_TO_OUT_AMOUNT: LazyLock<HashMap<Pubkey, u64>> =
    LazyLock::new(|| HashMap::from([]));

pub struct AmmTestSwapParams<'a> {
    pub amm: &'a dyn Amm,
    pub source_mint: &'a Pubkey,
    pub destination_mint: &'a Pubkey,
    pub swap_mode: SwapMode,
    pub tolerance: u64,
    pub use_shared_accounts: bool,
    pub expected_error: Option<&'a anyhow::Error>,
    pub expect_swap: Option<Swap>,
    pub amount: Option<u64>,
}

pub struct AmmTestHarness {
    pub option: Option<String>,
}

pub struct AmmTestHarnessProgramTest {
    lite_svm: LiteSVM,
    program_test_authority: ProgramTestAuthority,
    program_test_user: ProgramTestUser,
    option: Option<String>,
}

fn clone_keypair(keypair: &Keypair) -> Keypair {
    keypair.insecure_clone()
}

impl AmmTestHarnessProgramTest {
    #[allow(clippy::result_large_err)]
    fn process_transaction(
        &mut self,
        instructions: &[Instruction],
        payer: Option<&Pubkey>,
        signers: &[&Keypair],
    ) -> (TransactionResult, Transaction) {
        let latest_blockhash = self.lite_svm.latest_blockhash();

        let transaction =
            Transaction::new_signed_with_payer(instructions, payer, signers, latest_blockhash);
        println!("tx: {transaction:?}");

        (
            self.lite_svm.send_transaction(transaction.clone()),
            transaction,
        )
    }

    /// Some pair cannot quote with the specified amount so we automatically discover an amount which works
    fn find_viable_quote(
        amm: &dyn Amm,
        swap_mode: SwapMode,
        amount: &mut u64,
        source_mint: &Pubkey,
        destination_mint: &Pubkey,
        expects_error: bool,
    ) -> (Option<Quote>, Option<Error>) {
        let mut quote_count: u32 = 0;
        let mut quote_result = None;
        let mut quote_error = None;

        // solution for amm that cant quote certain amount and also could be bug introducing, divide by 2 until can quote
        loop {
            *amount /= 2;
            match amm.quote(&QuoteParams {
                amount: *amount,
                input_mint: *source_mint,
                output_mint: *destination_mint,
                swap_mode,
            }) {
                Ok(quote) => {
                    quote_result = Some(quote);
                    break;
                }
                Err(e) => {
                    println!(
                        "quote error: {e}, source_mint: {source_mint}, destination_mint: {destination_mint}, amount: {amount}"
                    );
                    quote_error = Some(e);
                }
            }

            quote_count += 1;

            if quote_count >= 10 || expects_error {
                println!("quote_error: {quote_error:?}");
                break;
            }
        }

        (quote_result, quote_error)
    }

    pub fn assert_quote_matches_simulated_swap(
        &mut self,
        AmmTestSwapParams {
            amm,
            source_mint,
            destination_mint,
            swap_mode,
            tolerance,
            use_shared_accounts,
            expected_error,
            expect_swap,
            amount,
        }: AmmTestSwapParams<'_>,
    ) {
        let mut amount = amount.unwrap_or_else(|| match swap_mode {
            SwapMode::ExactIn => *TOKEN_MINT_TO_IN_AMOUNT.get(source_mint).unwrap(),
            SwapMode::ExactOut => *TOKEN_MINT_TO_OUT_AMOUNT.get(destination_mint).unwrap(),
        });
        let (quote_result, quote_error) = Self::find_viable_quote(
            amm,
            swap_mode,
            &mut amount,
            source_mint,
            destination_mint,
            expected_error.is_some(),
        );

        let user = self.program_test_user.keypair.pubkey();
        let user_source_token_account = self
            .program_test_user
            .mint_to_ata_with_program_id
            .get(source_mint)
            .unwrap()
            .0;
        let user_destination_token_account = self
            .program_test_user
            .mint_to_ata_with_program_id
            .get(destination_mint)
            .unwrap()
            .0;

        let (program_source_token_account, source_token_program) = *self
            .program_test_authority
            .mint_to_ata_with_program_id
            .get(source_mint)
            .unwrap();
        let (program_destination_token_account, destination_token_program) = *self
            .program_test_authority
            .mint_to_ata_with_program_id
            .get(destination_mint)
            .unwrap();

        let is_input_mint_token2022 = TOKEN2022_MINT_TO_IN_AMOUNT.contains_key(source_mint);
        let is_output_mint_token2022 = TOKEN2022_MINT_TO_IN_AMOUNT.contains_key(destination_mint);
        let source_token_account = if !use_shared_accounts || is_input_mint_token2022 {
            user_source_token_account
        } else {
            program_source_token_account
        };
        let destination_token_account = if !use_shared_accounts || is_output_mint_token2022 {
            user_destination_token_account
        } else {
            program_destination_token_account
        };

        let token_authority = if !use_shared_accounts || is_input_mint_token2022 {
            user
        } else {
            self.program_test_authority.pubkey
        };

        let swap_params = SwapParams {
            swap_mode,
            source_mint: *source_mint,
            destination_mint: *destination_mint,
            source_token_account,
            destination_token_account,
            token_transfer_authority: token_authority,
            quote_mint_to_referrer: None,
            in_amount: amount,
            out_amount: amount,
            jupiter_program_id: &AGGREGATOR_VERSION.program_id(),
            missing_dynamic_accounts_as_default: false,
        };
        let SwapAndAccountMetas {
            swap,
            account_metas,
        } = amm.get_swap_and_account_metas(&swap_params).unwrap();
        if let Some(expect_swap) = expect_swap {
            assert_eq!(swap, expect_swap);
        }

        let SwapAccounts { mut accounts } = build_swap_accounts(
            AGGREGATOR_VERSION,
            BuildSwapAccountsParams {
                use_shared_accounts,
                swap_mode,
                user_transfer_authority: &user,
                program_authority: &self.program_test_authority.pubkey,
                user_source_token_account: &user_source_token_account,
                source_token_account: &source_token_account,
                source_token_program: &source_token_program,
                user_destination_token_account: &user_destination_token_account,
                destination_token_account: &destination_token_account,
                destination_token_program: &destination_token_program,
                input_mint: source_mint,
                output_mint: destination_mint,
                token_ledger: None,
                platform_fee_account: None,
                optional_destination_token_account: None,
                token_2022_program: Some(spl_token_2022::ID),
                user_transfer_authority_as_writable: false,
                fee_mint: match swap_mode {
                    SwapMode::ExactIn => FeeMint::OutputMint,
                    SwapMode::ExactOut => FeeMint::InputMint,
                },
            },
        )
        .unwrap();
        accounts.extend(account_metas);

        let route_plan = vec![JupiterRoutePlanStep {
            swap,
            percent: Some(100),
            bps: Some(10_000),
            input_index: 0,
            output_index: 1,
        }];
        let data = build_swap_instruction_data(
            AGGREGATOR_VERSION,
            BuildSwapInstructionDataParams {
                use_shared_accounts,
                use_token_ledger: false,
                program_authority_id: self.program_test_authority.id,
                route_plan,
                amount,
                quoted_amount: match swap_mode {
                    SwapMode::ExactIn => 0,
                    SwapMode::ExactOut => u64::MAX,
                },
                swap_mode,
                platform_fee_bps: 0,
                slippage_bps: 1,
            },
        )
        .unwrap();
        let swap_ix = Instruction {
            program_id: AGGREGATOR_VERSION.program_id(),
            accounts,
            data,
        };

        let mut ixs: Vec<Instruction> =
            vec![ComputeBudgetInstruction::set_compute_unit_limit(1_400_000)];

        ixs.push(swap_ix);

        let user_keypair = clone_keypair(&self.program_test_user.keypair);
        let user_before = self.lite_svm.get_balance(&user_keypair.pubkey()).unwrap();
        let source_token_account_before = self.get_token_account(&user_source_token_account);
        let destination_token_account_before =
            self.get_token_account(&user_destination_token_account);
        let (transaction_result, transaction) =
            self.process_transaction(&ixs, Some(&user), &[&user_keypair]);
        let user_after = self.lite_svm.get_balance(&user_keypair.pubkey()).unwrap();
        let source_token_account_after = self.get_token_account(&user_source_token_account);
        let destination_token_account_after =
            self.get_token_account(&user_destination_token_account);

        let user_diff = i128::from(user_after)
            .checked_sub(i128::from(user_before))
            .unwrap();
        let source_token_account_diff = source_token_account_before
            .amount
            .checked_sub(source_token_account_after.amount)
            .unwrap();
        let destination_token_account_diff = destination_token_account_after
            .amount
            .checked_sub(destination_token_account_before.amount)
            .unwrap();

        let (transaction_metadata, quote) = if let Some(expected_error) = expected_error {
            let quote_error: Error = quote_error.unwrap();
            match expected_error.downcast_ref::<anchor_lang::error::Error>() {
                Some(error) => {
                    let quote_error = quote_error
                        .downcast_ref::<anchor_lang::error::Error>()
                        .unwrap();
                    assert_eq!(error, quote_error);
                }
                None => {
                    assert_eq!(expected_error.to_string(), quote_error.to_string());
                }
            }
            println!("{transaction_result:?}");
            assert_matches!(transaction_result, Err(_));
            return;
        } else {
            // We don't expect any errors
            println!(
                "source_mint: {source_mint}, destination_mint: {destination_mint}, amount: {amount}"
            );

            match transaction_result {
                Ok(transaction_metadata) => {
                    println!("{transaction_metadata:#?}");
                    (transaction_metadata, quote_result.unwrap())
                }
                Err(failed_transaction_metadata) => {
                    display_inner_instructions(
                        &transaction,
                        &failed_transaction_metadata.meta.inner_instructions,
                    );
                    panic!(
                        "Transaction failed while we did not expect any error: {failed_transaction_metadata:?}"
                    );
                }
            }
        };

        println!("{source_mint} -> {destination_mint}");
        match swap_mode {
            SwapMode::ExactIn => {
                println!(
                    "quote.out_amount: {}, simulation_out_amount: {destination_token_account_diff}, exact_in_amount: {amount}, simulation_in_amount: {source_token_account_diff}, user_diff: {user_diff}",
                    quote.out_amount,
                );
                assert!(
                    (quote.out_amount as i128 - destination_token_account_diff as i128).abs()
                        <= tolerance as i128
                );
            }
            SwapMode::ExactOut => {
                println!(
                    "quote.in_amount: {}, simulation_in_amount: {source_token_account_diff}, exact_out_amount: {amount}, simulation_out_amount: {destination_token_account_diff}",
                    quote.in_amount,
                );
                assert!(
                    (quote.in_amount as i128 - source_token_account_diff as i128).abs()
                        <= tolerance as i128
                );
                assert_eq!(amount, destination_token_account_diff);
            }
        }

        let mut test_name_components = vec![
            amm.label(),
            source_mint.to_string(),
            destination_mint.to_string(),
            amount.to_string(),
        ];
        if let Some(ref option) = self.option {
            test_name_components.push(option.clone());
        }
        let test_name = test_name_components.join("-");

        let compute_units_consumed_by_swap_program = find_compute_units_consumed_by_program_id(
            &amm.program_id(),
            &transaction_metadata.logs,
        );
        let jupiter_cu = transaction_metadata.compute_units_consumed;

        let (overhead_cu, overhead_rate) = match compute_units_consumed_by_swap_program {
            Some(compute_units_consumed_by_swap_program) => {
                let overhead_cu = jupiter_cu.checked_sub(compute_units_consumed_by_swap_program);
                (
                    overhead_cu,
                    overhead_cu.map(|overhead_cu| {
                        overhead_cu as f64 / compute_units_consumed_by_swap_program as f64
                    }),
                )
            }
            None => (None, None),
        };
        println!(
            "Jupiter cu: {jupiter_cu}, swap program cu: {compute_units_consumed_by_swap_program:?}, overhead_cu: {overhead_cu:?}, overhead_rate: {overhead_rate:?}",
        );
        if std::env::var("CAPTURE_CU_USAGE").is_ok() {
            let test_name = format!(
                "{test_name}-{}",
                if use_shared_accounts {
                    "shared_accounts_route"
                } else {
                    "route"
                }
            );
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open("cu_usage.csv")
                .unwrap();

            file.write_all(
                format!(
                    "{test_name},{jupiter_cu:?},{compute_units_consumed_by_swap_program:?},{overhead_cu:?},{overhead_rate:?}\n",
                )
                .as_bytes(),
            )
            .unwrap();
        }

        // Benchmark Quote
        let now = Instant::now();
        let iterations = 100;
        for _ in 0..iterations {
            let quote = amm
                .quote(&QuoteParams {
                    amount,
                    input_mint: *source_mint,
                    output_mint: *destination_mint,
                    swap_mode,
                })
                .unwrap();
            black_box(quote);
        }
        let elapsed = now.elapsed();
        println!(
            "Amm {}, iterations: {iterations}, Quote time per iteration: {} us",
            amm.label(),
            elapsed.as_micros() as f64 / (iterations as f64),
        );

        // insta::assert_debug_snapshot!(test_name, quote);
    }

    /// To be used for exotic test setup
    pub fn assert_out_amount_matches_simulated_swap(
        &mut self,
        swap_instruction: Instruction,
        _input_mint: &Pubkey,
        output_mint: &Pubkey,
        _in_amount: u64,
        out_amount: u64,
        tolerance: Option<u64>,
    ) {
        let user_output_account = self.program_test_user.get_user_ata(output_mint);
        let user_keypair = clone_keypair(&self.program_test_user.keypair);
        let token_account_before = self.get_token_account(&user_output_account);
        let (transaction_result, _) = self.process_transaction(
            &[
                ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
                swap_instruction,
            ],
            Some(&user_keypair.pubkey()),
            &[&user_keypair],
        );
        assert_matches!(transaction_result, Ok(_));
        let token_account_after = self.get_token_account(&user_output_account);

        let simulation_amount = token_account_after
            .amount
            .checked_sub(token_account_before.amount)
            .unwrap();

        assert!(
            (out_amount as i128 - simulation_amount as i128).abs()
                <= tolerance.map(Into::into).unwrap_or(0)
        );
    }

    fn get_token_account(&mut self, address: &Pubkey) -> spl_token_2022::state::Account {
        let token_account = self.lite_svm.get_account(address).unwrap();
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&token_account.data)
            .unwrap()
            .base
    }

    pub fn get_user(&self) -> Pubkey {
        self.program_test_user.keypair.pubkey()
    }
}

/// Find the log from the underlying AMM, which should
fn find_compute_units_consumed_by_program_id(
    program_id: &Pubkey,
    log_messages: &[String],
) -> Option<u64> {
    let regex = Regex::new(&format!(
        r"Program {program_id} consumed (\d+) of (\d+) compute units"
    ))
    .unwrap();

    log_messages.iter().rev().find_map(|log_message| {
        let captures = regex.captures(log_message)?;
        captures.get(1)?.as_str().parse().ok()
    })
}

pub type AccountsSnapshot = AccountMap;

pub struct ProgramTestAuthority {
    id: u8,
    pubkey: Pubkey,
    mint_to_ata_with_program_id: HashMap<Pubkey, (Pubkey, Pubkey)>,
}

pub struct ProgramTestUser {
    keypair: Keypair,
    mint_to_ata_with_program_id: HashMap<Pubkey, (Pubkey, Pubkey)>,
}

impl ProgramTestUser {
    fn get_user_ata(&self, mint: &Pubkey) -> Pubkey {
        self.mint_to_ata_with_program_id.get(mint).unwrap().0
    }
}

/// Update AMM with only the accounts it requested,
/// to avoid relying on side effects
fn update_amm_precise(amm: &mut dyn Amm, account_map: &AccountMap) -> Result<()> {
    let account_map_requested = HashMap::from_iter(
        amm.get_accounts_to_update()
            .into_iter()
            .filter_map(|address| {
                account_map
                    .get(&address)
                    .cloned()
                    .map(|account| (address, account))
            }),
    );
    amm.update(&account_map_requested)
}

pub fn load_accounts_snapshot(directory_name: &str) -> AccountsSnapshot {
    let mut account_map = HashMap::default();
    for entry in glob(&format!("tests/fixtures/accounts/{directory_name}/*.bin")).unwrap() {
        match entry {
            Ok(entry) => {
                let file = File::open(&entry).unwrap();
                let address =
                    Pubkey::from_str(entry.file_stem().unwrap().to_str().unwrap()).unwrap();
                let account = bincode::deserialize_from::<_, Account>(file).unwrap();
                account_map.insert(address, account);
            }
            Err(error) => {
                log::warn!("Glob error: {error:?}");
            }
        }
    }
    account_map
}

pub fn snapshot_directory_name(key: Pubkey, option: Option<String>) -> String {
    let option = match option {
        Some(option) => format!("-{option}"),
        None => "".to_string(),
    };

    format!("{key}{option}")
}

pub struct TestProgram {
    program_id: Pubkey,
    program_bytes: Vec<u8>,
}

pub fn get_program_name(label: &str) -> String {
    label.to_lowercase().replace(' ', "_")
}

const AGGREGATOR_VERSION: AggregatorVersion = AggregatorVersion::V6;

pub fn load_test_programs(amm: &dyn Amm, program_name: Option<String>) -> Vec<TestProgram> {
    let amm_program_name = program_name.unwrap_or(get_program_name(&amm.label()));

    let program_name: &str = if cfg!(feature = "staging") {
        "jupiter_staging"
    } else {
        AGGREGATOR_VERSION.program_name()
    };

    let mut program_id_with_names = vec![
        (AGGREGATOR_VERSION.program_id(), program_name.into()),
        (amm.program_id(), amm_program_name),
    ];

    program_id_with_names.extend(amm.program_dependencies());

    let now = Instant::now();
    let mut test_programs = vec![];
    for (program_id, program_name) in program_id_with_names {
        let program_path = format!("tests/fixtures/{program_name}.so");
        let program_bytes = std::fs::read(&program_path)
            .unwrap_or_else(|err| panic!("Error reading {program_path}: {err}"));
        test_programs.push(TestProgram {
            program_id,
            program_bytes,
        });
    }
    log::debug!("Duration to load programs: {:?}", now.elapsed());
    test_programs
}

pub struct AmmTestAccountsSnapshot {
    amm_key: Pubkey,
    accounts_snapshot: AccountsSnapshot,
    option: Option<String>,
}

impl AmmTestAccountsSnapshot {
    pub fn load(amm_key: Pubkey, option: Option<String>) -> Self {
        let directory_name = snapshot_directory_name(amm_key, option.clone());
        log::debug!("Loading snapshot from {directory_name}");

        let now = Instant::now();
        let accounts_snapshot = load_accounts_snapshot(&directory_name);
        log::debug!("Duration to load account snapshot: {:?}", now.elapsed());
        Self {
            amm_key,
            accounts_snapshot,
            option,
        }
    }

    pub fn get_account(&self, address: &Pubkey) -> Option<Account> {
        self.accounts_snapshot.get(address).cloned()
    }

    pub fn get_clock(&self) -> Result<Clock> {
        let account = self
            .get_account(&sysvar::clock::ID)
            .context("Missing clock account")?;
        bincode::deserialize(&account.data).map_err(Into::into)
    }

    pub fn update_amm_from_snapshot(&self, amm: &mut dyn Amm) {
        update_amm_precise(amm, &self.accounts_snapshot).unwrap();
    }

    pub fn get_keyed_account(&self) -> Result<KeyedAccount> {
        let account = self.accounts_snapshot.get(&self.amm_key).cloned().unwrap();

        let directory_name = snapshot_directory_name(self.amm_key, self.option.clone());
        let params_file_path = format!("tests/fixtures/accounts/{directory_name}/params.json");
        let mut params: Option<Value> = None;

        // check if params file exists
        if Path::new(&params_file_path).exists() {
            let file = File::open(params_file_path).unwrap();
            params = serde_json::from_reader(file).unwrap();
        }

        Ok(KeyedAccount {
            key: self.amm_key,
            account,
            params,
        })
    }

    /// Snapshot necessary accounts to perform a swap so that we can reload it later on for reproducible tests
    /// Saved as <amm-id><option>/<address>.json, with the amm id to avoid collision between AMMs
    pub fn snapshot_amm_accounts(
        client: RpcClient,
        amm: &dyn Amm,
        params: Option<Value>,
        option: Option<String>,
        allow_executable: bool,
        force: bool,
    ) -> Result<()> {
        let placeholder = Pubkey::new_unique();
        let mut addresses_for_snapshot = HashSet::default();
        // What is the best way here? Since the mint list keeps on growing
        for (source_mint, destination_mint) in get_token_mints_permutations(amm, true) {
            let swap_leg_and_account_metas = amm.get_swap_and_account_metas(&SwapParams {
                // We can use just ExactIn here to cover all accounts needed for both ExactIn and ExactOut.
                // Only tricky one is Whirlpool where it uses 2 different set of accounts for ExactIn and ExactOut. But using ExactIn here will cover both.
                swap_mode: SwapMode::ExactIn,
                source_mint,
                destination_mint,
                source_token_account: placeholder,
                destination_token_account: placeholder,
                token_transfer_authority: placeholder,
                quote_mint_to_referrer: None,
                in_amount: *TOKEN_MINT_TO_IN_AMOUNT
                    .get(&source_mint)
                    .unwrap_or_else(|| panic!("No in amount for mint: {source_mint}")),
                out_amount: *TOKEN_MINT_TO_IN_AMOUNT
                    .get(&destination_mint)
                    .unwrap_or_else(|| panic!("No out amount for mint: {destination_mint}")),
                jupiter_program_id: &placeholder,
                missing_dynamic_accounts_as_default: false,
            })?;

            addresses_for_snapshot.extend(
                swap_leg_and_account_metas
                    .account_metas
                    .iter()
                    .map(|account_meta| account_meta.pubkey),
            );
        }
        addresses_for_snapshot.extend(amm.get_accounts_to_update());
        addresses_for_snapshot.extend(amm.get_reserve_mints());
        addresses_for_snapshot.remove(&placeholder);

        // Some sysvar that are not necessarily referenced through instruction accounts
        addresses_for_snapshot.insert(sysvar::clock::ID);
        addresses_for_snapshot.insert(sysvar::last_restart_slot::ID);

        let snapshot_path_string = format!(
            "tests/fixtures/accounts/{}",
            snapshot_directory_name(amm.key(), option)
        );
        let snapshot_path = Path::new(&snapshot_path_string);
        if force {
            remove_dir_all(snapshot_path)?;
        }
        create_dir(snapshot_path)?;

        if params.is_some() {
            let mut f = File::create(snapshot_path.join("params.json")).unwrap();
            f.write_all(serde_json::to_value(params).unwrap().to_string().as_bytes())
                .unwrap();
        }

        let addresses = addresses_for_snapshot.into_iter().collect::<Vec<_>>();
        let keyed_accounts = addresses
            .clone()
            .into_iter()
            .zip(
                client
                    .get_multiple_accounts_chunked(&addresses, None)
                    .unwrap(),
            )
            .collect::<Vec<_>>();

        for (address, account) in keyed_accounts {
            if let Some(account) = account {
                if !allow_executable && account.executable {
                    // Avoid snapshotting programs as it breaks program test
                    continue;
                }
                let account_file_path = snapshot_path.join(format!("{address}.bin"));
                let f = File::create(account_file_path).unwrap();
                bincode::serialize_into::<_, Account>(f, &account).unwrap();
            }
        }

        Ok(())
    }
}

impl AmmTestHarness {
    pub fn load_program_test(
        amm_test_accounts_snapshot: &AmmTestAccountsSnapshot,
        test_programs: &[TestProgram],
        amm: &mut dyn Amm,
        mints: Option<&[Pubkey]>,
        before_test_setup: Option<&mut impl FnMut(&dyn Amm, &mut AccountMap)>,
    ) -> AmmTestHarnessProgramTest {
        let mut feature_set = FeatureSet::default();
        for feature in MAINNET_ACTIVE_FEATURES {
            feature_set.activate(&Pubkey::from_str(feature).unwrap(), 0);
        }

        let mut lite_svm = LiteSVM::new().with_feature_set(feature_set);

        let now = Instant::now();
        for test_program in test_programs {
            lite_svm.add_program(test_program.program_id, &test_program.program_bytes);
        }
        log::debug!(
            "Duration to add {} programs: {:?}",
            test_programs.len(),
            now.elapsed()
        );

        let now = Instant::now();
        let mut accounts_snapshot = amm_test_accounts_snapshot.accounts_snapshot.clone();

        // Modify the original snapshot before it gets loaded in the context or in the Amm
        if let Some(before_test_setup) = before_test_setup {
            before_test_setup(amm, &mut accounts_snapshot);
        }

        // This partition is necessary for litesvm, since it requires the program data account to be loaded before its front account
        let (program_data_accounts, other_accounts): (Vec<_>, Vec<_>) = accounts_snapshot
            .clone()
            .into_iter()
            .partition(|(_, account)| {
                account.owner.eq(&bpf_loader_upgradeable::ID) && !account.executable
            });

        for address_with_accounts in [program_data_accounts, other_accounts] {
            for (address, account) in address_with_accounts.iter() {
                lite_svm
                    .set_account(*address, account.clone())
                    .map_err(|error| format!("Error setting account for {address}: {error:?}"))
                    .unwrap();
            }
        }
        log::debug!("Duration to add accounts: {:?}", now.elapsed());

        for _ in 0..3 {
            update_amm_precise(amm, &accounts_snapshot).unwrap();
        }

        let reserve_mints = amm.get_reserve_mints();
        let mints = mints.unwrap_or(&reserve_mints);

        let now = Instant::now();
        let program_test_authority = AmmTestHarness::setup_authority(&mut lite_svm, mints);
        let program_test_user = AmmTestHarness::setup_user(&mut lite_svm, mints);
        log::debug!("Duration to setup accounts: {:?}", now.elapsed());

        AmmTestHarnessProgramTest {
            lite_svm,
            program_test_authority,
            program_test_user,
            option: amm_test_accounts_snapshot.option.clone(),
        }
    }

    /// Setup user and mutate token accounts with funded ATAs
    fn setup_user(lite_svm: &mut LiteSVM, mints: &[Pubkey]) -> ProgramTestUser {
        let keypair = Keypair::new();
        let user = keypair.pubkey();

        let mint_to_ata_with_program_id = setup_token_accounts(&user, lite_svm, mints, true);

        ProgramTestUser {
            keypair,
            mint_to_ata_with_program_id,
        }
    }

    /// Setup progrma authority and mutate token accounts with funded ATAs
    fn setup_authority(lite_svm: &mut LiteSVM, mints: &[Pubkey]) -> ProgramTestAuthority {
        let authority_id = 0;
        let program_authority = *AGGREGATOR_VERSION
            .get_program_authority(authority_id)
            .unwrap();

        let mint_to_ata_with_program_id =
            setup_token_accounts(&program_authority, lite_svm, mints, false);

        ProgramTestAuthority {
            id: authority_id,
            pubkey: program_authority,
            mint_to_ata_with_program_id,
        }
    }
}

fn setup_token_accounts(
    wallet: &Pubkey,
    lite_svm: &mut LiteSVM,
    reserve_mints: &[Pubkey],
    with_bootstrap_amounts: bool,
) -> HashMap<Pubkey, (Pubkey, Pubkey)> {
    use solana_system_interface;
    use spl_associated_token_account::instruction::create_associated_token_account;
    use spl_token_2022::extension::StateWithExtensionsMut;

    let payer_keypair = Keypair::new();
    lite_svm
        .airdrop(&payer_keypair.pubkey(), 100 * LAMPORTS_PER_SOL)
        .unwrap();

    let mut setup_ixs = vec![solana_system_interface::instruction::transfer(
        &payer_keypair.pubkey(),
        wallet,
        1_000_000_000,
    )];
    let mut mint_to_ata_with_program_id = HashMap::new();
    let mut ata_to_set_amount = HashMap::new();

    // We only snapshot mints for token2022, as a result we can only naturally create ATAs for token2022
    for reserve_mint in reserve_mints {
        let (in_amount, token_program_id) = SPL_TOKEN_MINT_TO_IN_AMOUNT
            .get(reserve_mint)
            .map(|in_amount| (*in_amount, spl_token::ID))
            .or_else(|| {
                TOKEN2022_MINT_TO_IN_AMOUNT
                    .get(reserve_mint)
                    .map(|in_amount| (*in_amount, spl_token_2022::ID))
            })
            .unwrap_or_else(|| panic!("Token mint to be defined: {reserve_mint}"));

        let ata = if token_program_id == spl_token_2022::ID {
            let ata = get_associated_token_address_with_program_id(
                wallet,
                reserve_mint,
                &spl_token_2022::ID,
            );
            setup_ixs.push(create_associated_token_account(
                &payer_keypair.pubkey(),
                wallet,
                reserve_mint,
                &spl_token_2022::ID,
            ));
            ata_to_set_amount.insert(ata, in_amount * 100);
            ata
        } else {
            let (ata, token_account) =
                create_ata_account(wallet, reserve_mint, in_amount * 100, spl_token::ID);
            lite_svm.set_account(ata, token_account).unwrap();
            ata
        };

        mint_to_ata_with_program_id.insert(*reserve_mint, (ata, spl_token::ID));
    }

    let latest_blockhash = lite_svm.latest_blockhash();
    lite_svm
        .send_transaction(Transaction::new_signed_with_payer(
            &setup_ixs,
            Some(&payer_keypair.pubkey()),
            &[&payer_keypair],
            latest_blockhash,
        ))
        .unwrap();

    if with_bootstrap_amounts {
        for (ata, set_amount) in ata_to_set_amount {
            let mut account = lite_svm.get_account(&ata).unwrap();
            println!("{}: {:?}", account.owner, account.data);

            let mut token_account =
                StateWithExtensionsMut::<spl_token_2022::state::Account>::unpack(&mut account.data)
                    .unwrap();
            token_account.base.amount = set_amount;
            token_account.pack_base();

            lite_svm.set_account(ata, account).unwrap();
        }
    }

    mint_to_ata_with_program_id
}

fn create_ata_account(
    user: &Pubkey,
    mint: &Pubkey,
    amount: u64,
    token_program_id: Pubkey,
) -> (Pubkey, Account) {
    let ata = get_associated_token_address_with_program_id(user, mint, &token_program_id);

    let mut is_native = COption::None;
    let mut lamports = 10_000_000; // More than enough
    if mint == &spl_token::native_mint::ID {
        let rent = 2_039_280;
        is_native = COption::Some(rent);
        lamports = amount + rent;
    };
    let token_account = spl_token::state::Account {
        mint: *mint,
        owner: *user,
        amount,
        delegate: COption::None,
        state: spl_token::state::AccountState::Initialized,
        is_native,
        delegated_amount: 0,
        close_authority: COption::None,
    };
    let mut data = [0; spl_token::state::Account::LEN].to_vec();
    spl_token::state::Account::pack(token_account, &mut data).unwrap();

    (
        ata,
        Account {
            lamports,
            data,
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        },
    )
}

pub async fn take_snapshot(
    rpc_url: String,
    amm_id: String,
    option: Option<String>,
    allow_executable: bool,
    force: bool,
    params: Option<String>,
) -> Result<()> {
    let amm_key = Pubkey::from_str(&amm_id).unwrap();

    let keyed_ui_account = {
        let client = RpcClient::new(&rpc_url);

        client
            .get_account_with_commitment(&amm_key, CommitmentConfig::finalized())
            .unwrap()
            .value
            .map(|account| {
                let ui_account =
                    encode_ui_account(&amm_key, &account, UiAccountEncoding::Base64, None, None);
                KeyedUiAccount {
                    pubkey: amm_id,
                    ui_account,
                    params: None,
                }
            })
    };

    let client = nonblocking::rpc_client::RpcClient::new(rpc_url.clone());
    let amm_context = get_amm_context(&client).await?;
    let (mut amm, params) = match keyed_ui_account {
        Some(mut keyed_ui_account) => {
            println!("keyed_ui_account {keyed_ui_account:?}");
            if let Some(params) = params {
                keyed_ui_account.params = Some(serde_json::from_str(&params).unwrap());
            }
            let keyed_account: KeyedAccount = keyed_ui_account.try_into()?;
            (
                amm_factory(&keyed_account, &amm_context)?.with_context(|| {
                    format!("AMM with address {} not supported", keyed_account.key)
                })?,
                keyed_account.params,
            )
        }
        None => {
            panic!();
        }
    };

    let client = RpcClient::new(rpc_url);
    let amm: &mut (dyn Amm + Send + Sync) = amm.as_mut();
    for _ in 0..3 {
        fetch_and_update_amm(&client, amm);
    }

    AmmTestAccountsSnapshot::snapshot_amm_accounts(
        client,
        amm,
        params,
        option,
        allow_executable,
        force,
    )?;

    Ok(())
}

pub async fn take_account_snapshot(rpc_url: String, address: String) -> Result<()> {
    let address = Pubkey::from_str(&address)?;

    let account = RpcClient::new(rpc_url).get_account(&address)?;
    let account_file_path = format!("{address}.bin");
    let f = File::create(account_file_path).unwrap();

    bincode::serialize_into::<_, Account>(f, &account).unwrap();

    Ok(())
}

pub fn fetch_and_update_amm(client: &RpcClient, amm: &mut dyn Amm) {
    let accounts_to_update = amm.get_accounts_to_update();
    let account_map = client
        .get_multiple_accounts_chunked(&accounts_to_update, None)
        .unwrap()
        .into_iter()
        .zip(accounts_to_update)
        .fold(HashMap::default(), |mut m, (account, address)| {
            if let Some(account) = account {
                m.insert(address, account);
            }
            m
        });

    amm.update(&account_map).unwrap();
}

pub struct BorrowedInnerInstruction<'a> {
    /// Compiled instruction
    pub instruction: BorrowedInstruction<'a>,
    /// Invocation stack height of the instruction,
    pub stack_height: u8,
}

pub fn decompile_inner_instructions<'a>(
    sanitized_message: &'a SanitizedMessage,
    inner_instructions: &'a [Vec<InnerInstruction>],
) -> Vec<Vec<BorrowedInnerInstruction<'a>>> {
    let account_keys = sanitized_message.account_keys();
    inner_instructions
        .iter()
        .map(|inner_instructions| {
            inner_instructions
                .iter()
                .map(
                    |InnerInstruction {
                         instruction,
                         stack_height,
                     }| {
                        let accounts = instruction
                            .accounts
                            .iter()
                            .map(|account_index| {
                                let account_index = *account_index as usize;
                                BorrowedAccountMeta {
                                    is_signer: sanitized_message.is_signer(account_index),
                                    is_writable: sanitized_message.is_writable(account_index),
                                    pubkey: account_keys.get(account_index).unwrap(),
                                }
                            })
                            .collect();

                        let instruction = BorrowedInstruction {
                            accounts,
                            data: &instruction.data,
                            program_id: account_keys
                                .get(usize::from(instruction.program_id_index))
                                .unwrap(),
                        };
                        BorrowedInnerInstruction {
                            instruction,
                            stack_height: *stack_height,
                        }
                    },
                )
                .collect()
        })
        .collect()
}

fn display_inner_instructions(
    transaction: &Transaction,
    inner_instructions: &[Vec<InnerInstruction>],
) {
    let sanitized_message = SanitizedMessage::try_from_legacy_message(
        transaction.message.clone(),
        &std::collections::HashSet::new(),
    )
    .unwrap();
    for borrowed_inner_instructions in
        decompile_inner_instructions(&sanitized_message, inner_instructions)
    {
        for BorrowedInnerInstruction {
            instruction:
                BorrowedInstruction {
                    program_id,
                    accounts,
                    data,
                },
            stack_height,
        } in borrowed_inner_instructions
        {
            println!(
                "stack_height: {stack_height}, program_id: {program_id}, accounts: {:#?}, data: {data:?}",
                accounts
                    .iter()
                    .map(
                        |BorrowedAccountMeta {
                             pubkey,
                             is_signer,
                             is_writable,
                         }| format!("{pubkey},{is_signer},{is_writable}")
                    )
                    .collect::<Vec<_>>()
            );
        }
    }
}
