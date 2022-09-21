use std::{path::Path, sync::Arc, time::Duration};

use crate::{
    datastore::{Transaction, VersionStore, WriteTransaction, STORE_VERSION_MINIMUM},
    logger_mt::Logger,
    TxnTrackingConfig,
};

use super::{
    EnvOptions, LmdbAccountStore, LmdbBlockStore, LmdbConfirmationHeightStore, LmdbEnv,
    LmdbFinalVoteStore, LmdbFrontierStore, LmdbOnlineWeightStore, LmdbPeerStore, LmdbPendingStore,
    LmdbPrunedStore, LmdbUncheckedStore, LmdbVersionStore,
};

#[derive(PartialEq, Eq)]
pub enum Vacuuming {
    Needed,
    NotNeeded,
}

pub struct LmdbStore {
    pub env: Arc<LmdbEnv>,
    pub block_store: Arc<LmdbBlockStore>,
    pub frontier_store: Arc<LmdbFrontierStore>,
    pub account_store: Arc<LmdbAccountStore>,
    pub pending_store: Arc<LmdbPendingStore>,
    pub online_weight_store: Arc<LmdbOnlineWeightStore>,
    pub pruned_store: Arc<LmdbPrunedStore>,
    pub peer_store: Arc<LmdbPeerStore>,
    pub confirmation_height_store: Arc<LmdbConfirmationHeightStore>,
    pub final_vote_store: Arc<LmdbFinalVoteStore>,
    pub unchecked_store: Arc<LmdbUncheckedStore>,
    pub version_store: Arc<LmdbVersionStore>,
    logger: Arc<dyn Logger>,
}

impl LmdbStore {
    pub fn new(
        path: &Path,
        options: &EnvOptions,
        tracking_cfg: TxnTrackingConfig,
        block_processor_batch_max_time: Duration,
        logger: Arc<dyn Logger>,
    ) -> anyhow::Result<Self> {
        let env = Arc::new(LmdbEnv::with_tracking(
            path,
            options,
            tracking_cfg,
            block_processor_batch_max_time,
            logger.clone(),
        )?);

        Ok(Self {
            env: env.clone(),
            block_store: Arc::new(LmdbBlockStore::new(env.clone())),
            frontier_store: Arc::new(LmdbFrontierStore::new(env.clone())),
            account_store: Arc::new(LmdbAccountStore::new(env.clone())),
            pending_store: Arc::new(LmdbPendingStore::new(env.clone())),
            online_weight_store: Arc::new(LmdbOnlineWeightStore::new(env.clone())),
            pruned_store: Arc::new(LmdbPrunedStore::new(env.clone())),
            peer_store: Arc::new(LmdbPeerStore::new(env.clone())),
            confirmation_height_store: Arc::new(LmdbConfirmationHeightStore::new(env.clone())),
            final_vote_store: Arc::new(LmdbFinalVoteStore::new(env.clone())),
            unchecked_store: Arc::new(LmdbUncheckedStore::new(env.clone())),
            version_store: Arc::new(LmdbVersionStore::new(env.clone())),
            logger,
        })
    }

    pub fn open_databases(&self, txn: &dyn Transaction, flags: u32) -> anyhow::Result<()> {
        self.block_store.open_db(txn, flags)?;
        self.frontier_store.open_db(txn, flags)?;
        self.account_store.open_db(txn, flags)?;
        self.pending_store.open_db(txn, flags)?;
        self.online_weight_store.open_db(txn, flags)?;
        self.pruned_store.open_db(txn, flags)?;
        self.peer_store.open_db(txn, flags)?;
        self.confirmation_height_store.open_db(txn, flags)?;
        self.final_vote_store.open_db(txn, flags)?;
        self.unchecked_store.open_db(txn, flags)?;
        self.version_store.open_db(txn, flags)
    }

    pub fn do_upgrades(&self, txn: &dyn WriteTransaction) -> anyhow::Result<Vacuuming> {
        let version = self.version_store.get(txn.as_transaction());
        match version {
            1..=20 => {
                self.logger.always_log(&format!("The version of the ledger ({}) is lower than the minimum ({}) which is supported for upgrades. Either upgrade to a v23 node first or delete the ledger.", version, STORE_VERSION_MINIMUM));
                Err(anyhow!("version too low"))
            }
            21 => {
                // most recent version
                Ok(Vacuuming::NotNeeded)
            }
            _ => {
                self.logger.always_log(&format!(
                    "The version of the ledger ({}) is too high for this node",
                    version
                ));
                Err(anyhow!("version too high"))
            }
        }
    }
}
