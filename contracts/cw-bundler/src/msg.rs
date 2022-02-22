use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw20::Cw20ExecuteMsg;
use cw20::Cw20ReceiveMsg;

use cw721::Cw721ReceiveMsg;
use cw721_base::msg::MintMsg as Cw721MintMsg;
use cw721_base::Extension;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintMsg {
    pub base: Cw721MintMsg<Extension>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub enum ReceiveMsg {
//     DepositCw721 { bundle_id: String },
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint(MintMsg),
    Receive(Cw20ReceiveMsg),
    ReceiveNft(Cw721ReceiveMsg),
    Withdraw { bundle_id: String },
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
