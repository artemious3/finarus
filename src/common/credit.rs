
use chrono::{DateTime, Utc};
use crate::common::bank::AccountID;
use crate::common::Money;
use crate::common::auth::Login;

#[repr(u8)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub enum CreditTerm {
    M3 = 3,
    M6 = 6,
    M12 = 12,
    M24 = 24,
    MG24(u8)
}

impl ToString for CreditTerm {
    fn to_string(&self) -> String {
        match self {
            Self::M3 => "3 months".to_string(),
            Self::M6 => "6 months".to_string(),
            Self::M12 => "12 months".to_string(),
            Self::M24 => "24 months".to_string(),
            Self::MG24(_) => "custom".to_string()
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CreditParams {
    pub src_account : AccountID,
    pub interest_rate : u8,
    pub term : u8,
    pub amount : Money,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Credit {
    pub owner : Login,

    pub params : CreditParams,

    pub monthly_pay : Money,

    pub first_pay : DateTime<Utc>,
    pub last_pay :  DateTime<Utc>,
}


impl ToString for CreditUnaccepted{
    fn to_string(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }
}


#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CreditUnaccepted {
    pub owner : Login,
    pub params : CreditParams
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreditNewRequest {
    pub src_account : AccountID,
    pub interest_rate : Option<u8>,
    pub term : CreditTerm,
    pub amount : Money
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreditAcceptRequest {
    pub idx : usize
}
