use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use shared::query_response_info::NameResponse;
use team::msg::{ExecuteMsg, InstantiateTeamMsg, PlayerResponse, PlayersResponse, QueryMsg};
use team::state::State;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateTeamMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(State), &out_dir);
    export_schema(&schema_for!(PlayersResponse), &out_dir);
    export_schema(&schema_for!(PlayerResponse), &out_dir);
    export_schema(&schema_for!(NameResponse), &out_dir);

}
