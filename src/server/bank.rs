use crate::services::deposit::DepositService;
use crate::services::credit::CreditService;
use crate::traits::dynamic::Dynamic;
use l1::common::auth::Login;
use l1::common::bank::*;
use l1::common::Money;

use std::collections::HashMap;



pub struct Bank {
    pub deposit_service: DepositService,
    pub credit_service : CreditService,
    pub accounts: HashMap<AccountID, Account>,
    pub clients: HashMap<Login, Vec<AccountID>>,
    pub public_info: BankPublicInfo,
}

impl Bank {
    pub fn new(public_info: BankPublicInfo) -> Self {
        Bank {
            deposit_service: DepositService::default(),
            credit_service: CreditService::default(),
            accounts: HashMap::new(),
            clients: HashMap::new(),
            public_info,
        }
    }

    fn add_client_if_not_exist(&mut self, login: &Login) {
        if !self.clients.contains_key(login) {
            self.add_client(login);
        }
    }

    pub fn add_client(&mut self, client: &String) {
        self.clients.insert(client.clone(), Vec::new());
        self.deposit_service.add_client(client.clone());
        self.credit_service.add_client(client.clone());
    }

    pub fn validate_account_identity(&self, acc: AccountID, login: &Login) -> Result<(), &str> {
        let maybe_acc_id = self
            .clients
            .get(login)
            .ok_or("Client not found. Open at least one ACCOUNT in the bank")?
            .iter()
            .find(|v: _| **v == acc);
        if let Some(acc_id) = maybe_acc_id {
            let acc = self
                .accounts
                .get(acc_id)
                .expect("Accounts are not synced with clients");
            match acc.status {
                AccountStatus::Frozen => Err("Account frozen"),
                AccountStatus::Blocked => Err("Account blocked"),
                AccountStatus::Normal => Ok(()),
            }
        } else {
            Err("Account not found")
        }
    }

    pub fn account_new(&mut self, login: &Login) -> Result<AccountID, &str> {
        self.add_client_if_not_exist(login);
        let client_accounts = self.clients.get_mut(login).ok_or("Client not found")?;
        let new_acc_id = (self.accounts.len() + 1) as u64;

        // TODO : remove 1334
        let new_acc = Account {
            balance: Money(1334),
            id: new_acc_id as u64,
            status: AccountStatus::Normal,
        };
        let mb_old_acc = self.accounts.insert(new_acc_id, new_acc);
        if let Some(old_acc) = mb_old_acc {
            log::error!("Lost an account with id {}", old_acc.id);
        }

        client_accounts.push(new_acc_id);
        Ok(new_acc_id)
    }

    pub fn account_close(&mut self, login: &Login, id: AccountID) -> Result<(), &str> {
        self.validate_account_identity(id, login)
            .map_err(|_| "Account does not exist")?;
        let client_accounts = self
            .clients
            .get_mut(login)
            .expect("Account identidy validation did't work?");
        // TODO : check for deposits and credits open

        self.accounts.remove(&id).ok_or("Invalid account ID")?;

        let remove_pos = client_accounts
            .iter()
            .position(|val| *val == id)
            .expect("Account identity validation did not work?");
        client_accounts.remove(remove_pos);
        Ok(())
    }

    pub fn accounts_get(&self, login: &Login) -> Result<Vec<Account>, &str> {
        let client_accounts = self.clients.get(login).ok_or("Client not found")?;
        Ok(client_accounts
            .iter()
            .map(|acc_id| {
                self.accounts
                    .get(acc_id)
                    .expect("Client has unexisting account!")
                    .clone()
            })
            .collect())
    }

    //     /* Performs account replenishment without checking authentification.*/
    //     pub fn replenish_account(
    //         &mut self,
    //         account_id: AccountID,
    //         amount: Money,
    //     ) -> Result<(), &str> {
    //         let acc =self
    //             .accounts
    //             .get_mut(&account_id)
    //             .ok_or("Invalid account id")?;
    //         acc.balance += amount;
    //         Ok(())
    //     }
    //
    //     pub fn withdraw_account(
    //         &mut self,
    //         account_id: AccountID,
    //         amount: Money,
    //     ) -> Result<(), &str> {
    //         let acc =self
    //             .accounts
    //             .get_mut(&account_id)
    //             .ok_or("Invalid account id")?;
    //         if acc.balance < amount {
    //             Err("Not enought money on account")
    //         } else {
    //             acc.balance += amount;
    //             Ok(())
    //         }
    //     }
}

impl Dynamic for Bank {
    fn update(&mut self, time: &chrono::DateTime<chrono::Utc>) {
        self.deposit_service.update(time);
    }
}
