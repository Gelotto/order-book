use crate::models::{Order, TimeInForce};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64};
use cw20_base::msg::InstantiateMarketingInfo;
use cw_lib::models::{Token, TokenAmount};

#[cw_serde]
pub struct BaseTokenInitArgs {
  pub code_id: Uint64,
  pub name: String,
  pub symbol: String,
  pub decimals: u8,
  pub marketing: Option<InstantiateMarketingInfo>,
  pub cap: Uint128,
}

#[cw_serde]
pub struct InstantiateMsg {
  pub base_token: BaseTokenInitArgs,
  pub quote_tokens: Vec<Token>,
}

#[cw_serde]
pub enum ExecuteMsg {
  Submit(OrderRequest),
}

#[cw_serde]
pub enum QueryMsg {
  Select {
    fields: Option<Vec<String>>,
    account: Option<Addr>,
  },
  Orders {
    account: Addr,
    limit: Option<u8>,
    cursor: Option<Uint64>,
  },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct AccountView {
  pub base_balance: Uint128,
  pub quote_balances: Vec<TokenAmount>,
}

#[cw_serde]
pub struct OrdersResponse {
  pub orders: Vec<Order>,
  pub cursor: Option<Uint64>,
}

#[cw_serde]
pub struct SelectResponse {
  pub account: Option<AccountView>,
}

#[cw_serde]
pub enum OrderRequest {
  MarketBuy {
    quote: Token,
    balance: Uint128,
    tif: TimeInForce,
  },
  MarketSell {
    quote: Token,
    qty: Uint128,
    tif: TimeInForce,
  },
  LimitBuy {
    quote: Token,
    qty: Uint128,
    price: Uint128,
    tif: TimeInForce,
  },
  LimitSell {
    quote: Token,
    qty: Uint128,
    price: Uint128,
    tif: TimeInForce,
  },
}
