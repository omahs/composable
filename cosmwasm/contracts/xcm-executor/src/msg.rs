use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo,
    Response, StdResult,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct QueryResp {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub enum QueryMsg {
    Metadata {
        // owner, cw20, treasury, transactor addresses
    },
}