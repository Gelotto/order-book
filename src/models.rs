use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};

pub type OrderId = u64;

pub const BUY: u8 = 1;
pub const SELL: u8 = 2;

pub const FOK: u8 = 1;
pub const IOC: u8 = 2;
pub const GTC: u8 = 3;

pub const MARKET: u8 = 1;
pub const LIMIT: u8 = 2;

pub const CREATED: u8 = 1;
pub const MATCHED: u8 = 2;
pub const PARTIAL: u8 = 3;
pub const FILLED: u8 = 4;
pub const CANCELED: u8 = 5;

#[cw_serde]
pub enum TimeInForce {
  Fok,
  Ioc,
  Gtc,
}

impl From<TimeInForce> for u8 {
  fn from(value: TimeInForce) -> u8 {
    match value {
      TimeInForce::Fok => FOK,
      TimeInForce::Ioc => IOC,
      TimeInForce::Gtc => GTC,
    }
  }
}

impl From<u8> for TimeInForce {
  fn from(value: u8) -> Self {
    match value {
      FOK => TimeInForce::Fok,
      IOC => TimeInForce::Ioc,
      GTC => TimeInForce::Gtc,
      _ => panic!("Invalid u8 value for TimeInForce"),
    }
  }
}

#[cw_serde]
pub enum OrderSide {
  Buy,
  Sell,
}

impl From<OrderSide> for u8 {
  fn from(value: OrderSide) -> u8 {
    match value {
      OrderSide::Buy => BUY,
      OrderSide::Sell => SELL,
    }
  }
}

impl From<u8> for OrderSide {
  fn from(value: u8) -> Self {
    match value {
      BUY => OrderSide::Buy,
      SELL => OrderSide::Sell,
      _ => panic!("Invalid u8 value for OrderSide"),
    }
  }
}

#[cw_serde]
pub enum OrderKind {
  Market,
  Limit,
}

impl From<OrderKind> for u8 {
  fn from(value: OrderKind) -> u8 {
    match value {
      OrderKind::Market => MARKET,
      OrderKind::Limit => LIMIT,
    }
  }
}

impl From<u8> for OrderKind {
  fn from(value: u8) -> Self {
    match value {
      MARKET => OrderKind::Market,
      LIMIT => OrderKind::Limit,
      _ => panic!("Invalid u8 value for OrderKind"),
    }
  }
}

#[cw_serde]
pub enum OrderStatus {
  Created,
  Matched,
  Partial,
  Filled,
  Canceled,
}

impl From<OrderStatus> for u8 {
  fn from(value: OrderStatus) -> u8 {
    match value {
      OrderStatus::Created => CREATED,
      OrderStatus::Matched => MATCHED,
      OrderStatus::Partial => PARTIAL,
      OrderStatus::Filled => FILLED,
      OrderStatus::Canceled => CANCELED,
    }
  }
}

impl From<u8> for OrderStatus {
  fn from(value: u8) -> Self {
    match value {
      CREATED => OrderStatus::Created,
      PARTIAL => OrderStatus::Partial,
      MATCHED => OrderStatus::Matched,
      FILLED => OrderStatus::Filled,
      CANCELED => OrderStatus::Canceled,
      _ => panic!("Invalid u8 value for OrderStatus"),
    }
  }
}

#[cw_serde]
pub struct Order {
  pub id: Option<Uint64>,
  pub owner: Addr,
  pub created_at: Timestamp,
  pub side: u8,
  pub kind: u8,
  pub tif: u8,
  pub status: u8,
  pub balance: Uint128,
  pub funds: Uint128,
  pub qty_matched: Uint128,
  pub qty_requested: Uint128,
  pub limit_price: Uint128,
}

impl Order {
  pub fn get_qty_unmatched(&self) -> Uint128 {
    if self.qty_requested.is_zero() {
      return Uint128::zero();
    }
    self.qty_requested - self.qty_matched
  }
  pub fn get_balance_remaining(&self) -> Uint128 {
    if self.balance.is_zero() {
      return Uint128::zero();
    }
    self.funds - self.balance
  }

  pub fn is_qty_filled(&self) -> bool {
    self.qty_matched == self.qty_requested
  }

  pub fn is_buy_side(&self) -> bool {
    self.side == u8::from(OrderSide::Buy)
  }

  pub fn is_sell_side(&self) -> bool {
    self.side == u8::from(OrderSide::Sell)
  }

  pub fn is_limit_order(&self) -> bool {
    self.kind == u8::from(OrderKind::Limit)
  }

  pub fn is_market_order(&self) -> bool {
    self.kind == u8::from(OrderKind::Market)
  }
}

impl OrderSide {
  pub fn u8(&self) -> u8 {
    match self {
      OrderSide::Buy => BUY,
      OrderSide::Sell => SELL,
    }
  }
  pub fn u8_complement(&self) -> u8 {
    match self {
      OrderSide::Buy => SELL,
      OrderSide::Sell => BUY,
    }
  }
}
