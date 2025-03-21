use crate::menu::Action;
use std::sync::{Arc, Mutex};
use crate::client::ClientContext;
use crate::utils::*;
use l1::common::transaction::*;
use crate::selector::*;
use std::io::Write;
use l1::common::salary::*;

pub fn flush(){
    std::io::stdout().flush().unwrap();
}


pub struct TransactionsGetAction {}


impl Action for TransactionsGetAction {
    fn name(&self) -> &'static str {
        "GET transactions"
    }


    fn description(&self) -> &'static str {
        "Get transactions along the finance system"
    }


    fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();

        let resp = get_with_params(API!("/transaction"), &ctx)?;
        let resp_s = handle_errors(resp)?;
        let yaml = json_to_yaml::<Vec<Transaction>>(resp_s).ok_or(
            "Server sent wrong response".to_string()
        )?;

        println!("Transaction list : \n\n{}", yaml);
        Ok(())
    }
}


pub struct TransactionsRevertAction {}


impl Action for TransactionsRevertAction {
    fn name(&self) -> &'static str {
        "REVERT transaction"
    }

    fn description(&self) -> &'static str {
        "Revert transaction"
    }


    fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();
        
        print!("Please print `yes` to confirm the revert action.");
        let mut s = String::new();
        std::io::stdin().read_line(&mut s);
        if s != "yes\n"{
            return Err("Cancelled".to_string());
        }


        let resp = post_with_params(API!("/transaction/revert"),
                                    String::new(),
                                    &ctx)?;
        let _ = handle_errors(resp)?;


        Ok(())
    }

}


pub struct SalaryAcceptProjAction {}

impl Action for SalaryAcceptProjAction {
    fn name(&self) -> &'static str {
        "ACCEPT salary project"
    }


    fn description(&self) -> &'static str {
        "Accept salary project"
    }

    fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();
        let requests_resp = get_with_params(API!("/salary/accept_proj"), &ctx)?;
        let requests : Vec<SalaryProjectResp> = serde_json::from_str(&handle_errors(requests_resp)?)
            .map_err(|_| "Server sent wrong response".to_string())?;

        let idx = select_idx(&requests).ok_or("Cancelled".to_string())?;


        let accept_req = SalaryAcceptProjRequest {
            enterprise : requests[idx].enterprise.clone()
        };

        let resp = post_with_params(API!("/salary/accept_proj"), 
            serde_json::to_string(&accept_req).unwrap(), 
            &ctx)?;

        handle_errors(resp)?;
        Ok(())
    }


}
