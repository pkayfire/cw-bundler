#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo,
    Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Map;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use cw721::Cw721ExecuteMsg;
use cw721_base::msg::{
    ExecuteMsg as cw721_execute_msg, InstantiateMsg, QueryMsg as cw721_query_msg,
};
use cw721_base::{Cw721Contract, Extension};

use cw1155::{Cw1155ExecuteMsg, Cw1155ReceiveMsg};

use crate::error::ContractError;
use crate::msg::MintMsg;
use crate::msg::{Cw20HookMsg, ExecuteMsg, QueryMsg, ReceiveMsg};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-bundler";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CW20Wrapper {
    pub contract_address: Addr,
    pub amount: Uint128,
}
const CW20_BUNDLE: Map<String, Vec<CW20Wrapper>> = Map::new("cw20_bundle");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CW721Wrapper {
    pub contract_address: Addr,
    pub token_id: String,
}
const CW721_BUNDLE: Map<String, Vec<CW721Wrapper>> = Map::new("cw721_bundle");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CW1155Wrapper {
    pub contract_address: Addr,
    pub token_id: String,
    pub amount: Uint128,
}
const CW1155_BUNDLE: Map<String, Vec<CW1155Wrapper>> = Map::new("cw1155_bundle");

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
        ExecuteMsg::DepositCW20 {
            contract_address,
            amount,
            bundle_id,
        } => deposit_cw20(deps, env, info, contract_address, amount, bundle_id),
        ExecuteMsg::DepositCW721 {
            contract_address,
            token_id,
            bundle_id,
        } => deposit_cw721(deps, env, contract_address, token_id, bundle_id),
        ExecuteMsg::DepositCW1155 {
            contract_address,
            amount,
            token_id,
            bundle_id,
        } => deposit_cw1155(
            deps,
            env,
            info,
            amount,
            contract_address,
            token_id,
            bundle_id,
        ),
        ExecuteMsg::Withdraw { bundle_id } => withdraw(deps, info, bundle_id),
        ExecuteMsg::Receive(msg) => match msg {
            ReceiveMsg::Cw20ReceiveMsg(msg) => receive_cw20(deps, env, info, msg),
            ReceiveMsg::Cw721ReceiveMsg(msg) => receive_cw721(deps, env, info, msg),
            ReceiveMsg::Cw1155ReceiveMsg(msg) => receive_cw1155(deps, env, info, msg),
        },
    }
}

pub fn receive_cw20(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg = from_binary(&cw20_msg.msg);
    match msg {
        Ok(Cw20HookMsg::DepositCw20 { bundle_id }) => {
            let bundle = CW20_BUNDLE.may_load(deps.storage, bundle_id.clone())?;
            if let Some(mut i) = bundle {
                i.push(CW20Wrapper {
                    contract_address: Addr::unchecked(cw20_msg.sender), // check later
                    amount: cw20_msg.amount,
                });
            } else {
                let vector = vec![CW20Wrapper {
                    contract_address: Addr::unchecked(cw20_msg.sender), // check later
                    amount: cw20_msg.amount,
                }];
                CW20_BUNDLE.save(deps.storage, bundle_id, &vector)?;
            }
            Ok(Response::new().add_attribute("method", "receive_cw20"))
        }
        _ => Err(ContractError::MissingReceiveHook {}),
    }
}

pub fn receive_cw721(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

pub fn receive_cw1155(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw1155ReceiveMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
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

pub fn deposit_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract_address: String,
    amount: Uint128,
    bundle_id: String,
) -> Result<Response, ContractError> {
    let transfer_cw20_msg = Cw20ExecuteMsg::TransferFrom {
        owner: info.sender.to_string(),
        recipient: env.contract.address.to_string().clone(),
        amount,
    };
    let exec_cw20_transfer = WasmMsg::Execute {
        contract_addr: contract_address.clone(),
        msg: to_binary(&transfer_cw20_msg)?,
        funds: vec![],
    };
    let cw20_transfer_cosmos_msg: CosmosMsg = exec_cw20_transfer.into();

    let bundle = CW20_BUNDLE.may_load(deps.storage, bundle_id.clone())?;
    if let Some(mut i) = bundle {
        i.push(CW20Wrapper {
            contract_address: Addr::unchecked(contract_address),
            amount,
        });
    } else {
        let vector = vec![CW20Wrapper {
            contract_address: Addr::unchecked(contract_address),
            amount,
        }];
        CW20_BUNDLE.save(deps.storage, bundle_id, &vector)?;
    }

    Ok(Response::new()
        .add_message(cw20_transfer_cosmos_msg)
        .add_attribute("method", "deposit_cw20"))
}

pub fn deposit_cw721(
    deps: DepsMut,
    env: Env,
    contract_address: String,
    token_id: String,
    bundle_id: String,
) -> Result<Response, ContractError> {
    let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
        recipient: env.contract.address.to_string().clone(),
        token_id: token_id.clone(),
    };
    let exec_cw721_transfer = WasmMsg::Execute {
        contract_addr: contract_address.clone(),
        msg: to_binary(&transfer_cw721_msg)?,
        funds: vec![],
    };
    let cw721_transfer_cosmos_msg: CosmosMsg = exec_cw721_transfer.into();

    let bundle = CW721_BUNDLE.may_load(deps.storage, bundle_id.clone())?;
    if let Some(mut i) = bundle {
        i.push(CW721Wrapper {
            contract_address: Addr::unchecked(contract_address),
            token_id,
        });
    } else {
        let vector = vec![CW721Wrapper {
            contract_address: Addr::unchecked(contract_address),
            token_id,
        }];
        CW721_BUNDLE.save(deps.storage, bundle_id, &vector)?;
    }

    Ok(Response::new()
        .add_message(cw721_transfer_cosmos_msg)
        .add_attribute("method", "deposit_cw721"))
}

