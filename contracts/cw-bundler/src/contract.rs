#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Response,
    StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Map;

use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};
use cw721_base::msg::{
    ExecuteMsg as cw721_execute_msg, InstantiateMsg, QueryMsg as cw721_query_msg,
};
use cw721_base::{Cw721Contract, Extension};

use crate::error::ContractError;
use crate::msg::MintMsg;
use crate::msg::{CountResponse, ExecuteMsg, QueryMsg};
use crate::state::{State, STATE};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-bundler";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CW20Wrapper {
    pub contract_address: Addr,
    pub amount: u128,
}
const CW20_BUNDLE: Map<u128, Vec<CW20Wrapper>> = Map::new("cw20_bundle");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CW721Wrapper {
    pub contract_address: Addr,
    pub token_id: String,
}
const CW721_BUNDLE: Map<String, Vec<CW721Wrapper>> = Map::new("cw721_bundle");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CW1155Wrapper {
    pub contract_address: Addr,
    pub token_id: u128,
    pub amount: u128,
}
const CW1155_BUNDLE: Map<u128, Vec<CW1155Wrapper>> = Map::new("cw1155_bundle");

const BUNDLE_MAPPING: Map<u128, u128> = Map::new("bundle_mapping");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Cw721Contract::<Extension, Empty>::default().instantiate(deps, _env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint(msg) => mint(deps, env, info, msg),
        ExecuteMsg::DepositCW20 {} => deposit_cw20(deps),
        ExecuteMsg::DepositCW721 {
            contract_address,
            token_id,
            bundle_id,
        } => deposit_cw721(deps, env, info, contract_address, token_id, bundle_id),
        ExecuteMsg::DepositCW1155 {} => deposit_cw1155(deps),
        ExecuteMsg::Withdraw { bundle_id } => withdraw(deps, info, bundle_id),
    }
}

pub fn mint(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MintMsg,
) -> Result<Response, ContractError> {
    let mint_msg = cw721_execute_msg::Mint(msg.base.clone());
    Cw721Contract::<Extension, Empty>::default().execute(deps.branch(), env, info, mint_msg)?;
    Ok(Response::default())
}

pub fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
    bundle_id: String,
) -> Result<Response, ContractError> {
    let bundle = CW721_BUNDLE.may_load(deps.storage, bundle_id)?;
    let mut cw721_transfer_cosmos_msgs = vec![];

    if let Some(mut i) = bundle {
        while let Some(asset) = i.pop() {
            let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
                recipient: info.sender.to_string(),
                token_id: asset.token_id,
            };
            let exec_cw721_transfer = WasmMsg::Execute {
                contract_addr: asset.contract_address.to_string(),
                msg: to_binary(&transfer_cw721_msg)?,
                funds: vec![],
            };
            let cw721_transfer_cosmos_msg: CosmosMsg = exec_cw721_transfer.into();
            cw721_transfer_cosmos_msgs.push(cw721_transfer_cosmos_msg);
        }
    }

    Ok(Response::new()
        .add_messages(cw721_transfer_cosmos_msgs)
        .add_attribute("method", "withdraw"))
}

pub fn deposit_cw20(deps: DepsMut) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn deposit_cw721(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract_address: String,
    token_id: String,
    bundle_id: String,
) -> Result<Response, ContractError> {
    let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
        recipient: env.contract.address.to_string().clone(),
        token_id: token_id.clone(),
        //msg: to_binary("deposit_cw721").unwrap(),
    };
    let exec_cw721_transfer = WasmMsg::Execute {
        contract_addr: contract_address,
        msg: to_binary(&transfer_cw721_msg)?,
        funds: vec![],
    };
    let cw721_transfer_cosmos_msg: CosmosMsg = exec_cw721_transfer.into();

    let bundle = CW721_BUNDLE.may_load(deps.storage, bundle_id.clone())?;
    if let Some(mut i) = bundle {
        i.push(CW721Wrapper {
            contract_address: info.sender, // fix
            token_id,
        });
    } else {
        let vector = vec![CW721Wrapper {
            contract_address: info.sender, // fix
            token_id,
        }];
        CW721_BUNDLE.save(deps.storage, bundle_id, &vector)?;
    }

    Ok(Response::new()
        .add_message(cw721_transfer_cosmos_msg)
        .add_attribute("method", "deposit_cw721"))
}

