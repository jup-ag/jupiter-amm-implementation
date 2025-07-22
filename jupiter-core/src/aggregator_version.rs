use std::str::FromStr;

use jupiter_aggregator_common::{compute_program_authorities, AUTHORITY_COUNT_USIZE};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

const JUPITER_V6_PROGRAM_AUTHORITIES: [Pubkey; AUTHORITY_COUNT_USIZE] =
    compute_program_authorities(jupiter_aggregator_v6::ID);

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum AggregatorVersion {
    #[default]
    V6,
}

impl FromStr for AggregatorVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "V6" => Ok(Self::V6),
            _ => Err(anyhow::anyhow!("{} is not a valid aggregator version", s)),
        }
    }
}

impl AggregatorVersion {
    pub fn program_id(&self) -> Pubkey {
        match self {
            AggregatorVersion::V6 => jupiter_aggregator_v6::ID,
        }
    }

    pub fn authorities(&self) -> [Pubkey; AUTHORITY_COUNT_USIZE] {
        match self {
            AggregatorVersion::V6 => JUPITER_V6_PROGRAM_AUTHORITIES,
        }
    }

    pub fn get_program_authority(&self, id: u8) -> Option<&'static Pubkey> {
        let authorities = match self {
            AggregatorVersion::V6 => &JUPITER_V6_PROGRAM_AUTHORITIES,
        };
        authorities.get(usize::from(id))
    }

    pub fn event_authority(&self) -> Pubkey {
        match self {
            AggregatorVersion::V6 => jupiter_aggregator_v6::EVENT_AUTHORITY,
        }
    }

    pub fn program_name(&self) -> &'static str {
        match self {
            AggregatorVersion::V6 => "jupiter_v6",
        }
    }
}
