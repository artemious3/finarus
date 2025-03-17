
use crate::menu::Action;
use std::sync::{Arc, Mutex};
use crate::client::ClientContext;
use crate::utils::*;
use crate::inputtable::*;
use l1::common::auth::{GetRegistrationsReq, AcceptRegistrationReq};
use l1::common::time::TimeAdvanceReq;
use l1::common::credit::{CreditUnaccepted, CreditAcceptRequest};
use crate::selector::{select_idx, select_from};
use chrono::{DateTime, Utc};


fn ensure_bank_selected(ctx: &ClientContext) -> Result<(), String> {
    if ctx.bik.is_none() {
        return Err("Select the bank first".to_string());
    }
    Ok(())
}

pub struct AcceptRegistrationRequestsAction {}

impl Action for AcceptRegistrationRequestsAction {
    fn name(&self) -> &'static str {
        "ACCEPT REGISTRATION requests"
    }
    fn description(&self) -> &'static str {
        "Accept registration requests"
    }
    fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {
        
                let ctx = ctx_ref.lock().unwrap();
                let resp = get_with_params(API!("/auth/accept"), &ctx)?;
                let response_str = handle_errors(resp)?;
                let options : Vec<GetRegistrationsReq> = serde_json::from_str(&response_str).map_err(
                    |_| "Server sent wrong response".to_string()
                )?;

                let accept_login_idx = select_idx(&options).ok_or("Cancelled")?;
                let accept_login = AcceptRegistrationReq{login : options[accept_login_idx].login.clone()};

                let resp = post_with_params(
                    API!("/auth/accept"),
                    serde_json::to_string(&accept_login).unwrap(),
                    &ctx,
                )?;
                let _ = handle_errors(resp)?;
                Ok(())
    }
}


pub struct AdvanceTimeAction {}

impl Action for AdvanceTimeAction {
    fn description(&self) -> &'static str {
        "Change server local time for all dynamic services to be recalculated."
    }
    fn name(&self) -> &'static str {
        "CHANGE server local TIME"
    }

    fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();

        let time = DateTime::input("Input time to advance : ", 0).ok_or("Cancelled")?;
        let req = TimeAdvanceReq{time};

        let resp = post_with_params(API!("/time/advance"), 
            serde_json::to_string(&req).expect("Unserializable"),
                        &ctx)?;
        let _ = handle_errors(resp)?;
        Ok(())
    }
}


pub struct GetTimeAction {}

impl Action for GetTimeAction {
    fn description(&self) -> &'static str {
        "GET server local TIME"
    }
    fn name(&self) -> &'static str {
        "Get server local time"
    }

    fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {
        let ctx = ctx_ref.lock().unwrap();

        let resp = get_with_params(API!("/time/get"),
                                    &ctx)?;
        let resp_s = handle_errors(resp)?;

        println!("Time : \n{}", resp_s);
        Ok(())
    }
}


pub struct CreditAcceptAction {}

impl Action for CreditAcceptAction {
    fn name(&self) -> &'static str {
        "ACCEPT credit"
    }

    fn description(&self) -> &'static str {
        "Accept credit"
    }


    fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {

        let ctx = ctx_ref.lock().unwrap();
        ensure_bank_selected(&ctx)?;

        let resp = get_with_params(API!("/credit/accept"),
                                    &ctx)?;
        let resp_s = handle_errors(resp)?;
        let credits : Vec<CreditUnaccepted> = serde_json::from_str(&resp_s).map_err(
            |_| "Server send wrong response".to_string()
        )?;

        let idx = select_idx(&credits).ok_or("Wrong input")?;

        let resp = post_with_params(API!("/credit/accept"),
                        serde_json::to_string(&CreditAcceptRequest{idx}).unwrap(),
                        &ctx)?;
        handle_errors(resp)?;
        Ok(())
    }
}

