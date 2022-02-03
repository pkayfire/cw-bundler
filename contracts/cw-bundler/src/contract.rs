#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw_storage_plus::Map;

use cw721_base::contract::{
    _transfer_nft as cw721_transfer_nft, execute_mint as cw721_execute_mint,
    instantiate as cw721_instantiate,
};
use cw721_base::msg::InstantiateMsg;

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
    pub address: Addr,
    pub amount: u128,
}
const CW20Bundle: Map<u128, Vec<CW20Wrapper>> = Map::new("cw20_bundle");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CW721Wrapper {
    pub address: Addr,
    pub token_id: String,
}
const CW721Bundle: Map<String, Vec<CW721Wrapper>> = Map::new("cw721_bundle");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CW1155Wrapper {
    pub address: Addr,
    pub token_id: u128,
    pub amount: u128,
}
const CW1155Bundle: Map<u128, Vec<CW1155Wrapper>> = Map::new("cw1155_bundle");

const BUNDLE_MAPPING: Map<u128, u128> = Map::new("bundle_mapping");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    cw721_instantiate(deps, _env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => try_increment(deps),
        ExecuteMsg::Reset { count } => try_reset(deps, info, count),
        ExecuteMsg::Mint(msg) => mint(deps, env, info, msg),
        ExecuteMsg::DepositCW20 {} => deposit_cw20(deps),
        ExecuteMsg::DepositCW721 {
            token_id,
            bundle_id,
        } => deposit_cw721(deps, env, info, token_id, bundle_id),
        ExecuteMsg::DepositCW1155 {} => deposit_cw1155(deps),
        ExecuteMsg::Withdraw { bundle_id } => withdraw(deps, env, info, bundle_id),
    }
}

pub fn mint(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MintMsg,
) -> Result<Response, ContractError> {
    cw721_execute_mint(deps.branch(), env, info, msg.base.clone())?;

    // STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
    //     state.count += 1;
    //     Ok(state)
    // })?;

    Ok(Response::default())
}

pub fn withdraw(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bundle_id: String,
) -> Result<Response, ContractError> {
    let bundle = CW721Bundle.may_load(deps.storage, bundle_id)?;
    if let Some(mut i) = bundle {
        while let Some(asset) = i.pop() {
            cw721_transfer_nft(
                deps.branch(),
                &env,
                &info,
                &info.sender.to_string(),
                &asset.token_id,
            )?;
        }
    }

    Ok(Response::new().add_attribute("method", "withdraw"))
}

pub fn deposit_cw20(deps: DepsMut) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn deposit_cw721(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    bundle_id: String,
) -> Result<Response, ContractError> {
    cw721_transfer_nft(
        deps.branch(),
        &env,
        &info,
        &env.contract.address.to_string(),
        &token_id,
    )?;

    let bundle = CW721Bundle.may_load(deps.storage, bundle_id.clone())?;

    if let Some(mut i) = bundle {
        i.push(CW721Wrapper {
            address: info.sender,
            token_id,
        });
    } else {
        let vector = vec![CW721Wrapper {
            address: info.sender,
            token_id,
        }];
        CW721Bundle.save(deps.storage, bundle_id, &vector)?;
    }

    Ok(Response::new().add_attribute("method", "deposit_cw721"))
}

pub fn deposit_cw1155(deps: DepsMut) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.count += 1;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_increment"))
}
pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "reset"))
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
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, Uint128};
    use cw721_base::msg::MintMsg as Cw721MintMsg;
    use cw721_base::state::num_tokens;

    const TOKEN_ID: &str = "123";
    const MINTER: &str = "minter_address";
    const ALICE: &str = "alice_address";
    const BOB: &str = "bob_address";

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {
            name: "Cosmic Apes".into(),
            symbol: "APE".into(),
            minter: MINTER.into(),
        };
        let info = mock_info(MINTER, &[]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
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

        let mint_msg = ExecuteMsg::Mint(MintMsg {
            base: Cw721MintMsg {
                token_id: TOKEN_ID.into(),
                owner: ALICE.into(),
                name: "Cosmic Ape 123".into(),
                description: Some("The first Cosmic Ape".into()),
                image: None,
            },
        });

        let info = mock_info(MINTER, &[]);
        let _ = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();

        // ensure num tokens increases
        let count = num_tokens(&deps.storage).unwrap();
        assert_eq!(1, count);
    }

    #[test]
    fn deposit_cw721() {
        let mut deps = mock_dependencies(&[]);
        setup_contract(deps.as_mut());

        let mint_msg = ExecuteMsg::Mint(MintMsg {
            base: Cw721MintMsg {
                token_id: TOKEN_ID.into(),
                owner: ALICE.into(),
                name: "Cosmic Ape 123".into(),
                description: Some("The first Cosmic Ape".into()),
                image: None,
            },
        });

        let info = mock_info(MINTER, &[]);
        let _ = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();

        // ensure num tokens increases
        let count = num_tokens(&deps.storage).unwrap();
        assert_eq!(1, count);
    }
}
