use crate::common::auth::Login;
use crate::common::transaction::TransactionEndPoint;
use crate::common::Money;

pub struct SalaryClientRequest {
    pub enterprise_name : Login,
    pub client_login : Login,
    pub account : TransactionEndPoint
}

pub struct SalaryAcceptRequest {
    pub idx : usize,
    pub salary : Money
}


