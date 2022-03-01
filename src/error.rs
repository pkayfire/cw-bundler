use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Claimed")]
    Claimed {},

    #[error("Expired")]
    Expired {},

    #[error("DecodeError")]
    DecodeError {},

    #[error("SerdeJsonError")]
    SerdeJsonError {},
}

impl From<cw721_base::ContractError> for ContractError {
    fn from(err: cw721_base::ContractError) -> Self {
        match err {
            cw721_base::ContractError::Std(error) => ContractError::Std(error),
            cw721_base::ContractError::Unauthorized {} => ContractError::Unauthorized {},
            cw721_base::ContractError::Claimed {} => ContractError::Claimed {},
            cw721_base::ContractError::Expired {} => ContractError::Expired {},
        }
    }
}

impl From<base64::DecodeError> for ContractError {
    fn from(err: base64::DecodeError) -> Self {
        match err {
            base64::DecodeError::InvalidByte(_usize, _u8) => ContractError::DecodeError {},
            base64::DecodeError::InvalidLength => ContractError::DecodeError {},
            base64::DecodeError::InvalidLastSymbol(_usize, _u8) => ContractError::DecodeError {},
        }
    }
}

impl From<serde_json_wasm::de::Error> for ContractError {
    fn from(err: serde_json_wasm::de::Error) -> Self {
        match err {
            _ => ContractError::SerdeJsonError {},
        }
    }
}
