use clap::Parser;
use jupiter_core::test_harness::take_snapshot;

#[derive(Parser, Debug)]
pub struct ConfigOverride {
    #[clap(long)]
    pub rpc_url: String,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Snapshot a single amm for test harness testing
    SnapshotAmm {
        #[clap(long)]
        amm_id: String,
        /// Expand an extra option to the snapshot directory (e.g. <amm-id><option>)
        #[clap(long)]
        option: Option<String>,
        /// Overwrite the output snapshot if it exists
        #[clap(short, long)]
        force: bool,
    },
}

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(flatten)]
    pub config_override: ConfigOverride,
    #[clap(subcommand)]
    pub command: Command,
}

#[tokio::main]
async fn main() {
    let Cli {
        config_override,
        command,
    } = Cli::parse();

    match command {
        Command::SnapshotAmm {
            amm_id,
            option,
            force,
        } => take_snapshot(config_override.rpc_url, amm_id, option, force)
            .await
            .unwrap(),
    }
}
