pub mod receive {
    use cw4::Member;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InstantiateMsg {
        pub count: i32,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        RemoveHook { addr: String },
        AddHook { addr: String },
        UpdateAdmin { admin_addr: Option<String> },
        UpdateMembers {
            remove: Vec<String>,
            add: Vec<Member>,
        },

    }

/*

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub members: Vec<Member>,
}

*/
}
