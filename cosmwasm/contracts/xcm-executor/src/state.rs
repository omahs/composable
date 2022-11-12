use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const XCM_COUNTER: Item<usize> = Item::new("XCM_COUNTER");
// XCM message body
// XCM message pointer
// transactor address
// drop address
// account converter address