mod bootstrap_attempt;
mod bootstrap_attempts;
mod bootstrap_client;
mod bootstrap_connections;
mod bootstrap_initiator;
mod bootstrap_lazy;
mod bootstrap_message_visitor;
mod bootstrap_message_visitor_factory;
mod bulk_pull_account_server;
mod bulk_pull_client;
mod bulk_pull_server;
mod bulk_push_server;
mod channel_tcp_wrapper;
mod frontier_req_server;
mod pulls_cache;

use std::sync::Arc;

pub use bootstrap_attempt::*;
pub use bootstrap_connections::*;
pub use bootstrap_initiator::*;
pub use frontier_req_server::FrontierReqServer;

pub use bootstrap_client::{
    BootstrapClient, BootstrapClientObserver, BootstrapClientObserverWeakPtr,
};

pub use bootstrap_attempts::BootstrapAttempts;
pub use bootstrap_lazy::*;
pub use bootstrap_message_visitor::BootstrapMessageVisitorImpl;
pub use bootstrap_message_visitor_factory::BootstrapMessageVisitorFactory;
pub use bulk_pull_account_server::BulkPullAccountServer;
pub use bulk_pull_client::*;
pub use bulk_pull_server::BulkPullServer;
pub use bulk_push_server::BulkPushServer;
pub use channel_tcp_wrapper::ChannelTcpWrapper;
pub use pulls_cache::{PullInfo, PullsCache};
use rsnano_core::{Account, BlockEnum};

pub mod bootstrap_limits {
    pub const PULL_COUNT_PER_CHECK: u64 = 8 * 1024;
    pub const BOOTSTRAP_MINIMUM_ELAPSED_SECONDS_BLOCKRATE: f64 = 0.02;
}

#[derive(Clone, Copy, FromPrimitive, Debug, PartialEq, Eq)]
pub enum BootstrapMode {
    Legacy,
    Lazy,
    WalletLazy,
    Ascending,
}

pub enum BootstrapStrategy {
    Lazy(BootstrapAttemptLazy),
    Other(BootstrapAttempt),
}

impl BootstrapStrategy {
    pub fn attempt(&self) -> &BootstrapAttempt {
        match self {
            BootstrapStrategy::Other(i) => i,
            BootstrapStrategy::Lazy(i) => &i.attempt,
        }
    }

    pub fn process_block(
        &self,
        block: Arc<BlockEnum>,
        known_account: &Account,
        pull_blocks_processed: u64,
        max_blocks: u32,
        block_expected: bool,
        retry_limit: u32,
    ) -> bool {
        match self {
            BootstrapStrategy::Other(i) => i.process_block(
                block,
                known_account,
                pull_blocks_processed,
                max_blocks,
                block_expected,
                retry_limit,
            ),
            BootstrapStrategy::Lazy(i) => i.process_block(
                block,
                known_account,
                pull_blocks_processed,
                max_blocks,
                block_expected,
                retry_limit,
            ),
        }
    }
}
