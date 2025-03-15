use crate::client::ClientContext;
use crate::inputtable::*;
use crate::menu::Action;
use crate::selector::select_from;
use crate::utils::*;
use l1::common::account::*;
use l1::common::transaction::{TransactionEndPoint, Transaction};
use l1::common::user::User;
use l1::common::bank:: BanksGetResp;
use l1::common::deposit::*;
use std::sync::{Arc, Mutex};

pub struct GetAuthInfoAction {}

fn ensure_bank_selected(ctx: &ClientContext) -> Result<(), String> {
    if ctx.bik.is_none() {
        return Err("Select the bank first".to_string());
    }
    Ok(())
}

impl Action for GetAuthInfoAction {
    fn name(&self) -> &'static str {
        "Info about current user"
    }

    fn description(&self) -> &'static str {
        "Obtain personal info of currently logged user"
    }

    fn exec(&mut self, ctx_ref: Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();
        let response = get_with_params(API!("/auth"), &ctx)?;
        let response_str = handle_errors(response)?;
        let yaml = json_to_yaml::<User>(response_str).ok_or("Server sent wrong response")?;
        println!("{}", yaml);
        Ok(())
    }
}

pub struct SelectBankAction {}

impl Action for SelectBankAction {
    fn name(&self) -> &'static str {
        "Select bank"
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
        "Open account"
    }

    fn description(&self) -> &'static str {
        "Open account in selected bank"
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
        "Get you accounts in selected banks"
    }

    fn description(&self) -> &'static str {
        "Get you accounts in selected banks"
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
        "Perform transaction"
    }

    fn description(&self) -> &'static str {
        concat!("Perform transaction between your account and another arbitraty account in the bank system.",
                "Owner of an account should give you his bank's BIK and account id")
    }

    fn exec(&mut self, ctx_ref: Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();
        ensure_bank_selected(&ctx)?;

        let resp = get_with_params(API!("/account"), &ctx)?;
        let resp_s = handle_errors(resp)?;
        let result: AccountsGetResp =
            serde_json::from_str(&resp_s).map_err(|_| "Wrong response".to_string())?;

        println!("Select source account for transaction");

        let acc_id = select_from(&result.accounts.iter().map(|acc| acc.id).collect())
            .ok_or("Cancelled".to_string())?;


        let dst_endpoint = TransactionEndPoint::input("Input the destination of transaction: \n", 0). 
            ok_or("Cancelled".to_string())?;


        let amount = i32::input("Input amount of money (BYN) : ", 0)
            .ok_or("Cancelled".to_string())?;


        let transaction_req = Transaction{
            src : TransactionEndPoint {
                bik : ctx.bik.unwrap(),
                account_id  : acc_id
            },

            dst : dst_endpoint,
            amount,
        };

        let transaction_resp = post_with_params(API!("/transaction"),
            serde_json::to_string(&transaction_req).unwrap(),
            &ctx)?;

        handle_errors(transaction_resp)?;

        Ok(())
    }
}


pub struct DepositOpen {}

impl Action for DepositOpen {
    fn name(&self) -> &'static str {
        "Open deposit"
    }

    fn description(&self) -> &'static str {
        "Open deposit"
    }

    fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();
        ensure_bank_selected(&ctx)?;
        let deposit_req = DepositNewRequest::input("Input deposit parameters : ", 0).ok_or("Wrong input")?;

        let resp = post_with_params(API!("/deposit/new"), 
                        serde_json::to_string(&deposit_req).expect("Unserializable"),
                        &ctx)?;

        let _ = handle_errors(resp)?;
        Ok(())
    }
}


pub struct DepositGet {}

impl Action for DepositGet {

    fn name(&self) -> &'static str {
        "Get deposits"
    }

    fn description(&self) -> &'static str {
        "Get bank deposits"
    }
     fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {

        let ctx = ctx_ref.lock().unwrap();
        ensure_bank_selected(&ctx)?;


        let resp = get_with_params(API!("/deposit"), &ctx)?;
        let resp_s = handle_errors(resp)?;
        let yaml = json_to_yaml::<Vec<Deposit>>(resp_s).ok_or("Server sent wrong response")?;
        println!("Deposits : \n{}", yaml);
        Ok(())

     }

}



