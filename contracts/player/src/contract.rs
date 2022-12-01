use cosmwasm_std::{Addr, Binary, Deps, DepsMut, Env, MessageInfo, QuerierWrapper, Response, StdResult, to_binary};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;


use shared::goi_manager::GoiManagerQueryMsg;
use shared::player::{PlayerInfo, Player};
pub use shared::player_attributes::Positions;

use crate::error::ContractError;
use shared::player::{InfoResponse, InstantiateMsg, QueryMsg};
use crate::state::{STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:player";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let name_valid = is_player_name_in_use(&deps.querier,
                                                            msg.first_name.clone(),
                                                            msg.last_name.clone(),
                                           msg.managing_contract_address);

    match name_valid {
        Ok(p) => {
            match p {
                None => {
                    let state = Player {
                        first_name: (&msg.first_name).parse().unwrap(),
                        last_name: (&msg.last_name).parse().unwrap(),
                        owner: info.sender.clone(),
                        position: msg.position,
                        attributes: msg.attributes,
                    };
                    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
                    STATE.save(deps.storage, &state)?;

                    Ok(Response::new()
                        .add_attribute("method", "instantiate")
                        .add_attribute("owner", info.sender)
                        .add_attribute("first_name", &msg.first_name)
                        .add_attribute("last_name", &msg.last_name)
                        .add_attribute("contract_address", &_env.contract.address))
                }
                Some(_) => {
                    Err(ContractError::PlayerNameAlreadyInUse
                    {
                        first_name: msg.first_name,
                        last_name: msg.last_name
                    })
                }
            }
        }
        Err(e) => {
            Err(ContractError::PlayerNameCheckError { error: e })
        }
    }


}


fn is_player_name_in_use(q_wrapper: &QuerierWrapper, first_name: String,
                         last_name: String, managing_contract_address: Addr)
                  -> StdResult<Option<PlayerInfo>>   {

        let res: StdResult<Option<PlayerInfo>> = q_wrapper.query_wasm_smart
        (managing_contract_address.clone(),
         &GoiManagerQueryMsg::GetPlayerByName { first_name, last_name });

        match res {
            Ok(p) => {
                Ok( p)
            }
            Err(e) => {
                Err(e)
            }
        }

}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetInfo {} => to_binary(&query_info(deps)?),
    }
}

fn query_info(deps: Deps) -> StdResult<InfoResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(InfoResponse { player: state })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    use shared::player_attributes::{AttrItem, PlayerAttributes};

    use super::*;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let attribs = PlayerAttributes {
            hands: AttrItem {value: "2.0".to_string()},
            accuracy: AttrItem {value: "2.0".to_string() },
            speed: AttrItem {value: "2.0".to_string() },
            strength: AttrItem {value: "2.0".to_string() },
            leader: AttrItem {value: "2.0".to_string() },
            pressure_threshold: AttrItem {value: "2.0".to_string() },
            agility: AttrItem {value: "2.0".to_string() },
            football_iq: AttrItem {value: "2.0".to_string() },
            temperament: AttrItem {value: "2.0".to_string() },
            angle_of_view: 3,
        };

        let msg = InstantiateMsg { first_name: "Power".to_string(), last_name: "Mack".to_string(),
            position: Positions::CB1, attributes: attribs,
            managing_contract_address: Addr::unchecked("".to_string())
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        //should generate an error due to no managing_contract_address, let's unwrap_err here
        let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        match err {
            ContractError::PlayerNameCheckError { .. } => {
                assert!(true)
            },

            _ => {
                assert!(false)
            }
        }

    }


}
