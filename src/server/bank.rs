use crate::services::deposit::DepositService;
use l1::common::auth::Login;
use l1::common::bank::*;
use crate::traits::dynamic::Dynamic;
use l1::common::transaction::*;
use l1::common::Money;
use std::collections::HashMap;

pub struct Bank {
    pub bik: BIK,
    pub deposit_service: DepositService,
    pub accounts: HashMap<AccountID, Account>,
    pub clients: HashMap<Login, Vec<AccountID>>,
}

impl Bank {
    pub fn new(bik: BIK) -> Self {
        Bank {
            bik: bik,
            deposit_service: DepositService::default(),
            accounts : HashMap::new(),
            clients : HashMap::new()
        }
    }

    pub fn add_client(&mut self, client: &String) {
        self.clients.insert(client.clone(), Vec::new());
        self.deposit_service.add_client(client.clone());
    }


    pub fn validate_account_identity(&self, acc : AccountID, login : &Login) -> Result<(), ()>{
        if self.clients.get(login).expect("Not a client").iter().find(|v:_| **v==acc).is_some(){
            Ok(())
        } else {
            Err(())
        }
    }

    /* Performs account replenishment without checking authentification.*/
    pub fn replenish_account(
        &mut self,
        account_id: AccountID,
        amount: Money,
    ) -> Result<(), &str> {
        let acc =self 
            .accounts
            .get_mut(&account_id)
            .ok_or("Invalid account id")?;
        acc.balance += amount;
        Ok(())
    }

    pub fn withdraw_account(
        &mut self,
        account_id: AccountID,
        amount: Money,
    ) -> Result<(), &str> {
        let acc =self 
            .accounts
            .get_mut(&account_id)
            .ok_or("Invalid account id")?;
        if acc.balance < amount {
            Err("Not enought money on account")
        } else {
            acc.balance += amount;
            Ok(())
        }
    }
}


impl Dynamic for Bank {

    fn update(&mut self, time :  &chrono::DateTime<chrono::Utc>) {
        self.deposit_service.update(time);
    }

}
