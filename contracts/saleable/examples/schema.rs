/*

use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use saleable::msg::{ForSaleInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use saleable::state::Saleable;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema_hold");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Saleable), &out_dir);
    export_schema(&schema_for!(ForSaleInfoResponse), &out_dir);
}

 */
fn main() {}
