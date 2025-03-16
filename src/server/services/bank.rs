use crate::bank::Bank;
use crate::server::RequestParams;
use crate::server::ServerError;
use crate::services::auth::AuthService;
use crate::traits::dynamic::Dynamic;
use crate::services::time::TimeService;
use chrono::Utc;
use l1::common::account::*;
use l1::common::auth::Token;
use l1::common::bank::*;
use l1::common::deposit::*;
use l1::common::transaction::{Transaction, TransactionEndPoint};
use l1::common::user::UserType;
use l1::common::Money;
use std::sync::{Arc, Mutex};

use std::collections::HashMap;
pub struct BankService {
    auth: Arc<Mutex<AuthService>>,
    time: Arc<Mutex<TimeService>>,

    banks: HashMap<BIK, Bank>,
    transactions: Vec<Transaction>,
}

struct BankRequestContext {
    login: String,
    token: Token,
    bik: BIK,
}

impl BankService {
    pub fn new(serv: Arc<Mutex<AuthService>>, tm : Arc<Mutex<TimeService>>) -> Self {
        let mut bs = BankService {
            auth: serv,
            time : tm,
            banks: HashMap::new(),
            transactions: Vec::new(),
        };

        //TMP
        bs.banks.insert(
            1003004,
            Bank::new(BankPublicInfo {
                name: "Belarusbank".to_string(),
                bik: 1003004,
                address: "Nezalezhnasci pr, 4".to_string(),
            }),
        );
        //TMP

        bs
    }

    fn get_request_context(
        &self,
        params: &RequestParams,
    ) -> Result<BankRequestContext, ServerError> {
        let token = params
            .token
            .ok_or(ServerError::BadRequest("No token".to_string()))?;
        let auth = self.auth.lock().expect("Mutex");
        let login = auth
            .validate_authentification(token, UserType::Client)
            .map_err(|_| ServerError::Forbidden(String::new()))?;
        let bik = params
            .bik
            .ok_or(ServerError::BadRequest("No bank".to_string()))?;
        Ok(BankRequestContext { token, login, bik })
    }

    fn get_bank_mut(&mut self, bik: BIK) -> Option<&mut Bank> {
        self.banks.get_mut(&bik)
    }
    fn get_bank(&self, bik: BIK) -> Option<&Bank> {
        self.banks.get(&bik)
    }

    /* Performs transaction WITHOUT CHECKING AUTHENTIFICATION */
    fn perform_transaction(&mut self, transaction: Transaction) -> Result<(), &str> {
        if transaction.src.account_id == transaction.dst.account_id
            && transaction.src.bik == transaction.dst.bik
        {
            return Err("Source and destination are the same");
        }
        if transaction.amount <= Money(0) {
            return Err("Invalid amount");
        }
        // account_id=0 is dumb account, used for client-bank transactions
        if transaction.src.account_id != 0 {
            let src_bank = self
                .banks
                .get_mut(&transaction.src.bik)
                .ok_or("Invalid src BIK")?;
            let src_acc = src_bank
                .accounts
                .get_mut(&transaction.src.account_id)
                .ok_or("Invalid account id")?;
            if src_acc.balance < transaction.amount {
                return Err("Not enough money on src account");
            } else {
                src_acc.balance -= transaction.amount;
            }
        }

        if transaction.dst.account_id != 0 {
            let dst_bank = self
                .banks
                .get_mut(&transaction.dst.bik)
                .ok_or("Invalid dst BIK")?;
            let dst_acc = dst_bank
                .accounts
                .get_mut(&transaction.dst.account_id)
                .ok_or("Invalid account id")?;

            dst_acc.balance += transaction.amount;
        }

        self.transactions.push(transaction);
        Ok(())
    }

    pub fn transaction(
        &mut self,
        transaction: Transaction,
        params: &RequestParams,
    ) -> Result<(), ServerError> {
        let ctx = self.get_request_context(params)?;
        let bank = self
            .banks
            .get_mut(&ctx.bik)
            .ok_or(ServerError::BadRequest("Invalid BIK".to_string()))?;

        bank.validate_account_identity(transaction.src.account_id, &ctx.login)
            .map_err(|_| ServerError::Forbidden("Accound does not exist or belong to user".to_string()))?;

        self.perform_transaction(transaction)
            .map_err(|e| ServerError::Forbidden(e.to_string()))?;

        Ok(())
    }

    pub fn banks_get(&self) -> BanksGetResp {
        let banks = self
            .banks
            .iter()
            .map(|priv_bank| priv_bank.1.public_info.clone())
            .collect();

        BanksGetResp { banks }
    }

