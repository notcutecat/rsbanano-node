use std::{net::SocketAddr, sync::Arc};

use crate::{
    bootstrap::BootstrapServer,
    transport::{Channel, ChannelTcp, SocketImpl},
    Account,
};

pub struct ChannelTcpWrapper {
    pub channel: Arc<ChannelTcp>,
    socket: Arc<SocketImpl>,
    pub response_server: Option<Arc<BootstrapServer>>,
}

impl ChannelTcpWrapper {
    pub fn new(
        channel: Arc<ChannelTcp>,
        socket: Arc<SocketImpl>,
        response_server: Option<Arc<BootstrapServer>>,
    ) -> Self {
        Self {
            channel,
            socket,
            response_server,
        }
    }

    pub fn endpoint(&self) -> SocketAddr {
        self.channel.endpoint()
    }

    pub fn last_packet_sent(&self) -> u64 {
        self.channel.get_last_packet_sent()
    }

    pub fn last_bootstrap_attempt(&self) -> u64 {
        self.channel.get_last_bootstrap_attempt()
    }

    pub fn socket(&self) -> Option<Arc<SocketImpl>> {
        self.channel.socket()
    }

    pub fn node_id(&self) -> Option<Account> {
        self.channel.get_node_id()
    }

    pub fn network_version(&self) -> u8 {
        self.channel.network_version()
    }
}
