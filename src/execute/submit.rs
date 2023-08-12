use std::marker::PhantomData;

use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response, Storage, Timestamp, Uint128, Uint64};
use cw_lib::models::Token;
use cw_storage_plus::PrefixBound;

use crate::{
  error::ContractError,
  models::{Order, OrderId, OrderKind, OrderSide, OrderStatus, TimeInForce},
  msg::OrderRequest,
  state::{load_token_id, ACCOUNT_ORDER_IDS, ASKS, BASE_TOKEN_ID, BIDS, ORDERS, ORDER_ID_SEQ_NO, TOKEN_BALANCES},
};

pub fn submit(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  req: OrderRequest,
) -> Result<Response, ContractError> {
  let (order_id, order) = process_order_request(deps.storage, env.block.time, &req, &info.sender)?;
  Ok(Response::new().add_attributes(vec![
    attr("action", "submit_order"),
    attr("order_id", order_id.to_string()),
    attr("order_status", format!("{:?}", order.status)),
  ]))
}

fn process_order_request(
  storage: &mut dyn Storage,
  time: Timestamp,
  req: &OrderRequest,
  owner: &Addr,
) -> Result<(OrderId, Order), ContractError> {
  let order_id = get_next_order_id(storage)?;
  let order = match req.clone() {
    OrderRequest::MarketBuy { balance, tif, quote } => {
      // Buy as many shares as possible using the given balance
      let side = OrderSide::Buy;
      let qty = Uint128::zero();
      match_market_order(storage, time, owner, order_id, quote, balance, qty, tif, side)?
    },
    OrderRequest::MarketSell { qty, tif, quote } => {
      // Buy as many shares as possible using the given balance
      let side = OrderSide::Sell;
      let balance = Uint128::zero();
      match_market_order(storage, time, owner, order_id, quote, balance, qty, tif, side)?
    },
    OrderRequest::LimitBuy { qty, price, tif, quote } => {
      // Only buy shares listed at the given limit price
      let side = OrderSide::Buy;
      match_limit_order(storage, time, owner, order_id, quote, qty, price, tif, side)?
    },
    OrderRequest::LimitSell { qty, price, tif, quote } => {
      // Only sell shares listed at the given limit price
      let side = OrderSide::Sell;
      match_limit_order(storage, time, owner, order_id, quote, qty, price, tif, side)?
    },
  };
  Ok((order_id, order))
}

fn get_next_order_id(storage: &mut dyn Storage) -> Result<OrderId, ContractError> {
  Ok(
    ORDER_ID_SEQ_NO
      .update(storage, |n| -> Result<_, ContractError> { Ok(n + Uint64::one()) })?
      .u64(),
  )
}

