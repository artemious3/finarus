use serde::{
Serialize, Deserialize
};
use crate::common::Money;
use crate::common::auth::Login;


use crate::common::bank::AccountID;

#[derive(Clone, Serialize, Deserialize)]
pub struct Deposit {
    pub owner : Login,
    pub interest_rate : u8,
    pub start_date : chrono::DateTime<chrono::Utc>,
    pub last_update : chrono::DateTime<chrono::Utc>,
    pub end_date : chrono::DateTime<chrono::Utc>,
    pub initial_amount : Money,
    pub current_amount : Money
}

#[derive(Serialize, Deserialize)]
pub struct DepositNewRequest {
    pub src_account : AccountID,
    pub interest_rate : u8,
    pub months_expires : u32,
    pub amount : Money
}

#[derive(Serialize, Deserialize)]
pub struct DepositWithdrawRequest {
    pub deposit_idx : usize,
    pub dst_account : AccountID,
}


#[derive(Serialize, Deserialize)]
pub struct DepositWithdrawResponse {
    pub withdrawn_money : Money
}
