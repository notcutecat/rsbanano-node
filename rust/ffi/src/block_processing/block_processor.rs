use super::unchecked_map::UncheckedMapHandle;
use crate::{
    core::BlockHandle, ledger::datastore::LedgerHandle, transport::ChannelHandle,
    work::WorkThresholdsDto, NodeConfigDto, NodeFlagsHandle, StatHandle,
};
use num_traits::FromPrimitive;
use rsnano_core::work::WorkThresholds;
use rsnano_node::{
    block_processing::{BlockProcessor, BlockSource},
    config::NodeConfig,
};
use std::{ops::Deref, sync::Arc};

pub struct BlockProcessorHandle(pub Arc<BlockProcessor>);

impl Deref for BlockProcessorHandle {
    type Target = Arc<BlockProcessor>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[no_mangle]
pub unsafe extern "C" fn rsn_block_processor_create(
    config: &NodeConfigDto,
    flags: &NodeFlagsHandle,
    ledger: &LedgerHandle,
    unchecked_map: &UncheckedMapHandle,
    stats: &StatHandle,
    work: &WorkThresholdsDto,
) -> *mut BlockProcessorHandle {
    let config = Arc::new(NodeConfig::try_from(config).unwrap());
    let flags = Arc::new(flags.lock().unwrap().clone());
    let ledger = Arc::clone(&ledger);
    let unchecked_map = Arc::clone(&unchecked_map);
    let stats = Arc::clone(&stats);
    let work = Arc::new(WorkThresholds::from(work));
    let processor = Arc::new(BlockProcessor::new(
        config,
        flags,
        ledger,
        unchecked_map,
        stats,
        work,
    ));
    Box::into_raw(Box::new(BlockProcessorHandle(processor)))
}

#[no_mangle]
pub extern "C" fn rsn_block_processor_destroy(handle: *mut BlockProcessorHandle) {
    drop(unsafe { Box::from_raw(handle) });
}

#[no_mangle]
pub extern "C" fn rsn_block_processor_full(handle: &BlockProcessorHandle) -> bool {
    handle.full()
}

#[no_mangle]
pub extern "C" fn rsn_block_processor_half_full(handle: &BlockProcessorHandle) -> bool {
    handle.half_full()
}

#[no_mangle]
pub extern "C" fn rsn_block_processor_stop(handle: &BlockProcessorHandle) {
    handle.stop();
}

#[no_mangle]
pub unsafe extern "C" fn rsn_block_processor_add(
    handle: &mut BlockProcessorHandle,
    block: &BlockHandle,
    source: u8,
    channel: *const ChannelHandle,
) -> bool {
    let channel = if channel.is_null() {
        None
    } else {
        Some(Arc::clone(&*channel))
    };
    handle.add(
        Arc::clone(block),
        FromPrimitive::from_u8(source).unwrap(),
        channel,
    )
}

#[no_mangle]
pub extern "C" fn rsn_block_processor_add_blocking(
    handle: &mut BlockProcessorHandle,
    block: &BlockHandle,
    source: u8,
    status: &mut u8,
) -> bool {
    match handle.add_blocking(Arc::clone(block), BlockSource::from_u8(source).unwrap()) {
        Some(i) => {
            *status = i as u8;
            true
        }
        None => false,
    }
}

#[no_mangle]
pub extern "C" fn rsn_block_processor_force(
    handle: &mut BlockProcessorHandle,
    block: &BlockHandle,
) {
    handle.force(Arc::clone(block));
}