    pub fn account_open(&mut self, params: &RequestParams) -> Result<AccountOpenResp, ServerError> {
        let ctx = self.get_request_context(params)?;
        let bank = self
            .banks
            .get_mut(&ctx.bik)
            .ok_or(ServerError::BadRequest("Invalid BIK".to_string()))?;
        let new_acc_id = bank
            .account_new(&ctx.login)
            .map_err(|e| ServerError::Forbidden(e.to_string()))?;

        Ok(AccountOpenResp {
            account_id: new_acc_id,
        })
    }

    pub fn account_close(
        &mut self,
        req: AccountCloseReq,
        params: &RequestParams,
    ) -> Result<(), ServerError> {
        let ctx = self.get_request_context(params)?;
        let bank = self
            .banks
            .get_mut(&ctx.bik)
            .ok_or(ServerError::BadRequest("Invalid BIK".to_string()))?;
        let closed_acc_id = req.account_id;
        bank.account_close(&ctx.login, closed_acc_id)
            .map_err(|e| ServerError::Forbidden(e.to_string()))?;

        Ok(())
    }

    pub fn accounts_get(&self, params: &RequestParams) -> Result<AccountsGetResp, ServerError> {
        let ctx = self.get_request_context(params)?;
        let bank = self
            .banks
            .get(&ctx.bik)
            .ok_or(ServerError::BadRequest("Invalid BIK".to_string()))?;

        let accounts = bank
            .accounts_get(&ctx.login)
            .map_err(|e| ServerError::Forbidden(e.to_string()))?;

        Ok(AccountsGetResp { accounts })
    }

    pub fn deposit_new(
        &mut self,
        req: DepositNewRequest,
        params: &RequestParams,
    ) -> Result<(), ServerError> {
        let ctx = self.get_request_context(params)?;

        self.get_bank(ctx.bik)
            .ok_or(ServerError::BadRequest("Bad bank".to_string()))?
            .validate_account_identity(req.src_account, &ctx.login)
            .map_err(|_| {
                ServerError::Forbidden(
                    "This account does not exist or does not belong to user".to_string(),
                )
            })?;
        self.perform_transaction(Transaction {
            src: TransactionEndPoint {
                bik: ctx.bik,
                account_id: req.src_account,
            },
            dst: TransactionEndPoint {
                bik: 0,
                account_id: 0,
            },
            amount: req.amount,
        })
        .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;

        let deposit = Deposit {
            owner: ctx.login.clone(),
            interest_rate: 5,
            start_date: Utc::now(),
            last_update: Utc::now(),
            end_date: Utc::now() + chrono::Months::new(req.months_expires),
            initial_amount: req.amount,
            current_amount: req.amount,
        };
        self.get_bank_mut(ctx.bik)
            .ok_or(ServerError::BadRequest("Bad bank".to_string()))?
            .deposit_service
            .add_deposit(ctx.login, deposit);
        Ok(())
    }

    pub fn deposit_withdraw(
        &mut self,
        req: DepositWithdrawRequest,
        params: &RequestParams,
    ) -> Result<(), ServerError> {
        let ctx = self.get_request_context(params)?;
        let cur_time = self.time.lock().unwrap().get_time();
        self.get_bank(ctx.bik)
            .ok_or(ServerError::BadRequest("Bad bank".to_string()))?
            .validate_account_identity(req.dst_account, &ctx.login)
            .map_err(|_| ServerError::Forbidden("This account does not exist".to_string()))?;

        let withdrawn = self
            .get_bank_mut(ctx.bik)
            .ok_or(ServerError::BadRequest("Bad bank".to_string()))?
            .deposit_service
            .withdraw(ctx.login, req.deposit_idx, cur_time)
            .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;

        self.perform_transaction(Transaction {
            src: TransactionEndPoint {
                bik: 0,
                account_id: 0,
            },
            dst: TransactionEndPoint {
                bik: ctx.bik,
                account_id: req.dst_account,
            },
            amount: withdrawn,
        })
        .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;

        Ok(())
    }

    pub fn deposits_get(& self, params: &RequestParams) -> Result<Vec<Deposit>, ServerError> {
        let ctx = self.get_request_context(params)?;
        let bank = self
            .banks
            .get(&ctx.bik)
            .ok_or(ServerError::BadRequest("Invalid BIK".to_string()))?;

        Ok(bank.deposit_service.get(ctx.login).clone())
    }

}

impl Dynamic for BankService {
    fn update(&mut self, time: &chrono::DateTime<chrono::Utc>) {
        for (_, bank) in &mut self.banks {
            bank.update(time);
        }
    }
}
