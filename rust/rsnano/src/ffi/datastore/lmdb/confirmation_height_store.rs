use std::sync::Arc;

use crate::{
    datastore::{lmdb::LmdbConfirmationHeightStore, ConfirmationHeightStore},
    ffi::ConfirmationHeightInfoDto,
    Account, ConfirmationHeightInfo,
};

use super::{lmdb_env::LmdbEnvHandle, TransactionHandle};

pub struct LmdbConfirmationHeightStoreHandle(LmdbConfirmationHeightStore);

#[no_mangle]
pub unsafe extern "C" fn rsn_lmdb_confirmation_height_store_create(
    env_handle: *mut LmdbEnvHandle,
) -> *mut LmdbConfirmationHeightStoreHandle {
    Box::into_raw(Box::new(LmdbConfirmationHeightStoreHandle(
        LmdbConfirmationHeightStore::new(Arc::clone(&*env_handle)),
    )))
}

#[no_mangle]
pub unsafe extern "C" fn rsn_lmdb_confirmation_height_store_destroy(
    handle: *mut LmdbConfirmationHeightStoreHandle,
) {
    drop(Box::from_raw(handle))
}

#[no_mangle]
pub unsafe extern "C" fn rsn_lmdb_confirmation_height_store_table_handle(
    handle: *mut LmdbConfirmationHeightStoreHandle,
) -> u32 {
    (*handle).0.table_handle
}

#[no_mangle]
pub unsafe extern "C" fn rsn_lmdb_confirmation_height_store_set_table_handle(
    handle: *mut LmdbConfirmationHeightStoreHandle,
    table_handle: u32,
) {
    (*handle).0.table_handle = table_handle;
}

#[no_mangle]
pub unsafe extern "C" fn rsn_lmdb_confirmation_height_store_put(
    handle: *mut LmdbConfirmationHeightStoreHandle,
    txn: *mut TransactionHandle,
    account: *const u8,
    info: *const ConfirmationHeightInfoDto,
) {
    (*handle).0.put(
        (*txn).as_write_txn(),
        &Account::from_ptr(account),
        &ConfirmationHeightInfo::from(&*info),
    );
}
