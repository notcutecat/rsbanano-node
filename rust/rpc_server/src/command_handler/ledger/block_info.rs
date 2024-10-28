use crate::command_handler::RpcCommandHandler;
use rsnano_core::BlockType;
use rsnano_rpc_messages::{BlockInfoResponse, BlockSubTypeDto, HashRpcMessage};

impl RpcCommandHandler {
    pub(crate) fn block_info(&self, args: HashRpcMessage) -> anyhow::Result<BlockInfoResponse> {
        let txn = self.node.ledger.read_txn();
        let block = self.load_block_any(&txn, &args.hash)?;
        let account = block.account();

        let amount = self.node.ledger.any().block_amount(&txn, &args.hash);

        let balance = self
            .node
            .ledger
            .any()
            .block_balance(&txn, &args.hash)
            .unwrap();

        let sideband = block.sideband().unwrap();
        let height = sideband.height;
        let local_timestamp = sideband.timestamp;
        let successor = sideband.successor;

        let confirmed = self
            .node
            .ledger
            .confirmed()
            .block_exists_or_pruned(&txn, &args.hash);

        let contents = block.json_representation();

        let subtype: Option<BlockSubTypeDto> = if block.block_type() == BlockType::State {
            Some(sideband.details.subtype().into())
        } else {
            None
        };

        Ok(BlockInfoResponse {
            block_account: account,
            amount,
            balance,
            height,
            local_timestamp,
            successor,
            confirmed,
            contents,
            subtype,
            source_account: None,
            receive_hash: None,
            receivable: None,
        })
    }
}