pub fn deposit_cw1155(deps: DepsMut) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse { count: state.count })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw721::NumTokensResponse;
    use cw721::OwnerOfResponse;
    use cw721_base::msg::MintMsg as Cw721MintMsg;
    use cw721_base::{Cw721Contract, Extension};

    const TOKEN_ID: &str = "123";
    const MINTER: &str = "minter_address";
    const ALICE: &str = "alice_address";

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {
            name: "CW Bundled Asset".into(),
            symbol: "CWBUNDLE".into(),
            minter: MINTER.into(),
        };
        let info = mock_info(MINTER, &[]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    fn setup_cw721_contract(deps: DepsMut<'_>) -> Cw721Contract<'static, Extension, Empty> {
        let contract = Cw721Contract::default();
        let msg = InstantiateMsg {
            name: "CW721 Token".to_string(),
            symbol: "CW721".to_string(),
            minter: MINTER.into(),
        };
        let info = mock_info(MINTER, &[]);
        let mut env = mock_env();
        env.contract.address = Addr::unchecked("cw721_contract_address");
        let res = contract.instantiate(deps, env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        contract
    }

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies(&[]);
        setup_contract(deps.as_mut());
    }

    #[test]
    fn mint() {
        let mut deps = mock_dependencies(&[]);
        setup_contract(deps.as_mut());

        let info = mock_info(MINTER, &[]);

        let mint_msg = cw721_execute_msg::Mint(Cw721MintMsg {
            token_id: TOKEN_ID.into(),
            owner: ALICE.into(),
            extension: None,
            token_uri: Some("ipfs://QmVKZ5YZYDqdnAQo93kaYbcMtzGgx9kvpAVwoERm5mZezh".to_string()),
        });
        let _ = Cw721Contract::<Extension, Empty>::default().execute(
            deps.as_mut(),
            mock_env(),
            info,
            mint_msg,
        );

        // ensure num tokens increases
        let count = Cw721Contract::<Extension, Empty>::default()
            .query(deps.as_ref(), mock_env(), cw721_query_msg::NumTokens {})
            .unwrap();
        let response: NumTokensResponse = from_binary(&count).unwrap();
        assert_eq!(1, response.count);
    }

    #[test]
    fn try_deposit_cw721() {
        let mut deps = mock_dependencies(&[]);
        setup_contract(deps.as_mut());

        let info = mock_info(MINTER, &[]);
        let mint_msg = cw721_execute_msg::Mint(Cw721MintMsg {
            token_id: TOKEN_ID.into(),
            owner: ALICE.into(),
            extension: None,
            token_uri: Some("ipfs://QmVKZ5YZYDqdnAQo93kaYbcMtzGgx9kvpAVwoERm5mZezh".to_string()),
        });
        let _ = Cw721Contract::<Extension, Empty>::default().execute(
            deps.as_mut(),
            mock_env(),
            info,
            mint_msg,
        );

        // ensure num tokens increases
        let count = Cw721Contract::<Extension, Empty>::default()
            .query(deps.as_ref(), mock_env(), cw721_query_msg::NumTokens {})
            .unwrap();
        let response: NumTokensResponse = from_binary(&count).unwrap();
        assert_eq!(1, response.count);

        // set up dummy cw721 contract
        let mut env = mock_env();
        env.contract.address = Addr::unchecked("cw721_contract_address");
        let mut deps = mock_dependencies(&[]);
        let contract = setup_cw721_contract(deps.as_mut());

        let mint_msg = cw721_execute_msg::Mint(cw721_base::MintMsg {
            token_id: "CW721_1".into(),
            owner: ALICE.into(),
            extension: None,
            token_uri: Some("ipfs://QmVKZ5YZYDqdnAQo93kaYbcMtzGgx9kvpAVwoERm5mZezh".to_string()),
        });

        let minter = mock_info(MINTER, &[]);
        contract
            .execute(deps.as_mut(), env, minter, mint_msg)
            .unwrap();

        // ensure num tokens is 1
        let mut env = mock_env();
        env.contract.address = Addr::unchecked("cw721_contract_address");
        let count = contract
            .query(deps.as_ref(), env, cw721_query_msg::NumTokens {})
            .unwrap();
        let response: NumTokensResponse = from_binary(&count).unwrap();
        assert_eq!(1, response.count);

        // deposit cw721 token
        let info = mock_info(ALICE, &[]);
        let res = deposit_cw721(
            deps.as_mut(),
            mock_env(),
            info,
            "cw721_contract_address".to_string(),
            "CW721_1".to_string(),
            TOKEN_ID.into(),
        )
        .unwrap();

        let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
            recipient: "cosmos2contract".to_string(),
            token_id: "CW721_1".to_string(),
        };
        let exec_cw721_transfer = WasmMsg::Execute {
            contract_addr: "cw721_contract_address".to_string(),
            msg: to_binary(&transfer_cw721_msg).unwrap(),
            funds: vec![],
        };
        let cw721_transfer_cosmos_msg: CosmosMsg = exec_cw721_transfer.into();

        assert_eq!(
            res,
            Response::new()
                .add_message(cw721_transfer_cosmos_msg)
                .add_attribute("method", "deposit_cw721")
        );

        // ensure num tokens in bundle is 1
        let bundle = CW721_BUNDLE
            .may_load(&deps.storage, TOKEN_ID.into())
            .unwrap()
            .unwrap();
        assert_eq!(1, bundle.len());
    }
}
