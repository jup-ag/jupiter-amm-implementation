use jupiter_amm_interface::{AccountMap, AmmContext, ClockRef, Swap, SwapMode};
use jupiter_core::{
    amm::Amm,
    amms::{spl_token_swap_amm::SplTokenSwapAmm, test_harness::AmmTestHarness},
    route::route::get_token_mints_permutations,
    test_harness::{load_test_programs, AmmTestAccountsSnapshot, AmmTestSwapParams, TestProgram},
};
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

#[derive(Default)]
struct TestAmmSettings {
    tolerance: u64,
    restricted_mint_permutations: Option<Vec<(Pubkey, Pubkey)>>,
    expect_error: Option<anyhow::Error>,
    expect_swaps: Option<Vec<Swap>>,
    program_name: Option<String>,
    amounts: Option<Vec<u64>>,
}
impl TestAmmSettings {
    fn new_with_tolerance(tolerance: u64) -> Self {
        Self {
            tolerance,
            ..Default::default()
        }
    }
    fn new_with_mint_permutations(mint_permutations: Vec<(Pubkey, Pubkey)>) -> Self {
        Self {
            restricted_mint_permutations: Some(mint_permutations),
            ..Default::default()
        }
    }
    fn new_with_expected_error(expected_error: anyhow::Error) -> Self {
        Self {
            expect_error: Some(expected_error),
            ..Default::default()
        }
    }
    fn new_with_expected_swaps(swaps: Vec<Swap>) -> Self {
        Self {
            expect_swaps: Some(swaps),
            ..Default::default()
        }
    }
}

/// Loads AMM from snapshot and tests quoting
#[allow(clippy::too_many_arguments)]
fn test_quoting_for_amm_key<T: Amm + 'static>(
    amm_key: Pubkey,
    swap_mode: SwapMode,
    use_shared_accounts: bool,
    test_amm_settings: TestAmmSettings,
    option: Option<String>,
    before_test_setup: Option<impl FnMut(&dyn Amm, &mut AccountMap)>,
) {
    let TestAmmSettings {
        tolerance,
        restricted_mint_permutations,
        expect_error,
        expect_swaps,
        program_name,
        amounts,
    } = test_amm_settings;

    let amm_test_accounts_snapshot = AmmTestAccountsSnapshot::load(amm_key, option.clone());
    let keyed_account = amm_test_accounts_snapshot.get_keyed_account().unwrap();
    let amm_context = AmmContext {
        clock_ref: ClockRef::from(amm_test_accounts_snapshot.get_clock().unwrap()),
    };

    // All the test setup and can go over the stack size
    let mut amm = T::from_keyed_account(&keyed_account, &amm_context).unwrap();
    let test_programs = load_test_programs(&amm, program_name.clone());
    if amm.requires_update_for_reserve_mints() {
        amm_test_accounts_snapshot.update_amm_from_snapshot(&mut amm);
    }
    test_quoting_with_amm(
        &amm_test_accounts_snapshot,
        &test_programs,
        Box::new(amm),
        tolerance,
        use_shared_accounts,
        swap_mode,
        before_test_setup,
        expect_error,
        expect_swaps.as_deref(),
        restricted_mint_permutations,
        amounts.as_deref(),
    )
}

macro_rules! test_exact_in_amms {
    ($(($amm_key:expr, $amm_struct:ty, $test_amm_settings:expr),)*) => {
        test_exact_in_amms!(
            $(($amm_key, $amm_struct, $test_amm_settings, "default"),)*
        );
    };
    ($(($amm_key:expr, $amm_struct:ty, $test_amm_settings:expr, $option:expr),)*) => {
        $(
            paste::item! {
                #[tokio::test]
                async fn [<test_quote_ $amm_key:lower _ $option:lower>] () {
                    let option = match $option {
                        "default" => None,
                        _ => Some($option.to_string()),
                    };
                    let before_test_setup: Option<fn(&dyn Amm, &mut AccountMap)> = None;
                    test_quoting_for_amm_key::<$amm_struct>($amm_key, SwapMode::ExactIn, false, $test_amm_settings.unwrap_or_default(), option, before_test_setup)
                }
                #[tokio::test]
                async fn [<test_quote_ $amm_key:lower _ $option:lower _ with_shared_accounts>] () {
                    let option = match $option {
                        "default" => None,
                        _ => Some($option.to_string()),
                    };
                    let before_test_setup: Option<fn(&dyn Amm, &mut AccountMap)> = None;
                    test_quoting_for_amm_key::<$amm_struct>($amm_key, SwapMode::ExactIn, true, $test_amm_settings.unwrap_or_default(), option, before_test_setup)
                }
            }
        )*
    };
}

