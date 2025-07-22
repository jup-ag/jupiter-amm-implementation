use anchor_lang::declare_program;
use jupiter_common::find_program_address::find_program_address;

#[cfg(feature = "staging")]
declare_program!(
    jupiter_aggregator,
    address = "JUPSjgjMFjU4453KMgxhqVmzep6W352bQpE4RsNqXAx"
);
#[cfg(not(feature = "staging"))]
declare_program!(jupiter_aggregator_v6);

pub use crate::jupiter_aggregator_v6::{ID, ID_CONST, client, types};
use rand::seq::SliceRandom;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

pub const EVENT_AUTHORITY: Pubkey = find_program_address(&[b"__event_authority"], &crate::ID).0;

const TOKEN_LEDGERS: [Pubkey; 4] = [
    pubkey!("HtncvpUBGhSrs48KtC58ntJcTDw53sn78Lpq71zVwiez"),
    pubkey!("HxTk98CmBcxmtkrBWqRszYxrnDpqAsbitQBc2QjVBG3j"),
    pubkey!("CnUPHtfUVw3D2s4FB8H6QBuLwoes8YxauVgDtFybm7rz"),
    pubkey!("FhLPkpFmszHtSyyayj7KsXNZeBTqfQbUPmvgWAyJHBXh"),
];

pub fn find_jupiter_token_ledger() -> Option<Pubkey> {
    let mut rng = rand::thread_rng();
    TOKEN_LEDGERS.choose(&mut rng).copied()
}

