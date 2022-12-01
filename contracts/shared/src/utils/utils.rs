use cosmwasm_std::{Addr, Decimal, Timestamp, Coin, BlockInfo};
use cw4::Member;
use cw_storage_plus::IntKey;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::manage::ManagedStatus;
use crate::utils::general::AssetTypes;



pub type AppAddress = Addr;
pub type ContractAddress = Addr;
pub type TaskAddress = Addr;
pub type JsonData = String;
pub type LNameFNameString = String;
pub type LName = String;
pub type FName = String;
pub type PlayerAddr = Addr;
pub type TeamAddr = Addr;
pub type OwnerAddr = Addr;
pub type PlayerTeamAddr = (PlayerAddr, TeamAddr);
pub type TeamPlayerAddr = (TeamAddr, PlayerAddr);
pub type LeagueAddr = Addr;
pub type SeasonId = u64;
pub type SeasonDepositId = u64;
pub type SeasonActiveStatusValue = u8; //works in concert with enum SeasonActiveStatus
pub type BlockChainTimeValue = u64;   //TimeStamp
pub type StartDate = u64; //TimeStamp
pub type EndDate = u64; //TimeStamp
pub type MessageId = u64;
pub type InviteAccepted = u8;
pub type AsseTypes_u8 = u8;

pub const PRIOR_TO_SEASON_START_PADDING: u64 = 900;

pub const THIRTY_MINUTES: u64 = 1800;
pub const FIFTEEN_MINUTES: u64 = 900;
pub const MAX_TEAMS_ALLOWED: u32 = 300;

pub const ONE_MINUTE: u64 = 60;




#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Endpoint {
    pub server: String,
    pub path: String
}

pub struct ManagementContractInfo {
    pub address: Addr,
}

pub trait IManagerResponse {
    fn get_management_contract_info(&self) -> ManagementContractInfo;
}

pub trait IManaged {
    /*
    Struct fields:
    managed_status: ManagedStatus,
    for_sale: bool,
     */
    fn asset_type(&self) -> AssetTypes;
    fn managed_status(&self) -> ManagedStatus;
    fn set_managed_status(&mut self, status: ManagedStatus) -> ();
    fn for_sale(&self) -> bool;
    fn set_for_sale(&mut self, status: bool) -> ();
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ManagedItemResponse {
    pub managed_status: ManagedStatus,
    pub for_sale: bool,

    pub asset_type: AssetTypes,
    pub contract_addr: Option<Addr>,
    pub for_sale_price: Option<Coin>,
    pub for_sale_price_version: u32,
    pub for_sale_last_updated: Timestamp,
    pub ownership_history: Vec<OwnershipHistory>
}

impl IManaged for ManagedItemResponse {
    fn asset_type(&self) -> AssetTypes {
        self.asset_type.clone()
    }
    /*
    Struct fields:
    managed_status: ManagedStatus,
    for_sale: bool,
     */

    fn managed_status(&self) -> ManagedStatus {
        self.managed_status.clone()
    }

    fn set_managed_status(&mut self, status: ManagedStatus) -> () {
        self.managed_status = status
    }

    fn for_sale(&self) -> bool {
        self.for_sale
    }

