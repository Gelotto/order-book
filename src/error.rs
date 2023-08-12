use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("NotAuthorized")]
  NotAuthorized {},

  #[error("InsufficientLiquidity")]
  InsufficientLiquidity,

  #[error("TimeInForceNotAllowed")]
  TimeInForceNotAllowed,

  #[error("MetadataNotFound")]
  MetadataNotFound,

  #[error("TokenNotAllowed")]
  TokenNotAllowed,

  #[error("TokenNotFound")]
  TokenNotFound,

  #[error("Cw20InstantiationFailed")]
  Cw20InstantiationFailed,
}

impl From<ContractError> for StdError {
  fn from(err: ContractError) -> Self {
    StdError::generic_err(err.to_string())
  }
}
