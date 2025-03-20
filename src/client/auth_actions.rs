
use crate::menu::Action;
use std::sync::{Arc, Mutex};
use crate::client::ClientContext;
use crate::utils::*;
use crate::inputtable::*;
use l1::common::auth::{LoginReq, SessionResponse, RegisterUserReq};


pub struct LoginAction {}

impl Action for LoginAction {
    fn name(&self) -> &'static str {
        "LOGIN into the system"
    }


    fn description(&self) -> &'static str {
        "Input login and password in order to login into the bank system"
    }
    

    fn exec(&mut self, ctx_ref: Arc<Mutex<ClientContext>> ) -> Result<(), String> {
                let mut ctx = ctx_ref.lock().expect("Mutex");
                let login_data =
                    LoginReq::input("Please enter yout credentials. \n", 0).ok_or("Wrong input")?;
                let response = post_with_params(
                    API!("/auth/login"),
                    serde_json::to_string(&login_data).unwrap(),
                    &ctx,
                )?;
                let resp_str = handle_errors(response)?;
                let token_data: SessionResponse =
                    serde_json::from_str(&resp_str).map_err(|_| "Server sent wrong response")?;
                ctx.auth_info = Some(token_data);
                ctx.login = Some(login_data.login.clone());
                println!("Successfully authorized as {}\n", login_data.login);
                Ok(())
    }
}

pub struct RegisterAction {}
impl Action for RegisterAction {

    fn name(&self) -> &'static str {
        "REGISTER as a new user"
    }


    fn description(&self) -> &'static str {
r#"
Input all data, requested by the system in order to get registered as CLIENT.
After manager will accept your request, you will be able to login in system"#
    }


    fn exec(&mut self, ctx_ref : Arc<Mutex<ClientContext>>) -> Result<(), String> {
                let ctx = ctx_ref.lock().expect("Mutex");
                let register_data = RegisterUserReq::input("Register as a new user : \n", 0)
                    .ok_or("Wrong input")?;
                let _ = post_with_params(
                    API!("/auth/register"),
                    serde_json::to_string(&register_data).unwrap(),
                    &ctx,
                )?;
                Ok(())
    }

}
