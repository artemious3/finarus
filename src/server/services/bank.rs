use crate::bank::Bank;
use crate::server::RequestParams;
use crate::server::ServerError;
use crate::services::auth::AuthService;
use crate::services::time;
use crate::traits::dynamic::Dynamic;
use chrono::Utc;
use l1::common::account::*;
use l1::common::auth::Token;
use l1::common::bank::*;
use l1::common::deposit::*;
use l1::common::transaction::{Transaction, TransactionEndPoint};
use l1::common::user::UserType;
use std::sync::{Arc, Mutex};

use std::collections::HashMap;
pub struct BankService {
    auth: Arc<Mutex<AuthService>>,

    banks: HashMap<BIK, Bank>,
    transactions: Vec<Transaction>,
}

struct BankRequestContext {
    login: String,
    token: Token,
    bik: BIK,
}

impl BankService {
    pub fn new(serv: Arc<Mutex<AuthService>>) -> Self {
        let mut bs = BankService {
            auth: serv,
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
                .get_mut(&transaction.src.account_id)
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

        // bank.validate_account_identity(transaction.src.account_id, &ctx.login)
        //     .map_err(|_| ServerError::Forbidden("Accound does not exist or belong to user".to_string()))?;

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
        self.get_bank(ctx.bik)
            .ok_or(ServerError::BadRequest("Bad bank".to_string()))?
            .validate_account_identity(req.dst_account, &ctx.login)
            .map_err(|_| ServerError::Forbidden("This account does not exist".to_string()))?;

        let withdrawn = self
            .get_bank_mut(ctx.bik)
            .ok_or(ServerError::BadRequest("Bad bank".to_string()))?
            .deposit_service
            .withdraw(ctx.login, req.deposit_idx, time::get_time())
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

    // pub fn handle_post(&mut self, req: &Request) -> Result<Response, ServerError> {
    //     assert_eq!(req.method(), "POST");
    //     let login = self
    //         .server
    //         .validate_authentification(req, UserType::Client)
    //         .map_err(|s: _| ServerError::Forbidden(s))?;
    //     let bank = self
    //         .bank_from_reqeust_mut(req)
    //         .map_err(|err: &str| ServerError::BadRequest(err.to_string()))?;
    //
    //     match req.url().as_str() {
    //         "/bank/deposit/new" => {
    //             let deposit_request = deserialize_request::<DepositNewRequest>(req)
    //                 .map_err(|_| ServerError::BadRequest("".to_string()))?;
    //             bank.validate_account_identity(deposit_request.src_account, &login)
    //                 .map_err(|_| {
    //                     ServerError::Forbidden(
    //                         "This account does not exist or does not belong to user".to_string(),
    //                     )
    //                 })?;
    //             self.perform_transaction(Transaction{
    //                 src : TransactionEndPoint {
    //                     bik : bank.bik,
    //                     account_id : deposit_request.src_account
    //                 },
    //                 dst : TransactionEndPoint {
    //                     bik : 0,
    //                     account_id : 0
    //                 },
    //                 amount : deposit_request.amount
    //             })
    //             .map_err(|err:_| ServerError::Forbidden(err.to_string()))?;
    //
    //             let deposit = Deposit {
    //                 owner: login.clone(),
    //                 interest_rate: 5,
    //                 start_date: Utc::now(),
    //                 last_update: Utc::now(),
    //                 end_date: Utc::now() + chrono::Months::new(deposit_request.months_expires),
    //                 initial_amount: deposit_request.amount,
    //                 current_amount: deposit_request.amount,
    //             };
    //             bank.deposit_service.add_deposit(login, deposit);
    //             Ok(Response::text("Ok"))
    //         }
    //         "/bank/deposit/withdraw" => {
    //             let deposit_request = deserialize_request::<DepositWithdrawRequest>(req)
    //                 .map_err(|_| ServerError::BadRequest("".to_string()))?;
    //             bank.validate_account_identity(deposit_request.dst_account, &login)
    //                 .map_err(|_| {
    //                     ServerError::Forbidden("This account does not exist".to_string())
    //                 })?;
    //
    //             let withdrawn = bank
    //                 .deposit_service
    //                 .withdraw(login, deposit_request.deposit_idx, time::get_time())
    //                 .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;
    //
    //             self.perform_transaction(Transaction{
    //                 src : TransactionEndPoint {
    //                     bik : 0,
    //                     account_id : 0
    //                 },
    //                 dst : TransactionEndPoint {
    //                     bik : bank.bik,
    //                     account_id : deposit_request.dst_account
    //                 },
    //                 amount : withdrawn
    //             })
    //             .map_err(|err:_| ServerError::Forbidden(err.to_string()))?;
    //
    //             Ok(Response::text("Ok"))
    //         }
    //
    //         "/bank/accounts/new" => {
    //             unimplemented!()
    //         }
    //
    //         "/bank/transaction" => {
    //             let transaction_request = deserialize_request::<Transaction>(req)
    //                 .map_err(|_| ServerError::BadRequest("".to_string()))?;
    //
    //             // bank.validate_account_identity(loging, transaction_request.sr)
    //             unimplemented!()
    //         }
    //
    //         _ => Err(ServerError::NotFound("".to_string())),
    //     }
    // }
    //
    // pub fn handle_get(&self, req: &Request) -> Result<Response, ServerError> {
    //     assert_eq!(req.method(), "GET");
    //     let bank = self
    //         .bank_from_reqeust(req)
    //         .map_err(|err: &str| ServerError::BadRequest(err.to_string()))?;
    //     match req.url().as_str() {
    //         "/bank/deposit" => {
    //             // bank.deposit_service.get()
    //             unimplemented!()
    //         }
    //
    //         "/bank/accounts" => {
    //             unimplemented!()
    //         }
    //         _ => Err(ServerError::NotFound("".to_string())),
    //     }
    // }
}

impl Dynamic for BankService {
    fn update(&mut self, time: &chrono::DateTime<chrono::Utc>) {
        for (_, bank) in &mut self.banks {
            bank.update(time);
        }
    }
}
