use crate::cli::get_path;
use anyhow::Result;
use clap::{ArgGroup, Parser};
use rsban_core::WalletId;
use rsban_node::wallets::{Wallets, WalletsExt};
use rsban_store_lmdb::LmdbEnv;
use std::sync::Arc;

#[derive(Parser)]
#[command(group = ArgGroup::new("input")
    .args(&["data_path", "network"]))]
pub(crate) struct DestroyWalletArgs {
    /// The wallet to be destroyed
    #[arg(long)]
    wallet: String,
    /// Optional password to unlock the wallet
    #[arg(long)]
    password: Option<String>,
    /// Uses sthe supplied path as the data directory
    #[arg(long, group = "input")]
    data_path: Option<String>,
    /// Uses the supplied network (live, test, beta or dev)
    #[arg(long, group = "input")]
    network: Option<String>,
}

impl DestroyWalletArgs {
    pub(crate) async fn destroy_wallet(&self) -> Result<()> {
        let path = get_path(&self.data_path, &self.network).join("wallets.ldb");
        let env = Arc::new(LmdbEnv::new(&path)?);

        let wallets = Arc::new(Wallets::new_null_with_env(
            env,
            tokio::runtime::Handle::current(),
        ));

        let wallet_id = WalletId::decode_hex(&self.wallet)?;
        let password = self.password.clone().unwrap_or_default();
        wallets.ensure_wallet_is_unlocked(wallet_id, &password);
        wallets.destroy(&wallet_id);
        Ok(())
    }
}
