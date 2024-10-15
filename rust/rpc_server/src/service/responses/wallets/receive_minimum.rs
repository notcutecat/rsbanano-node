use rsnano_node::Node;
use rsnano_rpc_messages::{AmountRpcMessage, ErrorDto};
use serde_json::to_string_pretty;
use std::sync::Arc;

pub async fn receive_minimum(node: Arc<Node>, enable_control: bool) -> String {
    if enable_control {
        let amount = node.config.receive_minimum;
        to_string_pretty(&AmountRpcMessage::new(amount)).unwrap()
    } else {
        to_string_pretty(&ErrorDto::new("RPC control is disabled".to_string())).unwrap()
    }
}
