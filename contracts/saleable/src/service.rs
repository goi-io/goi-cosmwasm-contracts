use std::ops::{Mul, Sub};

use cosmwasm_std::{Addr, attr, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, ReplyOn, Response, StdResult, Storage, SubMsg, to_binary};
use cw4::Member;
use cw4_group::contract::execute_update_members;
use cw_controllers::Admin;
use cw_storage_plus::Item;
use shared::manage::{ Manageable };
use shared::saleable::{DistributionPacket, DistributionType, Saleable};
use shared::utils::Fee;


use crate::coin_helpers::assert_sent_sufficient_coin;
use crate::error::SaleableItemError;

// state/logic
pub struct SaleableService<'a>(Item<'a, Saleable>);

impl<'a> SaleableService<'a> {
    pub const fn new(storage_key: &'a str) -> Self {
        SaleableService(Item::new(storage_key))
    }

    pub fn save(&self, storage: &mut dyn Storage, saleable: Saleable) -> StdResult<()> {
        self.0.save(storage, &saleable)
    }

    pub fn get(&self, deps: Deps) -> StdResult<Saleable> {
        self.0.load(deps.storage)
    }

    pub fn execute_update(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        for_sale_status: bool,
        price: Option<Coin>,
        manageable: Manageable
    ) -> Result<Response, SaleableItemError>
        {

            match for_sale_status  {
                true => {
                    let current_state = self.get(deps.as_ref())?;
                    match current_state.price.is_none() && price.is_none() {
                        true => Err(SaleableItemError::PriceNotSet {}  ),
                        false => {
                            Ok(self.update_for_sale_state(deps, for_sale_status, price.clone(), manageable)?)
                        }

                    }
                }
                false =>  Ok(self.update_for_sale_state(deps, for_sale_status, price.clone(), manageable)?)
            }

        }



    fn validate_price (&self, price: Option<Coin>) -> Result<Option<Coin>, SaleableItemError> {
        match price.clone() {
            Some(c) => {
                match c.amount.u128() > 0u128  {
                    true => Ok(Some(c)),
                    false => Err(SaleableItemError::InvalidPrice{}  )
                }
            },
            _ => Ok(None),
        }
    }

    pub fn update_for_sale_state
        (&self, deps: DepsMut, for_sale_status: bool,  a_new_price: Option<Coin>, _ : Manageable)
                -> Result<Response, SaleableItemError>  {

        let current_state = self.get(deps.as_ref())?;
        let f_price = {
            match (current_state.price, a_new_price) {
                (None, None) => None,
                (_, Some(new_price)) => self.validate_price( Some (new_price))?,
                (Some(current_price), None) => Some(current_price),
            }
        };
        let new_state =
            Saleable{
                price_version: (current_state.price_version + 1),
                price: f_price,
                for_sale: for_sale_status,
            };
        self.save(deps.storage, new_state.clone()).expect("Problem updating 'for sale' status.");

        let attributes = vec![
                attr("for_sale_status_updated", match new_state.for_sale { true => "true".to_string(), _ => "false".to_string()} )
            ];     
         
        Ok(Response::new()
            .add_attributes(attributes.clone()))            

    }

    fn process_payment_distribution(&self,
                                    payees: Vec<Member>,
                                    fees: Option<Vec<Fee>>, balance: Coin )
        -> Result<Option<Vec<SubMsg>>, SaleableItemError> {

        if balance.amount.is_zero() {
            return Err(SaleableItemError::EmptyBalance {} );
        }

        let total_ownership: u64 =
            payees.iter()
            .map( |x| x.weight)
            .sum();

        match total_ownership {
            100 => (),
            _ =>  return Err(SaleableItemError::OwnershipRequirementNotMet {})
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

    pub fn execute_buy(&self,
                       deps: Deps,
                       info: MessageInfo,
                       current_owners: Vec<Member>,
                       fees: Option<Vec<Fee>>,
    ) -> Result<Response,  SaleableItemError> {
        let state = self.get(deps)?;
        match state.for_sale  {
            true => {
                assert_sent_sufficient_coin(&info.funds, state.price.clone())?;
                let bank_transfer_sub_messages =
                    self.process_payment_distribution
                    (current_owners, fees, state.price.clone().unwrap())?.unwrap();


                let attributes = vec![
                    attr("amount_paid", state.price.unwrap().amount),
                    attr("new_owner", info.sender),
                    attr("action", "execute_buy"),
                ];

                let res = Response::new()
                    .add_attributes(attributes)
                    .add_submessages(bank_transfer_sub_messages);

                Ok(res)

            },
            false => return Err(SaleableItemError::NotForSale {} )
        }

    }


    pub fn transfer_ownership(mut deps: DepsMut, env: Env, info: MessageInfo,
                              current_owner_members: Vec<Member>, admin: &Admin) -> Result<Response, SaleableItemError> {

        let sender = info.sender.clone();
        let remove_current_owners: Vec<String> = current_owner_members.into_iter()
            .map(|i| i.addr)
            .collect();

        admin.set(deps.branch(), Some(Addr::unchecked(info.sender.clone()))).expect("Failed to transfer ownership.");

        let res = execute_update_members(deps, env, info.clone(),
                                         vec![Member { addr: sender.to_string(), weight: 100 }],
                                         remove_current_owners);
        match res {
            Ok(r) => Ok(r),
            Err(e) => match e {
                cw4_group::ContractError::Admin(_) => Err(SaleableItemError::NotAnAdmin {}),
                cw4_group::ContractError::Unauthorized { .. } => Err(SaleableItemError::Unauthorized {}),
                _ => Err(SaleableItemError::FailureTransferingOwnership {})
            }
        }

    }


}
