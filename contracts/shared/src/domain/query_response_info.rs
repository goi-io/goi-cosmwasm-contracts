use cosmwasm_std::Addr;
use cw4::Member;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::manage::Manageable;
use crate::saleable::Saleable;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoManagedResponse<T> {
    pub data: T,
    pub sale_info: Saleable,
    pub managed_info: Manageable,
    pub owners: Vec<Member>,
    pub admin: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse<T> {
    pub data: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NameResponse {
    pub name: String,
}