macro_rules! test_exact_out_amms {
    ($(($amm_key:expr, $amm_struct:ty, $test_amm_settings:expr),)*) => {
        test_exact_out_amms!(
            $(($amm_key, $amm_struct, $test_amm_settings, "exact-out"),)*
        );
    };
    ($(($amm_key:expr, $amm_struct:ty, $test_amm_settings:expr, $option:expr),)*) => {
        $(
            paste::item! {
                #[tokio::test]
                async fn [<test_quote_ $amm_key:lower _ $option:lower>] () {
                    let option = Some($option.to_string());
                    let before_test_setup: Option<fn(&dyn Amm, &mut AccountMap)> = None;
                    test_quoting_for_amm_key::<$amm_struct>($amm_key, SwapMode::ExactOut, true, $test_amm_settings.unwrap_or_default(), option, before_test_setup)
                }
                #[tokio::test]
                async fn [<test_quote_ $amm_key:lower _ $option:lower _ without_shared_accounts>] () {
                    let option = Some($option.to_string());
                    let before_test_setup: Option<fn(&dyn Amm, &mut AccountMap)> = None;
                    test_quoting_for_amm_key::<$amm_struct>($amm_key, SwapMode::ExactOut, false, $test_amm_settings.unwrap_or_default(), option, before_test_setup)
                }
            }
        )*
    };
}

const ORCA_V2_SOL_USDC_POOL: Pubkey = pubkey!("EGZ7tiLeH62TPV1gL8WwbXGzEPa9zmcpVnnkPKKnrE2U");
const ORCA_V2_USDC_USDT_POOL: Pubkey = pubkey!("F13xvvx45jVGd84ynK3c8T89UejQVxjCLtmHfPmAXAHP");

// You can run a single test by doing: `cargo test test_quote_<lower_case_constant>_<default | option_name> -- --nocapture`

test_exact_in_amms! {
    (ORCA_V2_SOL_USDC_POOL, SplTokenSwapAmm, None),
    (ORCA_V2_USDC_USDT_POOL, SplTokenSwapAmm, None),
}

#[allow(clippy::too_many_arguments)]
fn test_quoting_with_amm(
    amm_test_accounts_snapshot: &AmmTestAccountsSnapshot,
    test_programs: &[TestProgram],
    mut amm: Box<dyn Amm>,
    tolerance: u64,
    use_shared_accounts: bool,
    swap_mode: SwapMode,
    mut before_test_setup: Option<impl FnMut(&dyn Amm, &mut AccountMap)>,
    expect_error: Option<anyhow::Error>,
    expect_swaps: Option<&[Swap]>,
    restricted_mint_permutations: Option<Vec<(Pubkey, Pubkey)>>,
    amounts: Option<&[u64]>,
) {
    let amm = amm.as_mut();

    let reserve_token_mint_permutations =
        restricted_mint_permutations.unwrap_or(get_token_mints_permutations(amm, true));
    let mut one_test_passed = false;

    let mut amounts_iterator = amounts.map(|amounts| amounts.iter());
    let mut expect_swap_iterator = expect_swaps.map(|expect_swaps| expect_swaps.iter());

    for (source_mint, destination_mint) in reserve_token_mint_permutations {
        let mut test_harness_program_test = AmmTestHarness::load_program_test(
            amm_test_accounts_snapshot,
            test_programs,
            amm,
            Some(&[source_mint, destination_mint]),
            before_test_setup.as_mut(),
        );

        let amount = amounts_iterator
            .as_mut()
            .map(|it| it.next().copied().expect("Missing amount"));
        let expect_swap = expect_swap_iterator
            .as_mut()
            .map(|it| it.next().cloned().expect("Missing swap"));
        test_harness_program_test.assert_quote_matches_simulated_swap(AmmTestSwapParams {
            amm,
            source_mint: &source_mint,
            destination_mint: &destination_mint,
            swap_mode,
            tolerance,
            use_shared_accounts,
            expected_error: expect_error.as_ref(),
            expect_swap,
            amount,
        });

        one_test_passed = true;
    }
    assert!(one_test_passed);
}
