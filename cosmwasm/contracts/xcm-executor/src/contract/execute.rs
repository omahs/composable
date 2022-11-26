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
            let xcm = <xcm::VersionedXcm<Vec<u8>>>::decode(&mut &msg[..]).map_err(|x| {
                StdError::ParseErr {
                    target_type: "xcm::VersionedXcm<Vec<u8>>".to_string(),
                    msg: "Failed to deserialize XCM".to_string(),
                }
            })?;
            
            let xcm = match xcm {
                VersionedXcm::V3(xcm) => {
                    for instruction in xcm.0 {
                        match instruction {
                            WithdrawAsset(assets) => {
                                for asset in assets.into_inner() {
                                    
                                }
                            },
                            ReserveAssetDeposited(_) => todo!(),
                            ReceiveTeleportedAsset(_) => todo!(),
                            QueryResponse { query_id, response, max_weight, querier } => todo!(),
                            TransferAsset { assets, beneficiary } => todo!(),
                            TransferReserveAsset { assets, dest, xcm } => todo!(),
                            Transact { origin_kind, require_weight_at_most, call } => todo!(),
                            HrmpNewChannelOpenRequest { sender, max_message_size, max_capacity } => todo!(),
                            HrmpChannelAccepted { recipient } => todo!(),
                            HrmpChannelClosing { initiator, sender, recipient } => todo!(),
                            ClearOrigin => todo!(),
                            DescendOrigin(_) => todo!(),
                            ReportError(_) => todo!(),
                            DepositAsset { assets, beneficiary } => todo!(),
                            DepositReserveAsset { assets, dest, xcm } => todo!(),
                            ExchangeAsset { give, want, maximal } => todo!(),
                            InitiateReserveWithdraw { assets, reserve, xcm } => todo!(),
                            InitiateTeleport { assets, dest, xcm } => todo!(),
                            ReportHolding { response_info, assets } => todo!(),
                            BuyExecution { fees, weight_limit } => todo!(),
                            RefundSurplus => todo!(),
                            SetErrorHandler(_) => todo!(),
                            SetAppendix(_) => todo!(),
                            ClearError => todo!(),
                            ClaimAsset { assets, ticket } => todo!(),
                            Trap(_) => todo!(),
                            SubscribeVersion { query_id, max_response_weight } => todo!(),
                            UnsubscribeVersion => todo!(),
                            BurnAsset(_) => todo!(),
                            ExpectAsset(_) => todo!(),
                            ExpectOrigin(_) => todo!(),
                            ExpectError(_) => todo!(),
                            QueryPallet { module_name, response_info } => todo!(),
                            ExpectPallet { index, name, module_name, crate_major, min_crate_minor } => todo!(),
                            ReportTransactStatus(_) => todo!(),
                            ClearTransactStatus => todo!(),
                            UniversalOrigin(_) => todo!(),
                            ExportMessage { network, destination, xcm } => todo!(),
                            LockAsset { asset, unlocker } => todo!(),
                            UnlockAsset { asset, target } => todo!(),
                            NoteUnlockable { asset, owner } => todo!(),
                            RequestUnlock { asset, locker } => todo!(),
                            SetFeesMode { jit_withdraw } => todo!(),
                            SetTopic(_) => todo!(),
                            ClearTopic => todo!(),
                            AliasOrigin(_) => todo!(),
                            UnpaidExecution { weight_limit, check_origin } => todo!(),
                        }
                    }
                }
                _ => Err(StdError::GenericErr {
                    msg: "Not supported version".to_string(),
                })?,
            };
            panic!()
        }
    }
    Ok(Response::default())
}
