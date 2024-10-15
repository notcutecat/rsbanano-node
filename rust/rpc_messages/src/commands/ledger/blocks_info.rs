use std::collections::HashMap;
use crate::{HashesArgs, RpcCommand};
use rsnano_core::BlockHash;
use serde::{Deserialize, Serialize};
use super::BlockInfoDto;

impl RpcCommand {
    pub fn blocks_info(hashes: Vec<BlockHash>) -> Self {
        Self::BlocksInfo(HashesArgs::new(hashes))
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct BlocksInfoDto {
    blocks: HashMap<BlockHash, BlockInfoDto>,
}

impl BlocksInfoDto {
    pub fn new(blocks: HashMap<BlockHash, BlockInfoDto>) -> Self {
        Self { blocks }
    }
}
