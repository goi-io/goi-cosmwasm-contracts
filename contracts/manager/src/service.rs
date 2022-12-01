use std::fmt;

use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage};
use cw_controllers::Admin;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use shared::manage::{Management, ManagementFee, ManagedContract, ManagedStatus};
use shared::utils::general::AssetTypes;


use crate::error::ManagementError;

pub struct ManagementService<'a>(Item<'a, Management>);

impl<'a> ManagementService<'a> {
    pub const fn new(storage_key: &'a str) -> Self {
        ManagementService(Item::new(storage_key))
    }

    pub fn save(&self, storage: &mut dyn Storage, management: &Management) -> StdResult<()> {
        self.0.save(storage, &management)
    }

    pub fn get(&self, deps: Deps) -> StdResult<Management> {
        self.0.load(deps.storage)
    }

    pub fn update_fees<C>(
        &self,
        deps: DepsMut,
        _: MessageInfo,
        _env: Env,
        add: Option<Vec<ManagementFee>>,
        remove: Option<Vec<i32>>,
    ) -> Result<Response<C>, ManagementError>
        where
            C: Clone + fmt::Debug + PartialEq + JsonSchema,
    {

        let mut current_state = self.get(deps.as_ref())?;
        let mut updates =
            self.remove_fees(remove,
                             current_state.fees.clone());

        updates = self.add_fees(_env.block.height, add, updates.clone());

        current_state.fees.clear();
        for a_fee in updates {
            current_state.fees.push(a_fee);
        }

        self.save(deps.storage, &current_state.clone()).expect("Failed to update fees.");

        Ok(Response::new())

    }

    fn remove_fees(&self, ids_of_items_to_remove: Option<Vec<i32>>,
                   current_fees: Vec<ManagementFee> ) -> Vec<ManagementFee> {
        match ids_of_items_to_remove {
            None => current_fees,
            Some(r) => {
                let mut position = 0;
                let mut fees_minus_removed_items = current_fees.clone();
                for item in current_fees.clone(){
                    match r.contains(&item.id) {
                        true => {
                            fees_minus_removed_items.remove(position);

                        },
                        false => ()
                    }
                    position += 1;
                };
                fees_minus_removed_items
            }
        }
    }

    fn add_fees(&self, block_height: u64,  items_to_add: Option<Vec<ManagementFee>>,
                mut current_fees: Vec<ManagementFee> ) -> Vec<ManagementFee> {
        match items_to_add {
            None => current_fees,
            Some(a) => {
                let mut current_count = current_fees.len();

                for item in a {
                    current_count += 1;
                    current_fees.push(ManagementFee{
                        id: current_count as i32,
                        created_at_block_height: block_height,
                        active: item.active,
                        fees: item.fees,
                    })
                }
                current_fees
            }
        }
    }

}
