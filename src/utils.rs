use cosmwasm_std::{Storage, Uint128};
use cw_storage_plus::Item;
use serde::{de::DeserializeOwned, Serialize};

use crate::error::ContractError;

pub fn increment<T>(
  storage: &mut dyn Storage,
  item: &Item<T>,
  increment: T,
) -> Result<T, ContractError>
where
  T: DeserializeOwned + Serialize + std::ops::Add<Output = T>,
{
  item.update(storage, |x| -> Result<_, ContractError> {
    Ok(x + increment)
  })
}

pub fn decrement<T>(
  storage: &mut dyn Storage,
  item: &Item<T>,
  increment: T,
) -> Result<T, ContractError>
where
  T: DeserializeOwned + Serialize + std::ops::Sub<Output = T>,
{
  item.update(storage, |x| -> Result<_, ContractError> {
    Ok(x - increment)
  })
}

pub fn mul_pct(
  total: Uint128,
  pct: Uint128,
) -> Uint128 {
  total.multiply_ratio(pct, Uint128::from(1_000_000u128))
}
