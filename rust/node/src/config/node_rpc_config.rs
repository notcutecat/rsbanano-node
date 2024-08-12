use anyhow::Result;
use rsnano_core::utils::TomlWriter;
use std::path::PathBuf;
use super::get_default_rpc_filepath;

#[derive(Debug, PartialEq)]
pub struct RpcChildProcessConfig {
    pub enable: bool,
    pub rpc_path: PathBuf,
}

impl RpcChildProcessConfig {
    pub fn new() -> Self {
        Self {
            enable: false,
            rpc_path: get_default_rpc_filepath(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct NodeRpcConfig {
    pub enable_sign_hash: bool,
    pub child_process: RpcChildProcessConfig,
}

impl NodeRpcConfig {
    pub fn new() -> Self {
        Self {
            enable_sign_hash: false,
            child_process: RpcChildProcessConfig::new(),
        }
    }
}

impl Default for NodeRpcConfig {
    fn default() -> Self {
        Self {
            enable_sign_hash: false,
            child_process: RpcChildProcessConfig::new(),
        }
    }
}
