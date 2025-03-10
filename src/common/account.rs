
use serde::{Serialize, Deserialize};
use crate::common::bank::{Account, AccountID};

#[derive(Serialize,  Deserialize)]
pub struct AccountOpenResp {
    pub account_id : AccountID 
}

#[derive(Serialize, Deserialize)]
pub struct AccountCloseReq {
    pub account_id : AccountID 
}


#[derive(Serialize, Deserialize)]
pub struct AccountsGetResp {
    pub accounts : Vec<Account>
}

