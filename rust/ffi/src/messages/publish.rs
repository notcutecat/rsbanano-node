use super::{create_message_handle3, message_handle_clone, MessageHandle};
use crate::{core::BlockHandle, NetworkConstantsDto, StringDto};
use rsnano_node::messages::{MessageEnum, Payload, PublishPayload};
use std::{ops::Deref, sync::Arc};

#[no_mangle]
pub unsafe extern "C" fn rsn_message_publish_create(
    constants: *mut NetworkConstantsDto,
    block: &BlockHandle,
) -> *mut MessageHandle {
    create_message_handle3(constants, |protocol_info| {
        let block = Arc::clone((*block).deref());
        MessageEnum::new_publish(protocol_info, block)
    })
}

#[no_mangle]
pub extern "C" fn rsn_message_publish_clone(handle: &MessageHandle) -> *mut MessageHandle {
    message_handle_clone(handle)
}

fn get_publish_payload(handle: &MessageHandle) -> &PublishPayload {
    let Payload::Publish(payload) = &handle.message else {panic!("not a payload message")};
    payload
}

fn get_publish_payload_mut(handle: &mut MessageHandle) -> &mut PublishPayload {
    let Payload::Publish(payload) = &mut handle.message else {panic!("not a payload message")};
    payload
}

#[no_mangle]
pub unsafe extern "C" fn rsn_message_publish_block(handle: &MessageHandle) -> *mut BlockHandle {
    BlockHandle::new(get_publish_payload(handle).block.clone())
}

#[no_mangle]
pub unsafe extern "C" fn rsn_message_publish_digest(handle: &MessageHandle, result: *mut u8) {
    let result_slice = std::slice::from_raw_parts_mut(result, 16);
    let digest = get_publish_payload(handle).digest;
    result_slice.copy_from_slice(&digest.to_be_bytes());
}

#[no_mangle]
pub unsafe extern "C" fn rsn_message_publish_set_digest(
    handle: &mut MessageHandle,
    digest: *const u8,
) {
    let bytes = std::slice::from_raw_parts(digest, 16);
    let digest = u128::from_be_bytes(bytes.try_into().unwrap());
    get_publish_payload_mut(handle).digest = digest;
}

#[no_mangle]
pub unsafe extern "C" fn rsn_message_publish_to_string(
    handle: &mut MessageHandle,
    result: *mut StringDto,
) {
    (*result) = handle.message.to_string().into();
}
