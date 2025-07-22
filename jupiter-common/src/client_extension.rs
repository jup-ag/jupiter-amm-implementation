use anyhow::{Context, Result};
use jupiter_amm_interface::{AmmContext, ClockRef};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{clock::Clock, sysvar};

async fn get_clock(rpc_client: &RpcClient) -> Result<Clock> {
    let clock_data = rpc_client
        .get_account_with_commitment(&sysvar::clock::ID, rpc_client.commitment())
        .await?
        .value
        .context("Failed to get clock account")?;

    let clock = bincode::deserialize::<Clock>(&clock_data.data)
        .context("Failed to deserialize sysvar::clock::ID")?;

    Ok(clock)
}

pub async fn get_amm_context(rpc_client: &RpcClient) -> Result<AmmContext> {
    get_clock(rpc_client).await.map(|clock| AmmContext {
        clock_ref: ClockRef::from(clock),
    })
}

pub async fn update_amm_context(
    rpc_client: &RpcClient,
    amm_context: &mut AmmContext,
) -> anyhow::Result<()> {
    let clock = get_clock(rpc_client).await?;
    amm_context.clock_ref.update(clock);
    Ok(())
}
