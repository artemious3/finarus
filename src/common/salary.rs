use crate::common::auth::Login;
use crate::common::transaction::TransactionEndPoint;
use crate::common::Money;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct SalaryClientRequest {
    pub enterprise_name : Login,
    pub client_login : Login,
    pub account : TransactionEndPoint
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SalaryAcceptRequest {
    pub idx : usize,
    pub accept : bool,
    pub salary : Money
}


impl ToString for SalaryClientRequest {
    fn to_string(&self) -> String {
        serde_yaml::to_string(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SalaryInitProjRequest {
    pub account : TransactionEndPoint
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SalaryAcceptProjRequest {
    pub enterprise : Login 
}

