use l1::common::salary::*;
use l1::common::Money;
use l1::common::transaction::*;

use crate::client::ClientContext;
use crate::inputtable::*;
use crate::menu::*;
use crate::selector::*;
use crate::utils::*;
use std::sync::{Arc, Mutex};

pub struct SalaryAcceptAction {}

impl Action for SalaryAcceptAction {
    fn name(&self) -> &'static str {
        "ACCEPT/DECLINE salary"
    }

    fn description(&self) -> &'static str {
        "Accept or decline salary request"
    }

    fn exec(&mut self, ctx_ref: Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();

        let resp = get_with_params(API!("/salary/accept"), &ctx)?;
        let requests_s = handle_errors(resp)?;
        let requests: Vec<SalaryClientRequest> = serde_json::from_str(&requests_s)
            .map_err(|_| "Server sent wrong response".to_string())?;


        println!("Select salary request to accept or decline :\n");
        let idx = select_idx(&requests).ok_or("Cancelled".to_string())?;

        let accept = bool::input("Accept? [y/n] : ", 0).ok_or("Cancelled".to_string())?;
        if !accept {
            let resp = post_with_params(
                API!("/salary/accept"),
                serde_json::to_string(&SalaryAcceptRequest {
                    idx,
                    accept,
                    salary: Money(0),
                })
                .unwrap(),
                &ctx,
            )?;
            handle_errors(resp)?;
        } else {
            let salary = Money::input("Salary : ", 0).ok_or("Cancelled".to_string())?;

            let resp = post_with_params(
                API!("/salary/accept"),
                serde_json::to_string(&SalaryAcceptRequest {
                    idx,
                    accept,
                    salary,
                })
                .unwrap(),
                &ctx,
            )?;
            handle_errors(resp)?;
        }

        Ok(())
    }
}



pub struct SalaryInitProjectAction {}

impl Action for SalaryInitProjectAction {
    fn name(&self) -> &'static str {
        "INIT salary project"
    }

    fn description(&self) -> &'static str {
        "Create new salary project (only 1 for enterprise)"
    }


    fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();

        let acc = TransactionEndPoint::input("Account to pay salary : \n", 0).ok_or("Cancelled")?;

        let resp = post_with_params(API!("/salary/new"), serde_json::to_string(&SalaryInitProjRequest{
            account : acc
        }).unwrap(), &ctx)?;

        handle_errors(resp)?;

        Ok(())
    }
}


pub struct SalaryProjectGetAction {}

impl Action for SalaryProjectGetAction {
    fn name(&self) -> &'static str {
        "GET salary project"
    }

    fn description(&self) -> &'static str {
        "GET salary project"
    }


    fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();


        let resp = get_with_params(API!("/salary/proj"), &ctx)?;
        let proj = handle_errors(resp)?;
        let yaml = json_to_yaml::<SalaryProject>(proj).ok_or("Server sent wrong response".to_string())?;

        println!("Your salary project : \n{}", yaml);
        Ok(())
    }
}
