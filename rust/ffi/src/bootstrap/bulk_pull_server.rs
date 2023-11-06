use rsnano_core::utils::Logger;
use rsnano_node::{bootstrap::BulkPullServer, messages::Payload, transport::DeserializedMessage};
use std::sync::Arc;

use crate::{
    copy_hash_bytes,
    core::BlockHandle,
    ledger::datastore::LedgerHandle,
    messages::MessageHandle,
    utils::{LoggerHandle, LoggerMT, ThreadPoolHandle},
};

use super::bootstrap_server::TcpServerHandle;

pub struct BulkPullServerHandle(BulkPullServer);

#[no_mangle]
pub unsafe extern "C" fn rsn_bulk_pull_server_create(
    request: &MessageHandle,
    server: *mut TcpServerHandle,
    ledger: *mut LedgerHandle,
    logger: *mut LoggerHandle,
    thread_pool: *mut ThreadPoolHandle,
    logging_enabled: bool,
) -> *mut BulkPullServerHandle {
    let Payload::BulkPull(payload) = &request.message else {panic!("not a bulk_pull message")};
    let logger: Arc<dyn Logger> = Arc::new(LoggerMT::new(Box::from_raw(logger)));
    Box::into_raw(Box::new(BulkPullServerHandle(BulkPullServer::new(
        payload.clone(),
        (*server).0.clone(),
        (*ledger).0.clone(),
        logger,
        (*thread_pool).0.clone(),
        logging_enabled,
    ))))
}

#[no_mangle]
pub unsafe extern "C" fn rsn_bulk_pull_server_destroy(handle: *mut BulkPullServerHandle) {
    drop(Box::from_raw(handle))
}

#[no_mangle]
pub unsafe extern "C" fn rsn_bulk_pull_server_sent_count(
    handle: *const BulkPullServerHandle,
) -> u32 {
    (*handle).0.sent_count()
}

#[no_mangle]
pub unsafe extern "C" fn rsn_bulk_pull_server_max_count(
    handle: *const BulkPullServerHandle,
) -> u32 {
    (*handle).0.max_count()
}

#[no_mangle]
pub unsafe extern "C" fn rsn_bulk_pull_server_current(
    handle: *const BulkPullServerHandle,
    result: *mut u8,
) {
    copy_hash_bytes((*handle).0.current(), result);
}

#[no_mangle]
pub unsafe extern "C" fn rsn_bulk_pull_server_request(
    handle: &BulkPullServerHandle,
) -> *mut MessageHandle {
    // only for tests
    MessageHandle::new(DeserializedMessage::new(
        Payload::BulkPull(handle.0.request()),
        Default::default(),
    ))
}

#[no_mangle]
pub unsafe extern "C" fn rsn_bulk_pull_server_send_next(handle: *mut BulkPullServerHandle) {
    (*handle).0.send_next();
}

#[no_mangle]
pub unsafe extern "C" fn rsn_bulk_pull_server_get_next(
    handle: *mut BulkPullServerHandle,
) -> *mut BlockHandle {
    let block = (*handle).0.get_next();
    match block {
        Some(b) => Box::into_raw(Box::new(BlockHandle(Arc::new(b)))),
        None => std::ptr::null_mut(),
    }
}
