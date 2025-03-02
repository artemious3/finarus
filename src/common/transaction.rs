use crate::common::bank::{AccountID, BIK};
use crate::common::Money;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct TransactionEndPoint {
    pub bik : BIK,
    pub account_id : AccountID
}

impl TransactionEndPoint {
    pub fn null() -> Self {
        TransactionEndPoint{bik:0,account_id:0}
    }
}


#[derive(Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub src : TransactionEndPoint, 
    pub dst : TransactionEndPoint,
    pub amount : Money
}

impl Transaction {
    pub fn inverse(&self) -> Transaction{
        Transaction{
            src : self.dst.clone(),
            dst : self.src.clone(),
            amount: self.amount
        }
    }
}

