use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult};
use cw0::maybe_addr;
use cw4::{Member, MemberListResponse};
use cw4_group::contract::create as cw4Create;
use cw4_group::state::MEMBERS;
use cw_storage_plus::{Bound, SnapshotMap};

use crate::GroupAdminError;

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn list_members(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    members: SnapshotMap<&Addr, u64>
) -> StdResult<MemberListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let addr = maybe_addr(deps.api, start_after)?;
    let start =
                                         match &addr {
                                            Some(ad) =>  Some(Bound::exclusive(ad)),
                                            None => None,
                                        };



    let members: StdResult<Vec<_>> = members
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (key, weight) = item?;
            Ok(Member {
                addr: key.to_string(),
                weight,
            })
        })
        .collect();

    Ok(MemberListResponse { members: members? })
}


pub fn validate_owner_count(deps: Deps, remove: Option<Vec<String>>, add: Option<Vec<Member>>)
                        -> Result<(), GroupAdminError> {
    let (remove_count, add_count) =
        match (remove, add) {
            (Some(r), Some(a)) => (r.len(), a.len()),
            (Some(r), None) => (r.len(), 0),
            (None, Some(a)) => (0, a.len()),
            (None, None) => (0, 0)
        };
    let members = list_members(deps, None, None, MEMBERS).unwrap();
    let current_owners_total:usize = members.members.len() as usize;
    let new_total: usize = current_owners_total - remove_count + add_count;
    match new_total <= 10 && new_total > 0 {
        true => Ok(()),
        false => return Err(GroupAdminError::OwnershipMembersRequirementNotMet
        { message: "Owner count requirement not met. Minimum of 1; maximum of 10".to_string()})
    }
}

pub fn initialize_members_and_admin (deps: DepsMut, _env: Env, info: MessageInfo,
                                     admin: Option<String>, members: Vec<Member>)
                                     -> Result<Response, GroupAdminError> {
    match cw4Create(deps, admin.clone(), members, _env.block.height) {
        Ok(_) =>
            Ok(Response::new()
                .add_attribute("method", "instantiate")
                .add_attribute("owner", info.sender.clone())
                .add_attribute("admin", match admin
                { Some(a) => a, None => "ADMIN NOT SET".to_string()})),
        Err(e) => {
            Err(GroupAdminError::GroupError (e))
        }
    }
}
