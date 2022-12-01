


pub mod receive {
    use cosmwasm_std::Coin;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use shared::saleable::Saleable;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        Buy {},
        Update { for_sale_status: bool, price: Option<Coin>}
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        GetInfo{}
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct GetInfoResponse {
        pub info: Saleable,
    }
}
