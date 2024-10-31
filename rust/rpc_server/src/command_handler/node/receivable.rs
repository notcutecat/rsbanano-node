use crate::command_handler::RpcCommandHandler;
use indexmap::IndexMap;
use rsnano_core::{Amount, BlockHash};
use rsnano_rpc_messages::{
    ReceivableArgs, ReceivableResponse, ReceivableSimple, ReceivableSource, ReceivableThreshold,
    SourceInfo,
};

impl RpcCommandHandler {
    pub(crate) fn receivable(&self, args: ReceivableArgs) -> ReceivableResponse {
        let count = args.count.unwrap_or(u64::MAX) as usize;
        let offset = args.offset.unwrap_or(0) as usize;
        let threshold = args.threshold.unwrap_or_default();
        let source = args.source.unwrap_or(false);
        let min_version = args.min_version.unwrap_or(false);
        let include_only_confirmed = args.include_only_confirmed.unwrap_or(true);
        let sorting = args.sorting.unwrap_or(false);

        let mut offset_counter = offset;

        // if simple, response is a list of hashes
        let simple = threshold.is_zero() && !source && !min_version && !sorting;
        let should_sort = sorting && !simple;

        let mut peers_simple = Vec::new();
        let mut peers_source: IndexMap<BlockHash, SourceInfo> = IndexMap::new();
        let mut peers_amount: IndexMap<BlockHash, Amount> = IndexMap::new();
        let tx = self.node.store.tx_begin_read();

        let receivables = self.node.ledger.any().account_receivable_upper_bound(
            &tx,
            args.account,
            BlockHash::zero(),
        );

        for (key, info) in receivables {
            if !should_sort && (peers_simple.len() >= count || peers_source.len() >= count) {
                break;
            }

            if include_only_confirmed
                && !self
                    .node
                    .ledger
                    .confirmed()
                    .block_exists_or_pruned(&tx, &key.send_block_hash)
            {
                continue;
            }

            if !should_sort && offset_counter > 0 {
                offset_counter -= 1;
                continue;
            }

            if simple {
                peers_simple.push(key.send_block_hash);
                continue;
            }

            if info.amount < threshold {
                continue;
            }

            if source || min_version {
                let source_info = SourceInfo {
                    amount: info.amount,
                    source: source.then(|| info.source),
                    min_version: min_version.then(|| info.epoch.epoch_number()),
                };
                peers_source.insert(key.send_block_hash, source_info);
            } else {
                peers_amount.insert(key.send_block_hash, info.amount);
            }
        }

        if should_sort {
            if source || min_version {
                if offset >= peers_source.len() {
                    peers_source.clear()
                } else {
                    peers_source.sort_by(|_, v1, _, v2| v2.amount.cmp(&v1.amount));
                    peers_source = peers_source.split_off(offset);
                    peers_source.truncate(count);
                }
            } else {
                if offset >= peers_amount.len() {
                    peers_amount.clear();
                } else {
                    peers_amount.sort_by(|_, v1, _, v2| v2.cmp(v1));
                    peers_amount = peers_amount.split_off(offset);
                    peers_amount.truncate(count);
                }
            }
        }
        if simple {
            ReceivableResponse::Simple(ReceivableSimple {
                blocks: peers_simple,
            })
        } else if source || min_version {
            ReceivableResponse::Source(ReceivableSource {
                blocks: peers_source,
            })
        } else {
            ReceivableResponse::Threshold(ReceivableThreshold {
                blocks: peers_amount,
            })
        }
    }
}
