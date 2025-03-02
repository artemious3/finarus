use crate::common::Money;

pub type BIK = u64;
pub type AccountID = u64;
pub struct Account {
   pub id : AccountID,
   pub balance :Money,
}
