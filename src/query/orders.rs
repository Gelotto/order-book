use std::marker::PhantomData;

use crate::{
  error::ContractError,
  models::Order,
  msg::OrdersResponse,
  state::{ACCOUNT_ORDER_IDS, ORDERS},
};
use cosmwasm_std::{Addr, Deps, Uint64};
use cw_storage_plus::Bound;

pub fn orders(
  deps: Deps,
  account: Addr,
  maybe_cursor: Option<Uint64>,
  maybe_limit: Option<u8>,
) -> Result<OrdersResponse, ContractError> {
  let mut orders: Vec<Order> = Vec::with_capacity(20);

  let start_bound = if let Some(cursor) = maybe_cursor {
    Some(Bound::Exclusive((cursor.u64(), PhantomData)))
  } else {
    None
  };

  for result in ACCOUNT_ORDER_IDS
    .prefix(&account)
    .keys(deps.storage, None, start_bound, cosmwasm_std::Order::Descending)
    .take(maybe_limit.unwrap_or(50).clamp(1, 50) as usize)
  {
    let order_id = result?;
    let mut order = ORDERS.load(deps.storage, order_id)?;
    order.id = Some(order_id.into());
    orders.push(order)
  }

  Ok(OrdersResponse {
    cursor: if orders.is_empty() {
      None
    } else {
      orders.last().unwrap().id
    },
    orders,
  })
}
