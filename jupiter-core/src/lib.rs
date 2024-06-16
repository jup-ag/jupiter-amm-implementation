pub mod amms;
mod math;

pub mod build_swap_transaction;
pub mod config;
pub mod constants;
pub mod curve;
pub mod route;
pub mod utils;

pub use amms::amm;
pub use amms::test_harness;

use anchor_lang::prelude::*;

declare_id!("GoatAFSqACoMvJqvgW7aFACFkkArv69ezTJhS8xdEr5H");
