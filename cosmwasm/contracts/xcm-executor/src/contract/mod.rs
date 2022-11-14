use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError,
    StdResult,
};

pub mod execute;

pub use execute::execute;
use crate::msg::*;
use crate::prelude::*;

pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let resp = match msg {
        Metadata => QueryResp {
            message: "Hello World".to_string(),
        },
    };

    to_binary(&resp)
}

pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    Ok(Response::new())
}