impl From<jupiter_amm_interface::Swap> for types::Swap {
    fn from(value: jupiter_amm_interface::Swap) -> Self {
        match value {
            jupiter_amm_interface::Swap::Saber => types::Swap::Saber,
            jupiter_amm_interface::Swap::SaberAddDecimalsDeposit => {
                types::Swap::SaberAddDecimalsDeposit
            }
            jupiter_amm_interface::Swap::SaberAddDecimalsWithdraw => {
                types::Swap::SaberAddDecimalsWithdraw
            }
            jupiter_amm_interface::Swap::TokenSwap => types::Swap::TokenSwap,
            jupiter_amm_interface::Swap::Raydium => types::Swap::Raydium,
            jupiter_amm_interface::Swap::Crema { a_to_b } => types::Swap::Crema { a_to_b },
            jupiter_amm_interface::Swap::Mercurial => types::Swap::Mercurial,
            jupiter_amm_interface::Swap::Aldrin { side } => {
                types::Swap::Aldrin { side: side.into() }
            }
            jupiter_amm_interface::Swap::AldrinV2 { side } => {
                types::Swap::AldrinV2 { side: side.into() }
            }
            jupiter_amm_interface::Swap::Whirlpool { a_to_b } => types::Swap::Whirlpool { a_to_b },
            jupiter_amm_interface::Swap::Invariant { x_to_y } => types::Swap::Invariant { x_to_y },
            jupiter_amm_interface::Swap::Meteora => types::Swap::Meteora,
            jupiter_amm_interface::Swap::MarcoPolo { x_to_y } => types::Swap::MarcoPolo { x_to_y },
            jupiter_amm_interface::Swap::LifinityV2 => types::Swap::LifinityV2,
            jupiter_amm_interface::Swap::RaydiumClmm => types::Swap::RaydiumClmm,
            jupiter_amm_interface::Swap::Phoenix { side } => {
                types::Swap::Phoenix { side: side.into() }
            }
            jupiter_amm_interface::Swap::TokenSwapV2 => types::Swap::TokenSwapV2,
            jupiter_amm_interface::Swap::HeliumTreasuryManagementRedeemV0 => {
                types::Swap::HeliumTreasuryManagementRedeemV0
            }
            jupiter_amm_interface::Swap::StakeDexStakeWrappedSol => {
                types::Swap::StakeDexStakeWrappedSol
            }
            jupiter_amm_interface::Swap::MeteoraDlmm => types::Swap::MeteoraDlmm,
            jupiter_amm_interface::Swap::OpenBookV2 { side } => {
                types::Swap::OpenBookV2 { side: side.into() }
            }
            jupiter_amm_interface::Swap::RaydiumClmmV2 => types::Swap::RaydiumClmmV2,
            jupiter_amm_interface::Swap::StakeDexPrefundWithdrawStakeAndDepositStake {
                bridge_stake_seed,
            } => types::Swap::StakeDexPrefundWithdrawStakeAndDepositStake { bridge_stake_seed },
            jupiter_amm_interface::Swap::SanctumS {
                src_lst_value_calc_accs,
                dst_lst_value_calc_accs,
                src_lst_index,
                dst_lst_index,
            } => types::Swap::SanctumS {
                src_lst_value_calc_accs,
                dst_lst_value_calc_accs,
                src_lst_index,
                dst_lst_index,
            },
            jupiter_amm_interface::Swap::SanctumSAddLiquidity {
                lst_value_calc_accs,
                lst_index,
            } => types::Swap::SanctumSAddLiquidity {
                lst_value_calc_accs,
                lst_index,
            },
            jupiter_amm_interface::Swap::SanctumSRemoveLiquidity {
                lst_value_calc_accs,
                lst_index,
            } => types::Swap::SanctumSRemoveLiquidity {
                lst_value_calc_accs,
                lst_index,
            },
            jupiter_amm_interface::Swap::RaydiumCP => types::Swap::RaydiumCP,
            jupiter_amm_interface::Swap::WhirlpoolSwapV2 {
                a_to_b,
                remaining_accounts_info,
            } => types::Swap::WhirlpoolSwapV2 {
                a_to_b,
                remaining_accounts_info: remaining_accounts_info.map(Into::into),
            },
            jupiter_amm_interface::Swap::OneIntro => types::Swap::OneIntro,
            jupiter_amm_interface::Swap::PumpWrappedBuy => types::Swap::PumpWrappedBuy,
            jupiter_amm_interface::Swap::PumpWrappedSell => types::Swap::PumpWrappedSell,
            jupiter_amm_interface::Swap::PerpsV2 => types::Swap::PerpsV2,
            jupiter_amm_interface::Swap::PerpsV2AddLiquidity => types::Swap::PerpsV2AddLiquidity,
            jupiter_amm_interface::Swap::PerpsV2RemoveLiquidity => {
                types::Swap::PerpsV2RemoveLiquidity
            }
            jupiter_amm_interface::Swap::MoonshotWrappedBuy => types::Swap::MoonshotWrappedBuy,
            jupiter_amm_interface::Swap::MoonshotWrappedSell => types::Swap::MoonshotWrappedSell,
            jupiter_amm_interface::Swap::StabbleStableSwap => types::Swap::StabbleStableSwap,
            jupiter_amm_interface::Swap::StabbleWeightedSwap => types::Swap::StabbleWeightedSwap,
            jupiter_amm_interface::Swap::Obric { x_to_y } => types::Swap::Obric { x_to_y },
            jupiter_amm_interface::Swap::SolFi { is_quote_to_base } => {
                types::Swap::SolFi { is_quote_to_base }
            }
            jupiter_amm_interface::Swap::SolayerDelegateNoInit => {
                types::Swap::SolayerDelegateNoInit
            }
            jupiter_amm_interface::Swap::SolayerUndelegateNoInit => {
                types::Swap::SolayerUndelegateNoInit
            }
            jupiter_amm_interface::Swap::ZeroFi => types::Swap::ZeroFi,
            jupiter_amm_interface::Swap::StakeDexWithdrawWrappedSol => {
                types::Swap::StakeDexWithdrawWrappedSol
            }
            jupiter_amm_interface::Swap::VirtualsBuy => types::Swap::VirtualsBuy,
            jupiter_amm_interface::Swap::VirtualsSell => types::Swap::VirtualsSell,
            jupiter_amm_interface::Swap::Perena {
                in_index,
                out_index,
            } => types::Swap::Perena {
                in_index,
                out_index,
            },
            jupiter_amm_interface::Swap::PumpSwapBuy => types::Swap::PumpSwapBuy,
            jupiter_amm_interface::Swap::PumpSwapSell => types::Swap::PumpSwapSell,
            jupiter_amm_interface::Swap::Gamma => types::Swap::Gamma,
            jupiter_amm_interface::Swap::MeteoraDlmmSwapV2 {
                remaining_accounts_info,
            } => types::Swap::MeteoraDlmmSwapV2 {
                remaining_accounts_info: remaining_accounts_info.into(),
            },
            jupiter_amm_interface::Swap::Woofi => types::Swap::Woofi,
            jupiter_amm_interface::Swap::MeteoraDammV2 => types::Swap::MeteoraDammV2,
            jupiter_amm_interface::Swap::StabbleStableSwapV2 => types::Swap::StabbleStableSwapV2,
            jupiter_amm_interface::Swap::StabbleWeightedSwapV2 => {
                types::Swap::StabbleWeightedSwapV2
            }
            jupiter_amm_interface::Swap::RaydiumLaunchlabBuy { share_fee_rate } => {
                types::Swap::RaydiumLaunchlabBuy { share_fee_rate }
            }
            jupiter_amm_interface::Swap::RaydiumLaunchlabSell { share_fee_rate } => {
                types::Swap::RaydiumLaunchlabSell { share_fee_rate }
            }
            jupiter_amm_interface::Swap::BoopdotfunWrappedBuy => types::Swap::BoopdotfunWrappedBuy,
            jupiter_amm_interface::Swap::BoopdotfunWrappedSell => {
                types::Swap::BoopdotfunWrappedSell
            }
            jupiter_amm_interface::Swap::Plasma { side } => {
                types::Swap::Plasma { side: side.into() }
            }
            jupiter_amm_interface::Swap::GoonFi {
                is_bid,
                blacklist_bump,
            } => types::Swap::GoonFi {
                is_bid,
                blacklist_bump,
            },
            jupiter_amm_interface::Swap::HumidiFi {
                swap_id,
                is_base_to_quote,
            } => types::Swap::HumidiFi {
                swap_id,
                is_base_to_quote,
            },
            jupiter_amm_interface::Swap::MeteoraDynamicBondingCurveSwapWithRemainingAccounts => {
                types::Swap::MeteoraDynamicBondingCurveSwapWithRemainingAccounts
            }
            jupiter_amm_interface::Swap::TesseraV { side } => {
                types::Swap::TesseraV { side: side.into() }
            }
        }
    }
}

impl From<jupiter_amm_interface::Side> for types::Side {
    fn from(value: jupiter_amm_interface::Side) -> Self {
        match value {
            jupiter_amm_interface::Side::Bid => Self::Bid,
            jupiter_amm_interface::Side::Ask => Self::Ask,
        }
    }
}

impl From<jupiter_amm_interface::RemainingAccountsInfo> for types::RemainingAccountsInfo {
    fn from(
        jupiter_amm_interface::RemainingAccountsInfo{ slices }: jupiter_amm_interface::RemainingAccountsInfo,
    ) -> Self {
        Self {
            slices: slices
                .into_iter()
                .map(|s| types::RemainingAccountsSlice {
                    accounts_type: match s.accounts_type {
                        jupiter_amm_interface::AccountsType::TransferHookA => {
                            types::AccountsType::TransferHookA
                        }
                        jupiter_amm_interface::AccountsType::TransferHookB => {
                            types::AccountsType::TransferHookB
                        }
                    },
                    length: s.length,
                })
                .collect(),
        }
    }
}
