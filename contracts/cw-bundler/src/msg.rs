use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw20::Cw20ReceiveMsg;

use cw721::Cw721ReceiveMsg;
use cw721_base::msg::MintMsg as Cw721MintMsg;
use cw721_base::Extension;

use cw1155::Cw1155ReceiveMsg;

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
pub enum ReceiveMsg {
    Cw20ReceiveMsg(Cw20ReceiveMsg),
    Cw721ReceiveMsg(Cw721ReceiveMsg),
    Cw1155ReceiveMsg(Cw1155ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    DepositCw20 { bundle_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint(MintMsg),
    Receive(ReceiveMsg),
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
