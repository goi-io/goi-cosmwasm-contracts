use cosmwasm_std::{Addr, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Saleable {
    pub price_version: i32,
    pub price: Option<Coin>,
    pub for_sale: bool,
}

pub enum DistributionType {
    Owner,
    Services,
    Depositor
}


pub struct DistributionPacket {
    pub distribution_type: DistributionType,
    pub description: String,
    pub to_address: Addr,
    pub amount: Coin,
}
