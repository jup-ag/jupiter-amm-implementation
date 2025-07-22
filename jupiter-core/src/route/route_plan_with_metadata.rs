use anyhow::{Context, Result};
use jupiter_amm_interface::Swap;

#[derive(Clone, Debug, PartialEq)]
pub struct JupiterRoutePlanStep {
    pub swap: Swap,
    pub percent: Option<u8>,
    pub bps: Option<u16>,
    pub input_index: u8,
    pub output_index: u8,
}

impl TryInto<jupiter_aggregator_v6::types::RoutePlanStep> for JupiterRoutePlanStep {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<jupiter_aggregator_v6::types::RoutePlanStep> {
        Ok(jupiter_aggregator_v6::types::RoutePlanStep {
            swap: self.swap.into(),
            percent: self.percent.context("Missing percent")?,
            input_index: self.input_index,
            output_index: self.output_index,
        })
    }
}