    fn set_for_sale(&mut self, status: bool) -> () {
        self.for_sale = status;
    }
}




#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum FeeType {
    Dev,
    ServiceProvider
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ForSaleStatus {
    ForSale,
    NotForSale
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Fee {
    pub fee_type: FeeType,
    pub description: Option<String>,
    pub to_address: Addr,
    pub percent: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnershipHistory {
    pub owners: Addr,
    pub purchased: BlockInfo,
    pub sold: Option<BlockInfo>,
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MangedItem {
    pub asset_addr: Addr,
    pub asset_name: Option<String>,
    pub asset_owner: Addr,
    pub asset_type: AssetTypes,
    pub managed_status: ManagedStatus,
    pub created: Timestamp,
    pub updated: Timestamp,
    pub for_sale: u8,
    pub for_sale_price: Option<Coin>,
    pub for_sale_price_version: u32,
    pub for_sale_last_updated: Timestamp,
    pub ownership_history: Vec<OwnershipHistory>
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnedAsset {
    pub asset_addr: Addr,
    pub asset_type: AssetTypes,
}




#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssetSaleItem {
    pub ass_addr: Addr,
    pub asset_type: AssetTypes,
    pub created: Timestamp,
    pub updated: Timestamp,
    pub for_sale_price: Option<Coin>,
    pub for_sale: bool
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssetSaleItemAddUpdateModel {
    pub ass_addr: Addr,
    pub asset_type: AssetTypes,
    pub for_sale_price: Option<Coin>,
    pub for_sale: bool,
    pub for_sale_price_version: u32,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssetSaleItems {
    pub items: Vec<AssetSaleItem>

}




#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BlockTime {
    pub height: u64,
    pub time: Timestamp,
    pub chain_id: String,
}


pub mod general {
    use std::collections::HashSet;
    use std::ops::{Mul, Sub};
    use cosmwasm_std::{Addr, Coin, CosmosMsg, Decimal, ReplyOn, SubMsg};
    use cw4::Member;
    use cw_storage_plus::{IntKey, PrimaryKey};
    use schemars::JsonSchema;
    use serde_repr::{Deserialize_repr, Serialize_repr};

    use crate::data::ModelItem;
    use crate::GoiError;
    use crate::saleable::{DistributionPacket, DistributionType};
    use crate::team::TeamInfo;
    use crate::utils::Fee;


    fn payment_distribution(payees: Vec<Member>,
                                    fees: Option<Vec<Fee>>, balance: Coin )
                                    -> Result<Option<Vec<SubMsg>>, GoiError> {

        if balance.amount.is_zero() {
            return Err(GoiError::InsufficientFund {} );
        }



        let mut distributions: Option< Vec<DistributionPacket>> = None;
        match payees.len() > 0 {
            true => {
                let mut amount_to_distribute_after_fees = balance.amount.clone();
                let mut distribution_hold: Vec<DistributionPacket> = Vec::default();

                match fees {
                    Some (fs) => {
                        for item in fs {
                            let fee_amount = balance.amount.mul(item.percent);
                            amount_to_distribute_after_fees =
                                amount_to_distribute_after_fees.sub(fee_amount);
                            distribution_hold.push(DistributionPacket {
                                distribution_type: DistributionType::Services,
                                description: "Fee(s) or Tax".to_string(),
                                to_address: item.to_address,
                                amount: Coin { denom: balance.denom.clone(), amount: fee_amount },
                            })
                        }
                    },
                    None => ()
                }

                for payee in payees {
                    distribution_hold.push( DistributionPacket {
                        distribution_type: DistributionType::Owner,
                        description: "Asset sell distribution".to_string(),
                        to_address: Addr::unchecked (payee.addr),
                        amount:Coin
                        {
                            denom: balance.denom.clone(),
                            amount: (amount_to_distribute_after_fees.mul( Decimal::percent(payee.weight)))
                        },

                    });
                }
                distributions = Some(distribution_hold);
            },
            false => ()
        }

        match distributions {
            None => Ok(None),
            Some(items) => {
                let mut count: u64 = 0;
                let res: Vec<SubMsg> =
                    items.into_iter()
                        .map(|i| {
                            let res:CosmosMsg =
                                cosmwasm_std::BankMsg::Send
                                {
                                    to_address: i.to_address.to_string(),
                                    amount: vec![i.amount]
                                }.into();
                            let res_sub_msg =
                                SubMsg{
                                    id: count,
                                    msg: res,
                                    gas_limit: None,
                                    reply_on: ReplyOn::Never
                                };
                            count += 1;
                            res_sub_msg

                        })
                        .collect();
                Ok(Some(res))

            }
        }

    }

    #[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, JsonSchema)]
    #[repr(u8)]
    pub enum AssetTypes {
        Team = 0,
        League = 1,
        Display = 2,
        App = 3,
    }

    impl  AssetTypes {
      pub  fn to_u8(&self) -> u8 {
             match self {
                 AssetTypes::Team => {
                     0u8
                 }
                 AssetTypes::League => {
                     1u8
                 }
                 AssetTypes::Display => {
                     2u8
                 }
                 AssetTypes::App => {
                     3u8
                 }
             }

        }
    }
    



    
    #[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, JsonSchema)]
    #[repr(u8)]
    pub enum GameItemTypes {
        Player = 0,
        Reward = 1,
        Season = 2,
    }



    pub fn merge_data<T>(current_data: Option<T>, new_data: ModelItem<Option<T>>)
                     -> Option<T> {
        let new_data =
            match (new_data.update, new_data.data)  {
                (true, data) => data,
                (false, _) => None,

            };
        match (current_data, new_data) {
            (None, None) => None,
            (Some(_), Some(new_d)) => Some(new_d),
            (None, Some(new_d)) => Some(new_d),
            (Some(current_d) , None ) => Some(current_d),
        }
    }

    pub fn merge_strings(a: String, b: String) -> String {
        let mut owned_string: String = a.to_owned();
        let borrowed_string: &str = &b;
        owned_string.push_str(borrowed_string);
        owned_string
    }

    pub fn index_string(data: &str) -> Vec<u8> {
        data.as_bytes().to_vec()
    }


    pub fn generate_id_from_strings(a: String, b: String ) -> Vec<u8>{
        let merged = merge_strings(a, b);
        index_string(&merged)
    }




}



pub mod teams {

}

pub mod leagues {

}


pub mod displays {

}

pub mod management {


}

pub mod xnodes {
    use cosmwasm_std::{Addr, Coin};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use serde_repr::{Deserialize_repr, Serialize_repr};

    pub type XNodeAddress = Addr;
    pub type SuccessfulExecutionCount = i32;


    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct XNode {
        pub node_address: Addr,
        pub bonded_amount: Vec<Coin>,
        pub status: XNodeStatus
    }



    #[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, JsonSchema)]
    #[repr(u8)]
    pub enum XNodeStatus {
        Approved = 0,
        Denied = 1,
        Pending = 2,
        Suspended = 3,
    }

    impl Default for XNodeStatus {
        fn default() -> Self {
            XNodeStatus::Pending
        }
    }
}