fn match_market_order(
  storage: &mut dyn Storage,
  created_at: Timestamp,
  owner: &Addr,
  new_order_id: OrderId,
  quote_token: Token,
  initial_balance: Uint128,
  qty_requested: Uint128,
  tif: TimeInForce,
  side: OrderSide,
) -> Result<Order, ContractError> {
  let quote_token_id = load_token_id(storage, &quote_token)?;
  let is_buy_req = side == OrderSide::Buy;
  let matched_map = if is_buy_req { ASKS } else { BIDS };
  let mut matched_orders: Vec<(OrderId, Order, u128)> = Vec::with_capacity(4);
  let mut new_order = Order {
    owner: owner.clone(),
    side: side.into(),
    tif: tif.into(),
    balance: initial_balance,
    funds: initial_balance,
    qty_requested,
    created_at,
    id: None,
    status: OrderStatus::Created.into(),
    kind: OrderKind::Market.into(),
    limit_price: Uint128::zero(),
    qty_matched: Uint128::zero(),
  };

  match OrderSide::from(new_order.side) {
    OrderSide::Buy => {
      // Match against asks
      for result in matched_map.prefix_range(
        storage,
        Some(PrefixBound::Inclusive(((quote_token_id, u128::MIN), PhantomData))),
        Some(PrefixBound::Inclusive(((quote_token_id, u128::MAX), PhantomData))),
        cosmwasm_std::Order::Ascending,
      ) {
        let ((matched_price, _, matched_order_id), _) = result?;
        let mut matched_order = ORDERS.load(storage, matched_order_id)?;

        // Get the qty affordable with the new order's remaining balance. If it
        // can't afford anything more, stop matching.
        let qty_needed = new_order.balance / Uint128::from(matched_price);
        if qty_needed.is_zero() {
          break;
        }

        let qty_available = matched_order.get_qty_unmatched();
        let qty_delta = qty_available.min(qty_needed);
        let balance_delta = Uint128::from(matched_price) * qty_delta;

        new_order.balance -= balance_delta;
        new_order.qty_matched += qty_delta;
        matched_order.qty_matched += qty_delta;
        if matched_order.is_qty_filled() {
          matched_order.status = OrderStatus::Filled.into();
        } else {
          matched_order.status = OrderStatus::Partial.into();
        }

        matched_orders.push((matched_order_id, matched_order, qty_delta.into()));
      }

      // apply buy-side time in force
      match TimeInForce::from(new_order.tif) {
        TimeInForce::Fok => {
          if !new_order.balance.is_zero() {
            return Err(ContractError::InsufficientLiquidity);
          }
          new_order.status = OrderStatus::Filled.into();
        },
        TimeInForce::Ioc => {
          if new_order.qty_matched.is_zero() {
            return Err(ContractError::InsufficientLiquidity);
          } else if new_order.qty_matched.is_zero() {
            new_order.status = OrderStatus::Filled.into();
          } else {
            new_order.status = OrderStatus::Matched.into();
          }
        },
        TimeInForce::Gtc => {
          return Err(ContractError::TimeInForceNotAllowed);
        },
      }
    },
    OrderSide::Sell => {
      // Match against bids
      for result in matched_map.prefix_range(
        storage,
        Some(PrefixBound::Inclusive(((quote_token_id, u128::MAX), PhantomData))),
        Some(PrefixBound::Inclusive(((quote_token_id, u128::MIN), PhantomData))),
        cosmwasm_std::Order::Descending,
      ) {
        let ((_, _, matched_order_id), _) = result?;
        let mut matched_order = ORDERS.load(storage, matched_order_id)?;
        let qty_available = matched_order.get_qty_unmatched();
        let qty_needed = new_order.get_qty_unmatched();
        let qty_delta = qty_available.min(qty_needed);

        new_order.qty_matched += qty_delta;
        matched_order.qty_matched += qty_delta;
        if matched_order.is_qty_filled() {
          matched_order.status = OrderStatus::Filled.into();
        } else {
          matched_order.status = OrderStatus::Partial.into();
        }

        matched_orders.push((matched_order_id, matched_order, qty_available.into()));

        if new_order.is_qty_filled() {
          break;
        }
      }

      match TimeInForce::from(new_order.tif) {
        TimeInForce::Fok => {
          if new_order.qty_matched != qty_requested {
            return Err(ContractError::InsufficientLiquidity);
          }
          new_order.status = OrderStatus::Filled.into();
        },
        TimeInForce::Ioc => {
          if new_order.qty_matched.is_zero() {
            return Err(ContractError::InsufficientLiquidity);
          } else if new_order.qty_matched == qty_requested {
            new_order.status = OrderStatus::Filled.into();
          } else {
            new_order.status = OrderStatus::Matched.into();
          }
        },
        TimeInForce::Gtc => {
          return Err(ContractError::TimeInForceNotAllowed);
        },
      }
    },
  }

  // Save updated matched orders and update their balances.
  for (order_id, order, base_delta) in matched_orders.iter() {
    let base_delta = Uint128::from(*base_delta);
    let quote_delta = base_delta * order.limit_price;
    let token_delta = if is_buy_req { quote_delta } else { base_delta };
    let token_id = if is_buy_req { quote_token_id } else { BASE_TOKEN_ID };
    if order.status == u8::from(OrderStatus::Filled) {
      let map_key = (quote_token_id, order.limit_price.u128(), *order_id);
      matched_map.remove(storage, map_key);
    }
    ORDERS.save(storage, *order_id, order)?;
    increment_token_balance(storage, &order.owner, token_id, token_delta)?;
  }

  // Save new order and update its balance.
  ORDERS.save(storage, new_order_id, &new_order)?;
  ACCOUNT_ORDER_IDS.save(storage, (owner, new_order_id), &1)?;

  increment_token_balance(
    storage,
    &new_order.owner,
    if is_buy_req { BASE_TOKEN_ID } else { quote_token_id },
    if is_buy_req {
      new_order.qty_matched
    } else {
      new_order.get_balance_remaining()
    },
  )?;

  Ok(new_order)
}

