use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo,
    Response, StdResult,
};
use crate::prelude::*;

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

#[derive(Serialize, Deserialize)]
pub enum ExecuteMsg {
    ExecuteXcmInCredit {
        msg : Vec<u8>
    },
}
