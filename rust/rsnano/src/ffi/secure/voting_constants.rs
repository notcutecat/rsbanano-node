use std::convert::TryFrom;

use crate::{config::NetworkConstants, ffi::config::NetworkConstantsDto, secure::VotingConstants};

#[repr(C)]
pub struct VotingConstantsDto {
    pub max_cache: usize,
    pub delay_s: i64,
}

#[no_mangle]
pub unsafe extern "C" fn rsn_voting_constants_create(
    network_constants: &NetworkConstantsDto,
    dto: *mut VotingConstantsDto,
) -> i32 {
    let network_constants = match NetworkConstants::try_from(network_constants) {
        Ok(n) => n,
        Err(_) => return -1,
    };
    let voting = VotingConstants::new(&network_constants);
    (*dto).max_cache = voting.max_cache;
    (*dto).delay_s = voting.delay_s;
    0
}
