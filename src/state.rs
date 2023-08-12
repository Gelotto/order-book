use crate::msg::InstantiateMsg;
use crate::utils::increment;
use crate::{
  error::ContractError,
  models::{Order, OrderId},
};
use cosmwasm_std::{to_binary, Addr, DepsMut, Env, MessageInfo, Response, Storage, SubMsg, Uint128, Uint64, WasmMsg};
use cw20::{Cw20Coin, MinterResponse};
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use cw_lib::models::{Token, TokenAmount};
use cw_storage_plus::{Item, Map};

pub const CW20_INSTANTIATE_MSG_REPLY_ID: u64 = 1;
pub const BASE_TOKEN_ID: u32 = 1;

pub const BASE_TOKEN: Item<Token> = Item::new("base_token");
pub const TOKEN_ID_SEQ_NO: Item<u32> = Item::new("token_id_seq_no");
pub const ORDER_ID_SEQ_NO: Item<Uint64> = Item::new("order_id_seq_no");
pub const TOKEN_IDS: Map<String, u32> = Map::new("token_ids");
pub const TOKENS: Map<u32, Token> = Map::new("tokens");
pub const TOKEN_BALANCES: Map<(&Addr, u32), Uint128> = Map::new("token_balances");
pub const ORDERS: Map<OrderId, Order> = Map::new("orders");
pub const ACCOUNT_ORDER_IDS: Map<(&Addr, OrderId), u8> = Map::new("account_order_ids");
pub const ASKS: Map<(u32, u128, OrderId), u8> = Map::new("asks");
pub const BIDS: Map<(u32, u128, OrderId), u8> = Map::new("bids");

/// Initialize contract state data.
pub fn initialize(
  deps: DepsMut,
  env: &Env,
  info: &MessageInfo,
  msg: &InstantiateMsg,
) -> Result<Response, ContractError> {
  ORDER_ID_SEQ_NO.save(deps.storage, &Uint64::zero())?;
  TOKEN_ID_SEQ_NO.save(deps.storage, &BASE_TOKEN_ID)?;

  for token in msg.quote_tokens.iter() {
    register_token(deps.storage, token, None)?;
  }

  Ok(
    Response::new()
      .add_attribute("action", "instantiate")
      .add_submessage(SubMsg::reply_always(
        WasmMsg::Instantiate {
          admin: Some(env.contract.address.to_string()),
          code_id: msg.base_token.code_id.into(),
          msg: to_binary(&Cw20InstantiateMsg {
            decimals: msg.base_token.decimals,
            name: msg.base_token.name.clone(),
            symbol: msg.base_token.symbol.clone(),
            marketing: msg.base_token.marketing.clone(),
            initial_balances: vec![Cw20Coin {
              address: info.sender.to_string(),
              amount: msg.base_token.cap,
            }],
            mint: Some(MinterResponse {
              minter: env.contract.address.to_string(),
              cap: Some(msg.base_token.cap),
            }),
          })?,
          label: "CW20 Order Book Token".to_owned(),
          funds: vec![],
        },
        CW20_INSTANTIATE_MSG_REPLY_ID,
      )),
  )
}

pub fn load_token_amount(
  storage: &dyn Storage,
  owner: &Addr,
  token: &Token,
) -> Result<TokenAmount, ContractError> {
  let token_id = load_token_id(storage, token)?;
  if let Some(balance) = TOKEN_BALANCES.may_load(storage, (owner, token_id))? {
    Ok(TokenAmount {
      token: token.clone(),
      amount: balance,
    })
  } else {
    Ok(TokenAmount {
      token: token.clone(),
      amount: Uint128::zero(),
    })
  }
}

pub fn load_token_by_id(
  storage: &dyn Storage,
  token_id: u32,
) -> Result<Token, ContractError> {
  if let Some(token) = TOKENS.may_load(storage, token_id)? {
    Ok(token)
  } else {
    Err(ContractError::TokenNotFound)
  }
}

pub fn load_token_id(
  storage: &dyn Storage,
  token: &Token,
) -> Result<u32, ContractError> {
  if let Some(token_id) = TOKEN_IDS.may_load(storage, token.get_key())? {
    Ok(token_id)
  } else {
    Err(ContractError::TokenNotAllowed)
  }
}

pub fn register_token(
  storage: &mut dyn Storage,
  token: &Token,
  maybe_token_id: Option<u32>,
) -> Result<u32, ContractError> {
  let token_id = if let Some(token_id) = maybe_token_id {
    token_id
  } else {
    increment(storage, &TOKEN_ID_SEQ_NO, 1)?
  };
  TOKEN_IDS.save(storage, token.get_key(), &token_id)?;
  if token_id != BASE_TOKEN_ID {
    TOKENS.save(storage, token_id, token)?;
  }
  Ok(token_id)
}
