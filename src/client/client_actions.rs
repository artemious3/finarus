use crate::client::ClientContext;
use crate::inputtable::*;
use crate::menu::Action;
use crate::selector::{select_from, select_idx};
use crate::utils::*;
use l1::common::account::*;
use l1::common::bank::AccountID;
use l1::common::bank::BanksGetResp;
use l1::common::credit::*;
use l1::common::deposit::*;
use l1::common::transaction::{Transaction, TransactionEndPoint};
use l1::common::user::Client;
use l1::common::Money;
use std::sync::{Arc, Mutex};

fn select_account(ctx: &ClientContext) -> Result<AccountID, String> {
    let resp = get_with_params(API!("/account"), &ctx)?;
    let resp_s = handle_errors(resp)?;
    let result: AccountsGetResp =
        serde_json::from_str(&resp_s).map_err(|_| "Wrong response".to_string())?;

    let acc_id = select_from(&result.accounts.iter().map(|acc| acc.id).collect())
        .ok_or("Cancelled".to_string())?;

    Ok(acc_id)
}

pub struct GetAuthInfoAction {}

fn ensure_bank_selected(ctx: &ClientContext) -> Result<(), String> {
    if ctx.bik.is_none() {
        return Err("Select the bank first".to_string());
    }
    Ok(())
}

impl Action for GetAuthInfoAction {
    fn name(&self) -> &'static str {
        "AUTHENTIFICATION INFO"
    }

    fn description(&self) -> &'static str {
        "Below the public personal info of currently logged user will be printed"
    }

    fn exec(&mut self, ctx_ref: Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();
        let response = get_with_params(API!("/auth"), &ctx)?;
        let response_str = handle_errors(response)?;
        let yaml = json_to_yaml::<Client>(response_str).ok_or("Server sent wrong response")?;
        println!("{}", yaml);
        Ok(())
    }
}

pub struct SelectBankAction {}

impl Action for SelectBankAction {
    fn name(&self) -> &'static str {
        "SELECT BANK"
    }

    fn description(&self) -> &'static str {
        "Select bank for further operations"
    }

    fn exec(&mut self, ctx_ref: Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let mut ctx = ctx_ref.lock().unwrap();
        let resp = get_with_params(API!("/banks"), &ctx)?;
        let response_str = handle_errors(resp)?;
        let banks: BanksGetResp = serde_json::from_str(&response_str)
            .map_err(|_| "Server send wrong response".to_string())?;

        let yaml =
            json_to_yaml::<BanksGetResp>(response_str).ok_or("Server sent wrong response")?;

        println!(
            "The Bank system currently has the following banks:\n {}",
            yaml
        );

        let bank_options = banks.banks.iter().map(|b| b.bik).collect();
        let maybe_opt = crate::selector::select_from(&bank_options);

        if let Some(opt) = maybe_opt {
            ctx.bik = Some(opt);
            Ok(())
        } else {
            Err("Cancelled".to_string())
        }
    }
}

pub struct AccountOpenAction {}

impl Action for AccountOpenAction {
    fn name(&self) -> &'static str {
        "OPEN account"
    }

    fn description(&self) -> &'static str {
        "New account in currently selected bank will be opened"
    }

    fn exec(&mut self, ctx_ref: Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();
        ensure_bank_selected(&ctx)?;

        let resp = post_with_params(API!("/account/open"), String::new(), &ctx)?;
        let resp_s = handle_errors(resp)?;
        let result: AccountOpenResp =
            serde_json::from_str(&resp_s).map_err(|_| "Wrong response".to_string())?;

        println!("Opened new account with ID {}", result.account_id);
        Ok(())
    }
}

pub struct AccountsGetAction {}

impl Action for AccountsGetAction {
    fn name(&self) -> &'static str {
        "GET accounts"
    }

    fn description(&self) -> &'static str {
        "Get you accounts in selected bank"
    }

    fn exec(&mut self, ctx_ref: Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();
        ensure_bank_selected(&ctx)?;

        let resp = get_with_params(API!("/account"), &ctx)?;
        let resp_s = handle_errors(resp)?;
        let yaml = json_to_yaml::<AccountsGetResp>(resp_s).unwrap();

        println!("You have the following accounts : \n {}", yaml);
        Ok(())
    }
}

pub struct TransacionAction {}

impl Action for TransacionAction {
    fn name(&self) -> &'static str {
        "TRANSACTION"
    }

    fn description(&self) -> &'static str {
        r#"
