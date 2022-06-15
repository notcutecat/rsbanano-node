use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, MutexGuard, Weak,
};

use super::{Channel, Socket, SocketImpl};

pub struct TcpChannelData {
    last_bootstrap_attempt: u64,
    last_packet_received: u64,
    last_packet_sent: u64,
}

pub struct ChannelTcp {
    channel_mutex: Mutex<TcpChannelData>,
    socket: Weak<SocketImpl>,
    temporary: AtomicBool,
}

impl ChannelTcp {
    pub fn new(socket: &Arc<SocketImpl>, now: u64) -> Self {
        Self {
            channel_mutex: Mutex::new(TcpChannelData {
                last_bootstrap_attempt: 0,
                last_packet_received: now,
                last_packet_sent: now,
            }),
            socket: Arc::downgrade(socket),
            temporary: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) -> MutexGuard<TcpChannelData> {
        self.channel_mutex.lock().unwrap()
    }
}

impl Channel for ChannelTcp {
    fn is_temporary(&self) -> bool {
        self.temporary.load(Ordering::SeqCst)
    }

    fn set_temporary(&self, temporary: bool) {
        self.temporary.store(temporary, Ordering::SeqCst);
    }

    fn get_last_bootstrap_attempt(&self) -> u64 {
        self.channel_mutex.lock().unwrap().last_bootstrap_attempt
    }

    fn set_last_bootstrap_attempt(&self, instant: u64) {
        self.channel_mutex.lock().unwrap().last_bootstrap_attempt = instant;
    }

    fn get_last_packet_received(&self) -> u64 {
        self.channel_mutex.lock().unwrap().last_packet_received
    }

    fn set_last_packet_received(&self, instant: u64) {
        self.channel_mutex.lock().unwrap().last_packet_received = instant;
    }

    fn get_last_packet_sent(&self) -> u64 {
        self.channel_mutex.lock().unwrap().last_packet_sent
    }

    fn set_last_packet_sent(&self, instant: u64) {
        self.channel_mutex.lock().unwrap().last_packet_sent = instant;
    }
}

impl Drop for ChannelTcp {
    fn drop(&mut self) {
        // Close socket. Exception: socket is used by bootstrap_server
        if let Some(socket) = self.socket.upgrade() {
            if !self.temporary.load(Ordering::Relaxed) {
                socket.close();
            }
        }
    }
}
