use crate::{
  error::ContractError,
  msg::{AccountView, SelectResponse},
  state::{load_token_amount, load_token_by_id, BASE_TOKEN, BASE_TOKEN_ID, TOKEN_BALANCES},
};
use cosmwasm_std::{Addr, Deps, Storage};
use cw_lib::{loader::StateLoader, models::TokenAmount};

pub fn select(
  deps: Deps,
  fields: Option<Vec<String>>,
  account: Option<Addr>,
) -> Result<SelectResponse, ContractError> {
  let loader = StateLoader::new(deps.storage, &fields, &account);
  let base_token = BASE_TOKEN.load(deps.storage)?;
  Ok(SelectResponse {
    account: loader.view("account", |account_addr| {
      Ok(Some(AccountView {
        base_balance: load_token_amount(deps.storage, &account_addr, &base_token)?.amount,
        quote_balances: load_quote_balances(deps.storage, &account_addr)?,
      }))
    })?,
  })
}

fn load_quote_balances(
  storage: &dyn Storage,
  owner: &Addr,
) -> Result<Vec<TokenAmount>, ContractError> {
  let mut token_amounts: Vec<TokenAmount> = Vec::with_capacity(2);
  for result in TOKEN_BALANCES
    .prefix(owner)
    .range(storage, None, None, cosmwasm_std::Order::Ascending)
  {
    let (token_id, balance) = result?;
    if token_id != BASE_TOKEN_ID {
      let token = load_token_by_id(storage, token_id)?;
      token_amounts.push(TokenAmount { token, amount: balance })
    }
  }
  Ok(token_amounts)
}