pub fn deposit_cw1155(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    contract_address: String,
    token_id: String,
    bundle_id: String,
) -> Result<Response, ContractError> {
    let transfer_cw1155_msg = Cw1155ExecuteMsg::SendFrom {
        from: info.sender.to_string(),
        to: env.contract.address.to_string().clone(),
        token_id: token_id.clone(),
        value: amount,
        msg: None,
    };
    let exec_cw1155_transfer = WasmMsg::Execute {
        contract_addr: contract_address.clone(),
        msg: to_binary(&transfer_cw1155_msg)?,
        funds: vec![],
    };
    let cw1155_transfer_cosmos_msg: CosmosMsg = exec_cw1155_transfer.into();

    let bundle = CW1155_BUNDLE.may_load(deps.storage, bundle_id.clone())?;
    if let Some(mut i) = bundle {
        i.push(CW1155Wrapper {
            contract_address: Addr::unchecked(contract_address),
            amount,
            token_id,
        });
    } else {
        let vector = vec![CW1155Wrapper {
            contract_address: Addr::unchecked(contract_address),
            amount,
            token_id,
        }];
        CW1155_BUNDLE.save(deps.storage, bundle_id, &vector)?;
    }

    Ok(Response::new()
        .add_message(cw1155_transfer_cosmos_msg)
        .add_attribute("method", "deposit_cw1155"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Tokens {
            owner,
            start_after,
            limit,
        } => query_tokens(
            deps,
            _env,
            cw721_query_msg::Tokens {
                owner: owner,
                start_after: start_after,
                limit: limit,
            },
        ),
        QueryMsg::NftInfo { token_id } => {
            query_nft_info(deps, _env, cw721_query_msg::NftInfo { token_id: token_id })
        }
    }
}

fn query_nft_info(deps: Deps, _env: Env, msg: cw721_query_msg) -> StdResult<Binary> {
    let response = Cw721Contract::<Extension, Empty>::default().query(deps, _env, msg)?;
    Ok(response)
}

fn query_tokens(deps: Deps, _env: Env, msg: cw721_query_msg) -> StdResult<Binary> {
    let response = Cw721Contract::<Extension, Empty>::default().query(deps, _env, msg)?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw721::NumTokensResponse;
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
    fn try_deposit_cw20() {
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

        // deposit cw20 token
        let res = deposit_cw20(
            deps.as_mut(),
            mock_env(),
            "cw20_contract_address".to_string(),
            Uint128::from(1000u128),
            TOKEN_ID.into(),
        )
        .unwrap();

        let transfer_cw20_msg = Cw20ExecuteMsg::Transfer {
            recipient: "cosmos2contract".to_string(),
            amount: Uint128::from(1000u128),
        };
        let exec_cw20_transfer = WasmMsg::Execute {
            contract_addr: "cw20_contract_address".to_string(),
            msg: to_binary(&transfer_cw20_msg).unwrap(),
            funds: vec![],
        };
        let cw20_transfer_cosmos_msg: CosmosMsg = exec_cw20_transfer.into();

        assert_eq!(
            res,
            Response::new()
                .add_message(cw20_transfer_cosmos_msg)
                .add_attribute("method", "deposit_cw20")
        );
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
        let res = deposit_cw721(
            deps.as_mut(),
            mock_env(),
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

    #[test]
    fn try_deposit_cw1155() {
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

        // deposit cw1155 token
        let info = mock_info(MINTER, &[]);
        let res = deposit_cw1155(
            deps.as_mut(),
            mock_env(),
            info,
            Uint128::from(1000u128),
            "cw1155_contract_address".to_string(),
            "CW1155_1".to_string(),
            TOKEN_ID.into(),
        )
        .unwrap();

        let info = mock_info(MINTER, &[]);
        let transfer_cw1155_msg = Cw1155ExecuteMsg::SendFrom {
            from: info.sender.to_string(),
            to: mock_env().contract.address.to_string(),
            token_id: "CW1155_1".to_string(),
            value: Uint128::from(1000u128),
            msg: None,
        };
        let exec_cw1155_transfer = WasmMsg::Execute {
            contract_addr: "cw1155_contract_address".to_string(),
            msg: to_binary(&transfer_cw1155_msg).unwrap(),
            funds: vec![],
        };
        let cw1155_transfer_cosmos_msg: CosmosMsg = exec_cw1155_transfer.into();

        assert_eq!(
            res,
            Response::new()
                .add_message(cw1155_transfer_cosmos_msg)
                .add_attribute("method", "deposit_cw1155")
        );
    }
}
