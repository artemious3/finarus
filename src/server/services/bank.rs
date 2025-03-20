use crate::bank::Bank;
use crate::server::RequestParams;
use crate::server::ServerError;
use crate::services::auth::AuthService;
use crate::services::salary::SalaryService;
use crate::services::time::TimeService;
use crate::traits::dynamic::Dynamic;

use l1::common::account::*;
use l1::common::auth::Token;
use l1::common::bank::*;
use l1::common::credit::*;
use l1::common::deposit::*;
use l1::common::salary::*;
use l1::common::transaction::{Transaction, TransactionEndPoint};
use l1::common::user::UserType;
use l1::common::Money;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Datelike, Utc};

use std::collections::HashMap;
pub struct BankService {
    auth: Arc<Mutex<AuthService>>,
    time: Arc<Mutex<TimeService>>,
    salary: SalaryService,

    banks: HashMap<BIK, Bank>,
    transactions: Vec<Transaction>,
}

struct BankRequestContext {
    login: String,
    token: Token,
    bik: BIK,
}

fn credit_monthly_pay(params: &CreditParams) -> Money {
    let amount = params.amount;
    let term = params.term as i32;
    let rate = params.interest_rate as f64 / 100.0;

    let res = (*amount as f64) * (rate + rate / ((1.0 + rate).powi(term) - 1.0));

    Money(res.ceil() as i32)
}

