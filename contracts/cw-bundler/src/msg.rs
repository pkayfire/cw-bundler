use cosmwasm_std::Uint128;
use cw721_base::msg::MintMsg as Cw721MintMsg;
use cw721_base::Extension;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintMsg {
    pub base: Cw721MintMsg<Extension>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint(MintMsg),
    DepositCW20 {
        amount: Uint128,
        bundle_id: String,
        contract_address: String,
    },
    DepositCW721 {
        token_id: String,
        bundle_id: String,
        contract_address: String,
    },
    DepositCW1155 {
        amount: Uint128,
        token_id: String,
        bundle_id: String,
        contract_address: String,
    },
    Withdraw {
        bundle_id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    NftInfo {
        token_id: String,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}
