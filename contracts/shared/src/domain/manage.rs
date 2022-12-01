use cosmwasm_std::{Addr, Binary, Coin, CosmosMsg, Response, StdResult, to_binary, WasmMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};




use crate::utils::Fee;
use crate::utils::general::AssetTypes;

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ManagedStatus {
    Pending = 0,
    Enabled = 1,
    Disabled = 2,
    Suspended = 3,
}

impl ToString for ManagedStatus {
    fn to_string(&self) -> String {
       match self {
           ManagedStatus::Pending => {
               "Pending".to_string()
           }
           ManagedStatus::Enabled => {
               "Enabled".to_string()
           }
           ManagedStatus::Disabled => {
               "Disabled".to_string()
           }
           ManagedStatus::Suspended => {
               "Suspended".to_string()
           }
       }
    }
}


impl Default for ManagedStatus {
    fn default() -> Self {
        ManagedStatus::Pending
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Manageable {
    pub managing_contract: Option<Addr>,
    pub managed_asset_type: AssetTypes
    //pub active_status: ManagedStatus
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ManageableItemResponse {
    pub item: Manageable,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ManagementFee {
    pub id: i32,
    pub created_at_block_height: u64,
    pub active: bool,
    pub fees: Fee,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ManagedContract {
    pub managed:  ManagedStatus,
    pub contract: Addr,
    pub contract_type: AssetTypes,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Management {
    pub fees: Vec<ManagementFee>,
    pub description: String,
    //pub managed_contracts: Vec<ManagedContract>// Option<Vec<ManagedContract>>
}



#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ManagedStatusUpdate {
    pub managed_contract: String,
    pub manager_contract: String,
    pub managed_status: ManagedStatus,
}


impl ManagedStatusUpdate {
    pub fn new<T: Into<String>>(managed_contract: T, manager_contract: T, managed_status: ManagedStatus)-> Self {
        ManagedStatusUpdate {
            managed_contract: managed_contract.into(),
            manager_contract: manager_contract.into(),
            managed_status
        }
    }
}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ManagedStatusChangedHookMsg {
    pub change: ManagedStatusUpdate,
}


impl ManagedStatusChangedHookMsg {
    pub fn new (change: ManagedStatusUpdate) -> Self {
        ManagedStatusChangedHookMsg {
            change
        }
    }

    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = ManagedStatusChangedExecuteMsg::ManagedStatusChangedHook(self);
        to_binary(&msg)
    }

    pub fn into_cosmos_msg<T: Into<String>>(self, contract_addr: T) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}


// This is just a helper to properly serialize the above message
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
enum ManagedStatusChangedExecuteMsg {
    ManagedStatusChangedHook(ManagedStatusChangedHookMsg),
}





///Required interfaces to be implemented by manager contracts
pub mod receive {
    use cosmwasm_std::Addr;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};



    use super::{ManagedContract, ManagementFee};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        UpdateFees { add: Option<Vec<ManagementFee>>, remove: Option<Vec<i32>>}
    }





    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct ManagementInfoResponse {
        pub fees: Option<Vec<ManagementFee>>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct ManagedContractInfoResponse {
        pub contract: Option< ManagedContract>,
    }
}


pub type OnSuccessfulForSaleStatusUpdateExec = fn(for_sale_status: bool, price: Option<Coin>, managing_contract: Addr, response: Response) -> Response;
pub type OnSuccessfulBuyExec = fn(new_owner: Addr, managing_contract: Addr, response: Response) ->  Response;
pub type OnSuccessfulInit = fn(asset_name: Option<String>, asset_owner: Addr, asset_type: AssetTypes, managing_contract: Addr, response: Response) -> Response;