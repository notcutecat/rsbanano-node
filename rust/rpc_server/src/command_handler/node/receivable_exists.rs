use crate::command_handler::RpcCommandHandler;
use rsnano_core::BlockHash;
use rsnano_node::Node;
use rsnano_rpc_messages::{ExistsDto, ReceivableExistsArgs};
use std::sync::Arc;

impl RpcCommandHandler {
    pub(crate) fn receivable_exists(&self, args: ReceivableExistsArgs) -> ExistsDto {
        let include_active = args.include_active.unwrap_or(false);
        let include_only_confirmed = args.include_only_confirmed.unwrap_or(true);
        let txn = self.node.ledger.read_txn();

        let exists = if let Some(block) = self.node.ledger.get_block(&txn, &args.hash) {
            if block.is_send() {
                let pending_key =
                    rsnano_core::PendingKey::new(block.destination().unwrap(), args.hash);
                let pending_exists = self
                    .node
                    .ledger
                    .any()
                    .get_pending(&txn, &pending_key)
                    .is_some();

                if pending_exists {
                    block_confirmed(
                        self.node.clone(),
                        &args.hash,
                        include_active,
                        include_only_confirmed,
                    )
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        ExistsDto::new(exists)
    }
}

fn block_confirmed(
    node: Arc<Node>,
    hash: &BlockHash,
    include_active: bool,
    include_only_confirmed: bool,
) -> bool {
    let txn = node.ledger.read_txn();

    if include_active && !include_only_confirmed {
        return true;
    }

    if node.ledger.confirmed().block_exists_or_pruned(&txn, hash) {
        return true;
    }

    if !include_only_confirmed {
        if let Some(block) = node.ledger.get_block(&txn, hash) {
            return !node.active.active(&block);
        }
    }

    false
}