impl BankService {
    pub fn new(serv: Arc<Mutex<AuthService>>, tm: Arc<Mutex<TimeService>>) -> Self {
        let mut bs = BankService {
            auth: serv,
            time: tm,
            banks: HashMap::new(),
            transactions: Vec::new(),
            salary: SalaryService::default(),
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
        role: UserType,
    ) -> Result<BankRequestContext, ServerError> {
        let token = params
            .token
            .ok_or(ServerError::BadRequest("No token".to_string()))?;
        let auth = self.auth.lock().expect("Mutex");
        let login = auth
            .validate_authentification(token, role)
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
    fn perform_transaction(
        &mut self,
        transaction: Transaction,
        check_balance: bool,
    ) -> Result<(), &str> {
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
            if check_balance && src_acc.balance < transaction.amount {
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

    pub fn transaction_revert(&mut self, params: &RequestParams) -> Result<(), ServerError> {
        let ctx = self.get_request_context(params, UserType::Operator);
        let inv_trans = self
            .transactions
            .last()
            .ok_or(ServerError::Forbidden("No transactions yet".to_string()))?
            .inverse();

        self.perform_transaction(inv_trans, false)
            .map_err(|e| ServerError::Forbidden(e.to_string()))?;
        Ok(())
    }

    pub fn transactions_get(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn transaction(
        &mut self,
        transaction: Transaction,
        params: &RequestParams,
    ) -> Result<(), ServerError> {
        let ctx = self.get_request_context(params, UserType::Client)?;
        let bank = self
            .banks
            .get_mut(&ctx.bik)
            .ok_or(ServerError::BadRequest("Invalid BIK".to_string()))?;

        bank.validate_account_identity(transaction.src.account_id, &ctx.login)
            .map_err(|_| {
                ServerError::Forbidden("Accound does not exist or belong to user".to_string())
            })?;

        self.perform_transaction(transaction, true)
            .map_err(|e| ServerError::Forbidden(e.to_string()))?;

        Ok(())
    }

    pub fn transaction_unprotected(
        &mut self,
        transaction: Transaction,
        params: &RequestParams,
    ) -> Result<(), ServerError> {
        let _ctx = self.get_request_context(params, UserType::Manager);
        self.perform_transaction(transaction, true)
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
        let ctx = self.get_request_context(params, UserType::Client)?;
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
        let ctx = self.get_request_context(params, UserType::Client)?;
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
        let ctx = self.get_request_context(params, UserType::Client)?;
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
        let ctx = self.get_request_context(params, UserType::Client)?;
        let now = self.time.lock().unwrap().get_time();

        self.get_bank(ctx.bik)
            .ok_or(ServerError::BadRequest("Bad bank".to_string()))?
            .validate_account_identity(req.src_account, &ctx.login)
            .map_err(|_| {
                ServerError::Forbidden(
                    "This account does not exist or does not belong to user".to_string(),
                )
            })?;
        self.perform_transaction(
            Transaction {
                src: TransactionEndPoint {
                    bik: ctx.bik,
                    account_id: req.src_account,
                },
                dst: TransactionEndPoint {
                    bik: 0,
                    account_id: 0,
                },
                amount: req.amount,
            },
            true,
        )
        .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;

        let deposit = Deposit {
            owner: ctx.login.clone(),
            interest_rate: 5,
            start_date: now,
            last_update: now,
            end_date: now + chrono::Months::new(req.months_expires),
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
        let ctx = self.get_request_context(params, UserType::Client)?;
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

        self.perform_transaction(
            Transaction {
                src: TransactionEndPoint {
                    bik: 0,
                    account_id: 0,
                },
                dst: TransactionEndPoint {
                    bik: ctx.bik,
                    account_id: req.dst_account,
                },
                amount: withdrawn,
            },
            true,
        )
        .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;

        Ok(())
    }

    pub fn deposits_get(&self, params: &RequestParams) -> Result<Vec<Deposit>, ServerError> {
        let ctx = self.get_request_context(params, UserType::Client)?;
        let bank = self
            .banks
            .get(&ctx.bik)
            .ok_or(ServerError::BadRequest("Invalid BIK".to_string()))?;

        Ok(bank.deposit_service.get(ctx.login).clone())
    }

    pub fn credit_new(
        &mut self,
        req: CreditParams,
        params: &RequestParams,
    ) -> Result<(), ServerError> {
        let ctx = self.get_request_context(params, UserType::Client)?;
        let bank = self
            .get_bank_mut(ctx.bik)
            .ok_or(ServerError::BadRequest("Bad bank".to_string()))?;

        bank.validate_account_identity(req.src_account, &ctx.login)
            .map_err(|s| ServerError::Forbidden(s.to_string()))?;

        let credit = CreditUnaccepted {
            owner: ctx.login,
            params: req,
        };

        bank.credit_service.unaccepted_credits.push(credit);
        Ok(())
    }

    pub fn credit_accept(
        &mut self,
        req: CreditAcceptRequest,
        params: &RequestParams,
    ) -> Result<(), ServerError> {
        let ctx = self.get_request_context(params, UserType::Manager)?;
        let now = self.time.lock().unwrap().get_time();

        let credit_template = self
            .get_bank(ctx.bik)
            .ok_or(ServerError::BadRequest("Bad bank".to_string()))?
            .credit_service
            .unaccepted_credits
            .get(req.idx)
            .ok_or(ServerError::BadRequest("Index out of range".into()))?
            .clone();

        self.perform_transaction(
            Transaction {
                amount: credit_template.params.amount,
                src: TransactionEndPoint {
                    bik: 0,
                    account_id: 0,
                },
                dst: TransactionEndPoint {
                    bik: ctx.bik,
                    account_id: credit_template.params.src_account,
                },
            },
            true,
        )
        .map_err(|e| ServerError::Forbidden(e.to_string()))?;

        let credit = Credit {
            owner: credit_template.owner.clone(),
            params: credit_template.params.clone(),
            monthly_pay: credit_monthly_pay(&credit_template.params),
            first_pay: now + chrono::Months::new(1),
            last_pay: now + chrono::Months::new(1),
        };

        let bank = self
            .get_bank_mut(ctx.bik)
            .expect("Bank not found after it was found");
        bank.credit_service.unaccepted_credits.swap_remove(req.idx);

        bank.credit_service
            .accepted_credits
            .get_mut(&credit_template.owner)
            .expect("Bad client")
            .push(credit);

        Ok(())
    }

    pub fn credit_get(&self, params: &RequestParams) -> Result<Vec<Credit>, ServerError> {
        let ctx = self.get_request_context(params, UserType::Client)?;

        Ok(self
            .get_bank(ctx.bik)
            .ok_or(ServerError::BadRequest("Bad bank".into()))?
            .credit_service
            .accepted_credits
            .get(&ctx.login)
            .ok_or(ServerError::BadRequest("Bad token".into()))?
            .to_vec())
    }

    pub fn credit_get_unaccepted(
        &self,
        params: &RequestParams,
    ) -> Result<Vec<CreditUnaccepted>, ServerError> {
        let ctx = self.get_request_context(params, UserType::Manager)?;

        Ok(self
            .get_bank(ctx.bik)
            .ok_or(ServerError::BadRequest("Bad bank".into()))?
            .credit_service
            .unaccepted_credits
            .to_vec())
    }

    pub fn salary_request(
        &mut self,
        req: SalaryClientRequest,
        params: &RequestParams,
    ) -> Result<(), ServerError> {
        let ctx = self.get_request_context(params, UserType::Client)?;
        if req.client_login != ctx.login {
            Err(ServerError::BadRequest(
                "Invalid salary login request".to_string(),
            ))
        } else {
            let bik = req.account.bik;
            self.get_bank(bik).ok_or(ServerError::BadRequest("Bad bank".to_string()))?
                .validate_account_identity(req.account.account_id, &ctx.login)
                .map_err(|e| ServerError::Forbidden(e.to_string()))?;
            self.salary.salary_request(req)?;
            Ok(())
        }
    }

    pub fn salary_accept_decline(
        &mut self,
        req: SalaryAcceptRequest,
        params: &RequestParams,
    ) -> Result<(), ServerError> {
        let ctx = self.get_request_context(params, UserType::Client)?;
        self.salary.salary_accept_decline(ctx.login, &req)?;
        Ok(())
    }


    pub fn init_salary_proj(
        &mut self,
        req : SalaryInitProjRequest,
        params: &RequestParams
    ) -> Result<(), ServerError> {
        let ctx = self.get_request_context(params, UserType::Client)?;
        self.salary.init_salary_proj(ctx.login, req.account);
        Ok(())
    }


    pub fn salary_accept_decline_get(
        &self,
        params: &RequestParams
    ) -> Result<&Vec<SalaryClientRequest>, ServerError> {
        let ctx = self.get_request_context(params, UserType::Client)?;
        Ok(self.salary.salary_requests.get(&ctx.login).ok_or(
                ServerError::BadRequest("No salary requests for this enterprise".to_string())
        )?)
    }

    pub fn accept_salary_proj(&mut self, req: SalaryAcceptProjRequest) -> Result<(), ServerError>{
        self.salary.salary_projects.get_mut(&req.enterprise).ok_or(
            ServerError::BadRequest("No salary project for this enterprise".to_string())
        )?.accepted = true;
        Ok(())
    }
}

type UnitSize = i32;
pub fn signed_month_difference(start: &DateTime<Utc>, end: &DateTime<Utc>) -> UnitSize {
    let end_naive = end.date_naive();
    let start_naive = start.date_naive();

    let month_diff = end_naive.month() as UnitSize - start_naive.month() as UnitSize;
    let years_diff = (end_naive.year() - start_naive.year()) as UnitSize;
    if month_diff >= 0 {
        (years_diff * 12) + month_diff
    } else {
        (years_diff - 1) * 12 + (month_diff + 12)
    }
}

impl Dynamic for BankService {
    fn update(&mut self, time: &chrono::DateTime<chrono::Utc>) {
        let mut transactions: Vec<Transaction> = Vec::new();

        for (_, bank) in &mut self.banks {
            bank.update(time);

            for (_, credit_list) in &mut bank.credit_service.accepted_credits {
                for credit in credit_list {
                    let months_since_last_pay = signed_month_difference(&credit.last_pay, time);
                    let months_paid = signed_month_difference(&credit.first_pay, &credit.last_pay);
                    let months_remaining = credit.params.term as i32 - months_paid;

                    if months_since_last_pay > 0 && months_remaining > 0 {
                        let months_to_pay = std::cmp::min(months_since_last_pay, months_remaining);
                        transactions.push(Transaction {
                            amount: Money(months_to_pay * *credit.monthly_pay),
                            src: TransactionEndPoint {
                                bik: bank.public_info.bik,
                                account_id: credit.params.src_account,
                            },
                            dst: TransactionEndPoint {
                                bik: 0,
                                account_id: 0,
                            },
                        });

                        credit.last_pay = credit.first_pay
                            + chrono::Months::new((months_paid + months_to_pay) as u32);
                    }
                }
            }
        }

        for trans in transactions {
            let _ = self
                .perform_transaction(trans, false)
                .inspect_err(|e| log::error!("Transaction during update not performed : {}", e));
        }
    }
}
