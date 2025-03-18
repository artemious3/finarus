use crate::menu::Action;
use std::sync::{Arc, Mutex};
use crate::client::ClientContext;
use crate::utils::*;
use l1::common::transaction::*;
use std::io::Write;

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


