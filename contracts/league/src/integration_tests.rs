use anyhow::{anyhow, Result};
use assert_matches::assert_matches;
use cosmwasm_std::{Addr, CosmosMsg, Empty, QueryRequest, StdError, StdResult, to_binary, WasmMsg, WasmQuery};
use cw1::Cw1Contract;
use cw4::Member;
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
use derivative::Derivative;
use player::contract as plContract;
use player::ContractError;
use player::msg::{InfoResponse as plInfoResponse, InstantiateMsg as plIntantiateMsg, QueryMsg as plQueryMsg};
use player::player_attributes::{AttrItem, PlayerAttributes, Positions};
use serde::{de::DeserializeOwned, Serialize};

//use crate::msg::{AdminListResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::msg::InstantiateMsg as team_InstantiateMsg;

fn mock_app() -> App {
    App::default()
}

fn contract_hold() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}


#[derive(Derivative)]
#[derivative(Debug)]
pub struct Suite {
    /// Application mock
    #[derivative(Debug = "ignore")]
    app: App,
    /// Special account
    pub owner: String,
    /// ID of stored code for cw1 contract
    code_id: u64,
}

impl Suite {
    pub fn init() -> Result<Suite> {
        let mut app = mock_app();
        let owner = "owner".to_owned();
        let cw1_id = app.store_code(contract_hold());

        Ok(Suite { app, owner, code_id: cw1_id })
    }

    pub fn instantiate_player_contract(&mut self, owner: String) -> Cw1Contract {
        let instantiate_msg =
                &plIntantiateMsg {
                    first_name: "Prunter".to_string(),
                    last_name: "Nash".to_string(),
                    position: Positions::RB,
                    attributes: PlayerAttributes {
                        hands: AttrItem { value: "0.0".to_string() },
                        accuracy: AttrItem { value: "0.0".to_string() },
                        speed: AttrItem { value: "0.0".to_string()},
                        strength: AttrItem { value: "0.0".to_string() },
                        leader: AttrItem { value: "0.0".to_string() },
                        pressure_threshold: AttrItem { value: "0.0".to_string()},
                        agility: AttrItem { value: "0.0".to_string() },
                        football_iq: AttrItem { value: "0.0".to_string() },
                        temperament: AttrItem { value: "0.0".to_string()},
                        angle_of_view: 0
                    }
                };

        let contract = self
            .app
            .instantiate_contract(
                self.code_id,
                Addr::unchecked(self.owner.clone()),
                &instantiate_msg,
                &[],
                "Whitelist",
                None,
            )
            .unwrap();
        Cw1Contract(contract)
    }

    pub fn instantiate_team_contract(&mut self, admins: Vec<String>, members: Vec<Member>) -> Cw1Contract {
        let test_msg =
                &team_InstantiateMsg {
                    name: "Power Monks".to_string(),
                    admin: Some(admins[0].clone()),
                    members
                };


        let contract = self
            .app
            .instantiate_contract(
                self.code_id,
                Addr::unchecked(self.owner.clone()),
                &test_msg,
                &[],
                "Whitelist",
                None,
            )
            .unwrap();
        Cw1Contract(contract)
    }


    pub fn query<M>(&self, target_contract: Addr, msg: M) -> Result<plInfoResponse, StdError>
    where
        M: Serialize + DeserializeOwned,
    {
        self.app.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: target_contract.to_string(),
            msg: to_binary(&msg).unwrap(),
        }))
    }
}

#[test]
fn proxy_create_team_message() {
    let mut suite = Suite::init().unwrap();

    let first_contract = suite.instantiate_team_contract(vec![suite.owner.clone()], vec![]);
    let second_contract =
        suite.instantiate_team_contract(vec![first_contract.addr().to_string()], vec![]);
    assert_ne!(second_contract, first_contract);

    /*
    let query_msg: plQueryMsg = plQueryMsg::GetInfo {};
    assert_matches!(
        suite.query(second_contract.addr(), query_msg),
        Ok(
            plInfoResponse {
                player,
                ..
            })
    );

     */
}

#[test]
fn proxy_create_player_message() {
    let mut suite = Suite::init().unwrap();

    let first_contract = suite.instantiate_player_contract(suite.owner.clone());
    let second_contract =
        suite.instantiate_player_contract(first_contract.addr().to_string());
    assert_ne!(second_contract, first_contract);

    /*
    let query_msg: plQueryMsg = plQueryMsg::GetInfo {};
    assert_matches!(
        suite.query(second_contract.addr(), query_msg),
        Ok(
            plInfoResponse {
                player,
                ..
            })
    );

     */
}
