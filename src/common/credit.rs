
use chrono::{DateTime, Utc};
use crate::common::bank::AccountID;
use crate::common::Money;

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

pub struct Credit {
    pub src_account : AccountID,

    pub amount : Money,
    pub interest_rate : u8,
    pub monthly_pay : Money,
    pub term : CreditTerm,

    pub first_pay : DateTime<Utc>,
    pub last_pay :  DateTime<Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreditNewRequest {
    pub src_account : AccountID,
    pub interest_rate : Option<u8>,
    pub term : CreditTerm,
    pub amount : Money
}
