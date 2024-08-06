use std::collections::HashMap;

use anyhow::Error;
use jupiter_amm_interface::{AmmContext, ClockRef, KeyedAccount, SwapMode};
use jupiter_core::{
    amm::Amm,
    amms::{spl_token_swap_amm::SplTokenSwapAmm, test_harness::AmmTestHarness},
    route::get_token_mints_permutations,
    test_harness::AmmTestSwapParams,
};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{account::Account, pubkey};

/// Loads AMM from snapshot and tests quoting
async fn test_quoting_for_amm_key<T: Amm + 'static>(
    amm_key: Pubkey,
    swap_mode: SwapMode,
    use_shared_accounts: bool,
    tolerance: u64,
    option: Option<String>,
    before_test_setup: Option<impl FnMut(&dyn Amm, &mut HashMap<Pubkey, Account>)>,
    expect_error: Option<Error>,
    restricted_mint_permutations: Option<Vec<(Pubkey, Pubkey)>>,
) where
    T: Amm,
{
    let test_harness = AmmTestHarness::new_with_rpc_url("".into(), amm_key, option);
    let keyed_account: KeyedAccount = test_harness.get_keyed_account_from_snapshot().unwrap();

    let amm_context = AmmContext {
        clock_ref: ClockRef::from(test_harness.get_clock()),
    };
    let mut amm = T::from_keyed_account(&keyed_account, &amm_context).unwrap();
    // if amm.requires_update_for_reserve_mints() {
    //     test_harness.update_amm_from_snapshot(&mut amm).unwrap();
    // }
    test_quoting_with_amm(
        &test_harness,
        Box::new(amm),
        tolerance,
        use_shared_accounts,
        swap_mode,
        before_test_setup,
        expect_error,
        restricted_mint_permutations,
    )
    .await;
}

macro_rules! test_exact_in_amms {
    ($(($amm_key:expr, $amm_struct:ty, $tolerance:expr),)*) => {
        test_exact_in_amms!(
            $(($amm_key, $amm_struct, $tolerance, "default"),)*
        );
    };
    ($(($amm_key:expr, $amm_struct:ty, $tolerance:expr, $option:expr),)*) => {
        $(
            paste::item! {
                #[tokio::test]
                async fn [<test_quote_ $amm_key:lower _ $option:lower>] () {
                    let option = match $option {
                        "default" => None,
                        _ => Some($option.to_string()),
                    };
                    let before_test_setup: Option<fn(&dyn Amm, &mut HashMap<Pubkey, Account>)> = None;
                    test_quoting_for_amm_key::<$amm_struct>($amm_key, SwapMode::ExactIn, false, $tolerance, option, before_test_setup, None, None).await
                }
                #[tokio::test]
                async fn [<test_quote_ $amm_key:lower _ $option:lower _ with_shared_accounts>] () {
                    let option = match $option {
                        "default" => None,
                        _ => Some($option.to_string()),
                    };
                    let before_test_setup: Option<fn(&dyn Amm, &mut HashMap<Pubkey, Account>)> = None;
                    test_quoting_for_amm_key::<$amm_struct>($amm_key, SwapMode::ExactIn, true, $tolerance, option, before_test_setup, None, None).await
                }
            }
        )*
    };
}

macro_rules! test_exact_out_amms {
    ($(($amm_key:expr, $amm_struct:ty, $tolerance:expr),)*) => {
        test_exact_out_amms!(
            $(($amm_key, $amm_struct, $tolerance, "exact-out"),)*
        );
    };
    ($(($amm_key:expr, $amm_struct:ty, $tolerance:expr, $option:expr),)*) => {
        $(
            paste::item! {
                #[tokio::test]
                async fn [<test_quote_ $amm_key:lower _ $option:lower>] () {
                    let option = Some($option.to_string());
                    let before_test_setup: Option<fn(&dyn Amm, &mut HashMap<Pubkey, Account>)> = None;
                    test_quoting_for_amm_key::<$amm_struct>($amm_key, SwapMode::ExactOut, true, $tolerance, option, before_test_setup, None, None).await
                }
                #[tokio::test]
                async fn [<test_quote_ $amm_key:lower _ $option:lower _ without_shared_accounts>] () {
                    let option = Some($option.to_string());
                    let before_test_setup: Option<fn(&dyn Amm, &mut HashMap<Pubkey, Account>)> = None;
                    test_quoting_for_amm_key::<$amm_struct>($amm_key, SwapMode::ExactOut, false, $tolerance, option, before_test_setup, None, None).await
                }
            }
        )*
    };
}

const ORCA_V2_SOL_USDC_POOL: Pubkey = pubkey!("EGZ7tiLeH62TPV1gL8WwbXGzEPa9zmcpVnnkPKKnrE2U");
const ORCA_V2_USDC_USDT_POOL: Pubkey = pubkey!("F13xvvx45jVGd84ynK3c8T89UejQVxjCLtmHfPmAXAHP");

// You can run a single test by doing: `cargo test test_quote_<lower_case_constant>_<default | option_name> -- --nocapture`

test_exact_in_amms! {
    (ORCA_V2_SOL_USDC_POOL, SplTokenSwapAmm, 0),
    (ORCA_V2_USDC_USDT_POOL, SplTokenSwapAmm, 0),
}

async fn test_quoting_with_amm(
    test_harness: &AmmTestHarness,
    mut amm: Box<dyn Amm>,
    tolerance: u64,
    use_shared_accounts: bool,
    swap_mode: SwapMode,
    mut before_test_setup: Option<impl FnMut(&dyn Amm, &mut HashMap<Pubkey, Account>)>,
    expect_error: Option<anyhow::Error>,
    restricted_mint_permutations: Option<Vec<(Pubkey, Pubkey)>>,
) {
    let amm = amm.as_mut();
    let reserve_token_mint_permutations =
        restricted_mint_permutations.unwrap_or(get_token_mints_permutations(amm));
    let mut one_test_passed = false;
    for (source_mint, destination_mint) in reserve_token_mint_permutations {
        let mut test_harness_program_test = test_harness
            .load_program_test(amm, before_test_setup.as_mut())
            .await;

        test_harness_program_test
            .assert_quote_matches_simulated_swap(AmmTestSwapParams {
                amm,
                source_mint: &source_mint,
                destination_mint: &destination_mint,
                swap_mode,
                tolerance,
                use_shared_accounts,
                expected_error: expect_error.as_ref(),
            })
            .await;

        one_test_passed = true;
    }
    assert!(one_test_passed);
}
