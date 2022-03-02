#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Response,
    StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Map;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};
use cw721_base::msg::{
    ExecuteMsg as cw721_execute_msg, InstantiateMsg, QueryMsg as cw721_query_msg,
};
use cw721_base::state::TokenInfo;
use cw721_base::{Cw721Contract, Extension};

use cw1155::{Cw1155BatchReceiveMsg, Cw1155ExecuteMsg};

use crate::error::ContractError;
use crate::msg::MintMsg;
use crate::msg::{ExecuteMsg, QueryMsg};

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

#[derive(Serialize, Deserialize)]
struct DepositCwMsg {
    bundle_id: String,
}

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
        ExecuteMsg::Withdraw { bundle_id } => withdraw(deps, env, info, bundle_id),
        ExecuteMsg::Receive(msg) => receive_cw20(deps, info, msg),
        ExecuteMsg::ReceiveNft(msg) => receive_cw721(deps, info, msg),
        ExecuteMsg::BatchReceive(msg) => receive_cw1155(deps, info, msg),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let deposit_msg_string = msg.msg.to_base64();
    let bytes = base64::decode(deposit_msg_string)?;
    let deposit_msg: DepositCwMsg = serde_json_wasm::from_slice(&bytes)?;
    let bundle_id = deposit_msg.bundle_id;

    let token_info = Cw721Contract::<Extension, Empty>::default()
        .tokens
        .load(deps.storage, &bundle_id)?;
    check_can_deposit(&token_info, msg.sender.clone())?;

    let bundle = CW20_BUNDLE.may_load(deps.storage, bundle_id.clone())?;
    if let Some(mut i) = bundle {
        i.push(CW20Wrapper {
            contract_address: info.sender.clone(),
            amount: msg.amount,
        });
        CW20_BUNDLE.save(deps.storage, bundle_id.clone(), &i)?;
    } else {
        let vector = vec![CW20Wrapper {
            contract_address: info.sender.clone(),
            amount: msg.amount,
        }];
        CW20_BUNDLE.save(deps.storage, bundle_id.clone(), &vector)?;
    }

    Ok(Response::default()
        .add_attribute("action", "deposit_cw20")
        .add_attribute("sender", msg.sender)
        .add_attribute("contract_sender", info.sender.to_string())
        .add_attribute("amount", msg.amount)
        .add_attribute("bundle_id", bundle_id))
}

pub fn receive_cw721(
    deps: DepsMut,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let deposit_msg_string = msg.msg.to_base64();
    let bytes = base64::decode(deposit_msg_string)?;
    let deposit_msg: DepositCwMsg = serde_json_wasm::from_slice(&bytes)?;
    let bundle_id = deposit_msg.bundle_id;

    let token_info = Cw721Contract::<Extension, Empty>::default()
        .tokens
        .load(deps.storage, &bundle_id)?;
    check_can_deposit(&token_info, msg.sender.clone())?;

    let bundle = CW721_BUNDLE.may_load(deps.storage, bundle_id.clone())?;
    if let Some(mut i) = bundle {
        i.push(CW721Wrapper {
            contract_address: info.sender.clone(),
            token_id: msg.token_id.clone(),
        });
        CW721_BUNDLE.save(deps.storage, bundle_id.clone(), &i)?;
    } else {
        let vector = vec![CW721Wrapper {
            contract_address: info.sender.clone(),
            token_id: msg.token_id.clone(),
        }];
        CW721_BUNDLE.save(deps.storage, bundle_id.clone(), &vector)?;
    }

    Ok(Response::default()
        .add_attribute("action", "deposit_cw721")
        .add_attribute("sender", msg.sender)
        .add_attribute("contract_sender", info.sender.to_string())
        .add_attribute("token_id", msg.token_id)
        .add_attribute("bundle_id", bundle_id))
}

pub fn receive_cw1155(
    deps: DepsMut,
    info: MessageInfo,
    mut msg: Cw1155BatchReceiveMsg,
) -> Result<Response, ContractError> {
    let deposit_msg_string = msg.msg.to_base64();
    let bytes = base64::decode(deposit_msg_string)?;
    let deposit_msg: DepositCwMsg = serde_json_wasm::from_slice(&bytes)?;
    let bundle_id = deposit_msg.bundle_id;

    let token_info = Cw721Contract::<Extension, Empty>::default()
        .tokens
        .load(deps.storage, &bundle_id)?;
    check_can_deposit(&token_info, msg.operator.clone())?;

    let bundle = CW1155_BUNDLE.may_load(deps.storage, bundle_id.clone())?;
    if let Some(mut i) = bundle {
        while let Some(asset) = msg.batch.pop() {
            i.push(CW1155Wrapper {
                contract_address: info.sender.clone(),
                token_id: asset.0,
                amount: asset.1,
            });
        }
        CW1155_BUNDLE.save(deps.storage, bundle_id.clone(), &i)?;
    } else {
        let mut vector = vec![];
        while let Some(asset) = msg.batch.pop() {
            vector.push(CW1155Wrapper {
                contract_address: info.sender.clone(),
                token_id: asset.0,
                amount: asset.1,
            });
        }
        CW1155_BUNDLE.save(deps.storage, bundle_id.clone(), &vector)?;
    }

    Ok(Response::default()
        .add_attribute("action", "deposit_cw1155")
        .add_attribute("sender", msg.operator)
        .add_attribute("contract_sender", info.sender.to_string())
        .add_attribute("bundle_id", bundle_id))
}

