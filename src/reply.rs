use cosmwasm_std::{Addr, DepsMut, Reply, Response};
use cw_lib::models::Token;

use crate::{
  error::ContractError,
  state::{register_token, BASE_TOKEN, BASE_TOKEN_ID, CW20_INSTANTIATE_MSG_REPLY_ID},
};

pub fn handle_reply(
  deps: DepsMut,
  reply: Reply,
) -> Result<Response, ContractError> {
  // Save newly created CW20 token, created through submsg during instantiation.
  if reply.id == CW20_INSTANTIATE_MSG_REPLY_ID {
    match &reply.result {
      // Extract and save the new CW20 token address
      cosmwasm_std::SubMsgResult::Ok(subcall_resp) => {
        if let Some(e) = subcall_resp.events.iter().find(|e| e.ty == "instantiate") {
          if let Some(attr) = e.attributes.iter().find(|attr| attr.key == "_contract_address") {
            let cw20_addr = Addr::unchecked(attr.value.to_string());
            let base_token = Token::Cw20 { address: cw20_addr };
            BASE_TOKEN.save(deps.storage, &base_token)?;
            register_token(deps.storage, &base_token, Some(BASE_TOKEN_ID))?;
          }
        }
      },
      cosmwasm_std::SubMsgResult::Err(err_reason) => {
        deps.api.debug(format!(">>> {}", err_reason).as_str());
        return Err(ContractError::Cw20InstantiationFailed);
      },
    }
  }
  Ok(Response::default())
}
