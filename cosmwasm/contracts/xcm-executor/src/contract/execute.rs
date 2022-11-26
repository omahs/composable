use cosmwasm_std::Api;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError,
    StdResult,
};

use crate::msg::*;
use crate::prelude::*;

pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    let origin = deps.api.addr_canonicalize(info.sender.as_str())?;
    match msg {
        ExecuteMsg::ExecuteXcmInCredit { msg } => {
            let origin = MultiLocation {
                parents: 0,
                interior: X1(AccountId32 {
                    network: None,
                    id: origin.0 .0.try_into().map_err(|_| StdError::ParseErr {
                        target_type: "AccountId32".to_string(),
                        msg: "Only AccountId32 origin are supported now".to_string(),
                    })?,
                }),
            };
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
                VersionedXcm::V3(xcm) => {}
                _ => Err(StdError::GenericErr {
                    msg: "Not supported version".to_string(),
                })?,
            };
            panic!()
        }
    }
    Ok(Response::default())
}
