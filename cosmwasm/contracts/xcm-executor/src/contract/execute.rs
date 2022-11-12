use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError,
    StdResult,
};

use crate::msg::*;

pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::ExecuteXcmInCredit { msg } => {
            use parity_scale_codec::Decode;
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

        }
    }
    Ok(Response::default())
}
