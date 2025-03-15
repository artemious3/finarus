
use crate::menu::Action;
use std::sync::{Arc, Mutex};
use crate::client::ClientContext;
use crate::utils::*;
use crate::inputtable::*;
use l1::common::auth::{GetRegistrationsReq, AcceptRegistrationReq};
use l1::common::time::TimeAdvanceReq;
use chrono::{DateTime, Utc};



pub struct GetRegistrationRequestsAction {}

impl Action for GetRegistrationRequestsAction{
            fn name(&self) -> &'static str {
                "Get registration requests"
            }
            fn description(&self) -> &'static str {
                "Get registration requests"
            }
            fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {
                let ctx = ctx_ref.lock().unwrap();
                let resp = get_with_params(API!("/auth/accept"), &ctx)?;
                let response_str = handle_errors(resp)?;
                let yaml = json_to_yaml::<Vec<GetRegistrationsReq>>(response_str)
                    .ok_or("Server sent wrong response")?;
                println!("Below is the list of users, requested registration\n");
                println!("{:-^20}", "");
                println!("{}", yaml);
                println!("{:-^20}", "");
                Ok(())
            }
}


pub struct AcceptRegistrationRequestsAction {}

impl Action for AcceptRegistrationRequestsAction {
    fn name(&self) -> &'static str {
        "Accept registration requests"
    }
    fn description(&self) -> &'static str {
        "Accept registration requests"
    }
    fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {
                let ctx = ctx_ref.lock().unwrap();
                let accept_login = AcceptRegistrationReq::input(
                    "Input the login of the user to be accepted: \n",
                    0,
                )
                .ok_or("Wrong input")?;
                let response = post_with_params(
                    API!("/auth/accept"),
                    serde_json::to_string(&accept_login).unwrap(),
                    &ctx,
                )?;
                let _ = handle_errors(response)?;
                Ok(())
    }
}


pub struct AdvanceTimeAction {}

impl Action for AdvanceTimeAction {
    fn description(&self) -> &'static str {
        "Change server local time for all dynamic services to be recalculated."
    }
    fn name(&self) -> &'static str {
        "Change server local time"
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
        "Get server local time"
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


