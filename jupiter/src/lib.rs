anchor_gen::generate_cpi_crate!("idl.json");

#[cfg(feature = "staging")]
anchor_lang::declare_id!("JUPSjgjMFjU4453KMgxhqVmzep6W352bQpE4RsNqXAx");
#[cfg(not(feature = "staging"))]
anchor_lang::declare_id!("JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB");

// Temporarily redefined it until solution is found
pub mod jupiter_override {
    use super::Side;
    use anchor_lang::prelude::*;
    use anchor_lang::{AnchorSerialize, InstructionData};
    use std::io::Write;

    #[derive(AnchorSerialize)]
    pub enum Swap {
        Saber,
        SaberAddDecimalsDeposit,
        SaberAddDecimalsWithdraw,
        TokenSwap,
        Sencha,
        Step,
        Cropper,
        Raydium,
        Crema,
        Lifinity,
        Mercurial,
        Cykura,
        Serum { side: Side },
        MarinadeDeposit,
        MarinadeUnstake,
        Aldrin { side: Side },
        AldrinV2 { side: Side },
        Whirlpool { a_to_b: bool },
    }

    #[derive(AnchorSerialize)]
    pub struct SplitLeg {
        pub percent: u8,
        pub swap_leg: SwapLeg,
    }

    pub enum SwapLeg {
        Chain { swap_legs: Vec<SwapLeg> },
        Split { split_legs: Vec<SplitLeg> },
        Swap { swap: Swap },
    }

    impl AnchorSerialize for SwapLeg {
        #[inline]
        fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
            match self {
                SwapLeg::Chain { swap_legs } => {
                    0u8.serialize(writer)?;
                    swap_legs.serialize(writer)
                }
                SwapLeg::Split { split_legs } => {
                    1u8.serialize(writer)?;
                    split_legs.serialize(writer)
                }
                SwapLeg::Swap { swap } => {
                    2u8.serialize(writer)?;
                    swap.serialize(writer)
                }
            }
        }
    }

    #[derive(AnchorSerialize)]
    pub struct Route {
        pub swap_leg: SwapLeg,
        pub in_amount: u64,
        pub quoted_out_amount: u64,
        pub slippage_bps: u16,
        pub platform_fee_bps: u8,
    }

    impl InstructionData for Route {
        fn data(&self) -> Vec<u8> {
            // SHA256 "global:route"
            let mut d = vec![229, 23, 203, 151, 122, 227, 173, 42];
            d.append(&mut self.try_to_vec().unwrap());
            d
        }
    }
}