Perform transaction between your account and another arbitraty account 
in the bank system. Owner of an account should give you his bank's BIK
and account id.
        "#
    }

    fn exec(&mut self, ctx_ref: Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();
        ensure_bank_selected(&ctx)?;

        let acc_id = select_account(&ctx)?;

        let dst_endpoint =
            TransactionEndPoint::input("Input the destination of transaction: \n", 0)
                .ok_or("Cancelled".to_string())?;

        let amount =
            i32::input("Input amount of money (BYN) : ", 0).ok_or("Cancelled".to_string())?;

        let transaction_req = Transaction {
            src: TransactionEndPoint {
                bik: ctx.bik.unwrap(),
                account_id: acc_id,
            },

            dst: dst_endpoint,
            amount: Money(amount),
        };

        let transaction_resp = post_with_params(
            API!("/transaction"),
            serde_json::to_string(&transaction_req).unwrap(),
            &ctx,
        )?;

        handle_errors(transaction_resp)?;

        Ok(())
    }
}

pub struct DepositOpen {}

impl Action for DepositOpen {
    fn name(&self) -> &'static str {
        "OPEN deposit"
    }

    fn description(&self) -> &'static str {
        r#"Input all requested data to open a deposit in selected
bank. The money will be withdrawn from specified account
and transferred to bank's account. After specified amount
of time you will be able to withdraw you money back"#
    }

    fn exec(&mut self, ctx_ref: Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();
        ensure_bank_selected(&ctx)?;
        let deposit_req =
            DepositNewRequest::input("Input deposit parameters : ", 0).ok_or("Wrong input")?;

        let resp = post_with_params(
            API!("/deposit/new"),
            serde_json::to_string(&deposit_req).expect("Unserializable"),
            &ctx,
        )?;

        let _ = handle_errors(resp)?;
        Ok(())
    }
}

pub struct DepositGet {}

impl Action for DepositGet {
    fn name(&self) -> &'static str {
        "GET deposits"
    }

    fn description(&self) -> &'static str {
        "Below all deposits you've opened in currently seleteed bank will be printed."
    }

    fn exec(&mut self, ctx_ref: Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();
        ensure_bank_selected(&ctx)?;

        let resp = get_with_params(API!("/deposit"), &ctx)?;
        let resp_s = handle_errors(resp)?;
        let yaml = json_to_yaml::<Vec<Deposit>>(resp_s).ok_or("Server sent wrong response")?;
        println!("Deposits : \n{}", yaml);
        Ok(())
    }
}

pub struct DepositWithdrawAction {}

impl Action for DepositWithdrawAction {
    fn name(&self) -> &'static str {
        "WITHDRAW deposit"
    }

    fn description(&self) -> &'static str {
        r#"Withdraw specified deposit. You can perform this only after
it was expired"#
    }

    fn exec(&mut self, ctx_ref: Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();
        ensure_bank_selected(&ctx)?;

        println!("Select destination account for deposit");
        let dst_account = select_account(&ctx)?;

        let deposits_s = handle_errors(get_with_params(API!("/deposit"), &ctx)?)?;
        let deposits: Vec<Deposit> =
            serde_json::from_str(&deposits_s).map_err(|_| "Server sent bad request".to_string())?;
        let deposit_idx = select_idx(&deposits).ok_or("Wrong input")?;

        let req = DepositWithdrawRequest {
            deposit_idx,
            dst_account,
        };
        let resp = post_with_params(
            API!("/deposit/withdraw"),
            serde_json::to_string(&req).expect("Unserializable"),
            &ctx,
        )?;

        let resp_s = handle_errors(resp)?;

        println!("Withdrawn : {} BYN", resp_s);

        Ok(())
    }
}

pub struct CreditNewAction {}

impl Action for CreditNewAction {
    fn name(&self) -> &'static str {
        "NEW credit"
    }

    fn description(&self) -> &'static str {
        "Request a credit from a bank. You will be given the money after it will be accepted."
    }

    fn exec(&mut self, ctx_ref: Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();

        ensure_bank_selected(&ctx)?;

        let src_acc = select_account(&ctx)?;

        let req = CreditParams{
            amount : Money(i32::input("Amount of money : ", 0).ok_or("Wrong input")?),
            interest_rate : u8::input("Interest rate : ", 0).ok_or("Wrong input")?,
            term : u8::input("Term : ", 0).ok_or("Wrong input")?,
            src_account:src_acc
        };


        let resp = post_with_params(API!("/credit/new"),
                                serde_json::to_string(&req).unwrap(),
                                &ctx)?;
        handle_errors(resp)?;

        Ok(())
    }
}


pub struct CreditGetAction {}

impl Action for CreditGetAction {
    fn name(&self) -> &'static str {
        "GET credits"
    }

    fn description(&self) -> &'static str { 
        "GET all your active credits in selected bank"
    }


    fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {


        let ctx = ctx_ref.lock().unwrap();

        // ensure_bank_selected(&ctx)?;

        let resp = get_with_params(API!("/credit"), &ctx)?;
        let resp_s = handle_errors(resp)?;

        let yaml = json_to_yaml::<Vec<Credit>>(resp_s).ok_or(
            "Server sent wrong response".to_string()
        )?;

        println!("Currently active credits :\n{}", yaml);
        Ok(())
    }
}



