use crate::common::Money;

use serde::{Serialize, Deserialize};
pub type BIK = u64;
pub type AccountID = u64;



#[derive(Serialize, Deserialize, Clone)]
pub enum AccountStatus {
    Normal,
    Frozen,
    Blocked
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Account {
   pub id : AccountID,
   pub balance : Money,
   pub status : AccountStatus,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct BankPublicInfo {
    pub bik : BIK,
    pub address : String,
    pub name : String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BanksGetResp {
    pub banks : Vec<BankPublicInfo>
}

