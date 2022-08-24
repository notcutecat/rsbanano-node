mod account_store;
mod ledger;
pub mod lmdb;
mod write_database_queue;

use std::{
    any::Any,
    cmp::{max, min},
};

pub use account_store::AccountStore;
pub use ledger::Ledger;
use primitive_types::U256;
pub use write_database_queue::{WriteDatabaseQueue, WriteGuard, Writer};

use crate::utils::get_cpu_count;

use self::lmdb::LmdbRawIterator;

pub trait Transaction {
    fn as_any(&self) -> &(dyn Any + '_);
}

pub trait ReadTransaction: Transaction {}

pub trait WriteTransaction: Transaction {
    fn as_transaction(&self) -> &dyn Transaction;
}

pub trait DbIterator<K, V> {
    fn take_lmdb_raw_iterator(&mut self) -> Option<LmdbRawIterator>;
}

pub struct NullIterator {}

impl NullIterator {
    pub fn new() -> Self {
        Self {}
    }
}

impl<K, V> DbIterator<K, V> for NullIterator {
    fn take_lmdb_raw_iterator(&mut self) -> Option<LmdbRawIterator> {
        None
    }
}

pub fn parallel_traversal(action: &(impl Fn(U256, U256, bool) + Send + Sync)) {
    // Between 10 and 40 threads, scales well even in low power systems as long as actions are I/O bound
    let thread_count = max(10, min(40, 11 * get_cpu_count()));
    let value_max = U256::max_value();
    let split = value_max / thread_count;

    std::thread::scope(|s| {
        for thread in 0..thread_count {
            let start = split * thread;
            let end = split * (thread + 1);
            let is_last = thread == thread_count - 1;

            std::thread::Builder::new()
                .name("DB par traversl".to_owned())
                .spawn_scoped(s, move || {
                    action(start, end, is_last);
                })
                .unwrap();
        }
    });
}