fn increment_token_balance(
  storage: &mut dyn Storage,
  addr: &Addr,
  token_id: u32,
  delta: Uint128,
) -> Result<(), ContractError> {
  TOKEN_BALANCES.update(storage, (addr, token_id), |maybe_balance| -> Result<_, ContractError> {
    let balance = maybe_balance.unwrap_or_default();
    Ok(balance + delta)
  })?;
  Ok(())
}

fn match_limit_order(
  storage: &mut dyn Storage,
  created_at: Timestamp,
  owner: &Addr,
  new_order_id: OrderId,
  quote_token: Token,
  qty_requested: Uint128,
  price: Uint128,
  tif: TimeInForce,
  side: OrderSide,
) -> Result<Order, ContractError> {
  let quote_token_id = load_token_id(storage, &quote_token)?;
  let is_buy_req = side == OrderSide::Buy;
  let matched_map = if is_buy_req { ASKS } else { BIDS };
  let mut matched_orders: Vec<(OrderId, Order, Uint128)> = Vec::with_capacity(4);
  let mut new_order = Order {
    qty_requested,
    created_at,
    owner: owner.clone(),
    limit_price: price,
    id: None,
    tif: tif.into(),
    side: side.into(),
    qty_matched: Uint128::zero(),
    funds: Uint128::zero(),
    status: OrderStatus::Created.into(),
    kind: OrderKind::Limit.into(),
    balance: Uint128::zero(),
  };

  for result in
    matched_map
      .prefix((quote_token_id, price.into()))
      .keys(storage, None, None, cosmwasm_std::Order::Ascending)
  {
    let matched_order_id = result?;
    let mut matched_order = ORDERS.load(storage, matched_order_id)?;
    let qty_needed = new_order.get_qty_unmatched();
    let qty_available = matched_order.get_qty_unmatched();
    let qty_delta = qty_available.min(qty_needed);

    new_order.qty_matched += qty_delta;
    matched_order.qty_matched += qty_delta;
    if matched_order.is_qty_filled() {
      matched_order.status = OrderStatus::Filled.into();
    } else {
      matched_order.status = OrderStatus::Partial.into();
    }

    matched_orders.push((matched_order_id, matched_order, qty_available.into()));

    if new_order.is_qty_filled() {
      break;
    }
  }

  match TimeInForce::from(new_order.tif) {
    TimeInForce::Fok => {
      if new_order.qty_matched != qty_requested {
        return Err(ContractError::InsufficientLiquidity);
      }
      new_order.status = OrderStatus::Filled.into();
    },
    TimeInForce::Ioc => {
      if new_order.qty_matched.is_zero() {
        return Err(ContractError::InsufficientLiquidity);
      } else if new_order.qty_matched == qty_requested {
        new_order.status = OrderStatus::Filled.into();
      } else {
        new_order.status = OrderStatus::Matched.into();
      }
    },
    TimeInForce::Gtc => {
      if new_order.qty_matched == qty_requested {
        new_order.status = OrderStatus::Filled.into();
      } else if !new_order.qty_matched.is_zero() {
        new_order.status = OrderStatus::Partial.into();
      }
    },
  }

  for (order_id, order, qty_delta) in matched_orders.iter() {
    if order.status == u8::from(OrderStatus::Filled) {
      let qty_delta = *qty_delta;
      ORDERS.save(storage, *order_id, order)?;
      matched_map.remove(storage, (quote_token_id, price.into(), *order_id));
      increment_token_balance(
        storage,
        &order.owner,
        if is_buy_req { quote_token_id } else { BASE_TOKEN_ID },
        if is_buy_req { qty_delta * price } else { qty_delta },
      )?;
    }
  }

  let status: OrderStatus = new_order.status.into();

  if status == OrderStatus::Partial || status == OrderStatus::Created {
    let map = if new_order.is_buy_side() { BIDS } else { ASKS };
    map.save(storage, (quote_token_id, price.into(), new_order_id), &1)?;
  }

  ORDERS.save(storage, new_order_id, &new_order)?;
  ACCOUNT_ORDER_IDS.save(storage, (owner, new_order_id), &1)?;

  increment_token_balance(
    storage,
    &new_order.owner,
    if is_buy_req { BASE_TOKEN_ID } else { quote_token_id },
    if is_buy_req {
      new_order.qty_matched
    } else {
      new_order.qty_matched * price
    },
  )?;

  Ok(new_order)
}