pub fn mint(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MintMsg,
) -> Result<Response, ContractError> {
    let mint_msg = cw721_execute_msg::Mint(msg.base.clone());
    let res =
        Cw721Contract::<Extension, Empty>::default().execute(deps.branch(), env, info, mint_msg)?;
    Ok(res)
}

pub fn check_can_deposit(
    token: &TokenInfo<Extension>,
    sender: String,
) -> Result<(), ContractError> {
    // only owner can deposit
    if token.owner == sender {
        return Ok(());
    }
    Err(ContractError::Unauthorized {})
}

pub fn check_can_withdraw(
    info: &MessageInfo,
    token: &TokenInfo<Extension>,
) -> Result<(), ContractError> {
    // only owner can withdraw
    if token.owner == info.sender {
        return Ok(());
    }
    Err(ContractError::Unauthorized {})
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bundle_id: String,
) -> Result<Response, ContractError> {
    let token_info = Cw721Contract::<Extension, Empty>::default()
        .tokens
        .load(deps.storage, &bundle_id)?;
    check_can_withdraw(&info, &token_info)?;

    let mut cw_transfer_cosmos_msgs = vec![];
    let bundle = CW721_BUNDLE.may_load(deps.storage, bundle_id.clone())?;
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
            cw_transfer_cosmos_msgs.push(cw721_transfer_cosmos_msg);
        }
        CW721_BUNDLE.save(deps.storage, bundle_id.clone(), &i)?;
    }

    let bundle = CW20_BUNDLE.may_load(deps.storage, bundle_id.clone())?;
    if let Some(mut i) = bundle {
        while let Some(asset) = i.pop() {
            let transfer_cw20_msg = Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount: asset.amount,
            };
            let exec_cw20_transfer = WasmMsg::Execute {
                contract_addr: asset.contract_address.to_string(),
                msg: to_binary(&transfer_cw20_msg)?,
                funds: vec![],
            };
            let cw20_transfer_cosmos_msg: CosmosMsg = exec_cw20_transfer.into();
            cw_transfer_cosmos_msgs.push(cw20_transfer_cosmos_msg);
        }
        CW20_BUNDLE.save(deps.storage, bundle_id.clone(), &i)?;
    }

    let bundle = CW1155_BUNDLE.may_load(deps.storage, bundle_id.clone())?;
    let mut cw1155_batch = vec![];
    if let Some(mut i) = bundle {
        while let Some(asset) = i.pop() {
            cw1155_batch.push((asset.token_id, asset.amount));
        }
        CW1155_BUNDLE.save(deps.storage, bundle_id, &i)?;
        let transfer_cw1155_msg = Cw1155ExecuteMsg::BatchSendFrom {
            from: env.contract.address.to_string(),
            to: info.sender.to_string(),
            batch: cw1155_batch,
            msg: None,
        };
        let exec_cw1155_transfer = WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&transfer_cw1155_msg)?,
            funds: vec![],
        };
        let cw1155_transfer_cosmos_msg: CosmosMsg = exec_cw1155_transfer.into();
        cw_transfer_cosmos_msgs.push(cw1155_transfer_cosmos_msg);
    }

    Ok(Response::new()
        .add_messages(cw_transfer_cosmos_msgs)
        .add_attribute("method", "withdraw"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // add custom queries here
        _ => {
            let msg: cw721_query_msg = msg.into();
            let response = Cw721Contract::<Extension, Empty>::default().query(deps, _env, msg)?;
            Ok(response)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw721::NumTokensResponse;
    use cw721_base::msg::MintMsg as Cw721MintMsg;
    use cw721_base::{Cw721Contract, Extension};

    const TOKEN_ID: &str = "a";
    const MINTER: &str = "minter_address";
    const ALICE: &str = "alice_address";
    const CONTRACT: &str = "contract_address";

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
    fn try_receive_cw20() {
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

        let info = mock_info(CONTRACT, &[]);
        let msg = Cw20ReceiveMsg {
            sender: ALICE.into(),
            amount: Uint128::from(2u128),
            msg: Binary::from_base64(&"eyJidW5kbGVfaWQiOiAiYSJ9".to_string()).unwrap(),
        };

        // receive cw20 tokens
        let res = receive_cw20(deps.as_mut(), info, msg).unwrap();
        let info = mock_info(CONTRACT, &[]);
        let msg = Cw20ReceiveMsg {
            sender: ALICE.into(),
            amount: Uint128::from(2u128),
            msg: Binary::from_base64(&"eyJidW5kbGVfaWQiOiAiYSJ9".to_string()).unwrap(),
        };
        assert_eq!(
            res,
            Response::default()
                .add_attribute("action", "deposit_cw20")
                .add_attribute("sender", msg.sender)
                .add_attribute("contract_sender", info.sender.to_string())
                .add_attribute("amount", "2")
                .add_attribute("bundle_id", "a")
        );

        // ensure num tokens in bundle is 1
        let bundle = CW20_BUNDLE
            .may_load(&deps.storage, "a".into())
            .unwrap()
            .unwrap();
        assert_eq!(1, bundle.len());
    }

    #[test]
    fn try_receive_cw721() {
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

        let info = mock_info(CONTRACT, &[]);
        let msg = Cw721ReceiveMsg {
            sender: ALICE.into(),
            token_id: TOKEN_ID.into(),
            msg: Binary::from_base64(&"eyJidW5kbGVfaWQiOiAiYSJ9".to_string()).unwrap(),
        };

        // receive cw721 token
        let res = receive_cw721(deps.as_mut(), info, msg).unwrap();
        let info = mock_info(CONTRACT, &[]);
        let msg = Cw721ReceiveMsg {
            sender: ALICE.into(),
            token_id: TOKEN_ID.into(),
            msg: Binary::from_base64(&"eyJidW5kbGVfaWQiOiAiYSJ9".to_string()).unwrap(),
        };
        assert_eq!(
            res,
            Response::default()
                .add_attribute("action", "deposit_cw721")
                .add_attribute("sender", msg.sender)
                .add_attribute("contract_sender", info.sender.to_string())
                .add_attribute("token_id", msg.token_id)
                .add_attribute("bundle_id", "a")
        );

        // ensure num tokens in bundle is 1
        let bundle = CW721_BUNDLE
            .may_load(&deps.storage, "a".into())
            .unwrap()
            .unwrap();
        assert_eq!(1, bundle.len());
    }

    #[test]
    fn try_receive_cw1155() {
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

        let info = mock_info(CONTRACT, &[]);
        let msg = Cw1155BatchReceiveMsg {
            operator: ALICE.into(),
            from: None,
            batch: vec![(TOKEN_ID.into(), Uint128::from(2u128))],
            msg: Binary::from_base64(&"eyJidW5kbGVfaWQiOiAiYSJ9".to_string()).unwrap(),
        };

        // receive cw721 token
        let res = receive_cw1155(deps.as_mut(), info, msg).unwrap();
        let info = mock_info(CONTRACT, &[]);
        let msg = Cw1155BatchReceiveMsg {
            operator: ALICE.into(),
            from: None,
            batch: vec![(TOKEN_ID.into(), Uint128::from(2u128))],
            msg: Binary::from_base64(&"eyJidW5kbGVfaWQiOiAiYSJ9".to_string()).unwrap(),
        };
        assert_eq!(
            res,
            Response::default()
                .add_attribute("action", "deposit_cw1155")
                .add_attribute("sender", msg.operator)
                .add_attribute("contract_sender", info.sender.to_string())
                .add_attribute("bundle_id", "a")
        );

        // ensure num tokens in bundle is 1
        let bundle = CW1155_BUNDLE
            .may_load(&deps.storage, "a".into())
            .unwrap()
            .unwrap();
        assert_eq!(1, bundle.len());
    }

    #[test]
    fn try_withdraw() {
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

        let info = mock_info(MINTER, &[]);
        let msg = Cw721ReceiveMsg {
            sender: ALICE.into(),
            token_id: TOKEN_ID.into(),
            msg: Binary::from_base64(&"eyJidW5kbGVfaWQiOiAiYSJ9".to_string()).unwrap(),
        };

        // receive cw721 token
        let _res = receive_cw721(deps.as_mut(), info, msg).unwrap();

        // withdraw
        let info = mock_info(ALICE, &[]);
        let _res = withdraw(deps.as_mut(), mock_env(), info, "a".into()).unwrap();

        // ensure num tokens in bundle is 0
        let bundle = CW721_BUNDLE
            .may_load(&deps.storage, "a".into())
            .unwrap()
            .unwrap();
        assert_eq!(0, bundle.len());
    }
}
