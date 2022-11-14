use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError,
    StdResult,
};

use crate::prelude::*;
use crate::msg::*;

pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::ExecuteXcmInCredit { msg } => {
            use parity_scale_codec::Decode;
            // let k = xcm::v3::Junction::Parachain(42);
            // let k = xcm::v3::Junction::GeneralKey([42;32]);
            // let k = xcm::v3::Junction::GeneralIndex(42);
            // let k = xcm::v3::MultiLocation { parents: 42, interior: todo!() };
            let xcm = <xcm::VersionedXcm<Vec<u8>>>::decode(&mut &msg[..]).map_err(|x| {
                StdError::ParseErr {
                    target_type: "xcm::VersionedXcm<Vec<u8>>".to_string(),
                    msg: "Failed to deserialize XCM".to_string(),
                }
            })?;
            let xcm = match xcm {
                xcm::VersionedXcm::V2(xcm) => xcm,
                _ => Err(StdError::GenericErr {
                    msg: "Not supported version".to_string(),
                })?,
            };
            panic!()
        }
    }
    Ok(Response::default())
}
