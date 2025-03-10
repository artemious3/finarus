use crate::inputtable::Inputtable;
use crate::menu::*;
use l1::common::auth::{
    AcceptRegistrationReq, SessionResponse, GetRegistrationsReq, LoginReq, RegisterUserReq,
};
use l1::common::user::{User, UserType};
use reqwest::StatusCode;

fn json_to_yaml<T>(str: String) -> Option<String>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let obj = serde_json::from_str::<T>(str.as_str()).ok()?;
    let yaml = serde_yaml::to_string(&obj).ok()?;
    Some(yaml)
}

macro_rules! API {
    ($url : literal) => {
        concat!("http://127.0.0.1:8080/api/v1", $url)
    };
}



macro_rules! get_and_print_yaml {

    ( $url:literal, $msg:literal, $resp_type:ty  ) => {
                let token = self.auth_info.as_ref().unwrap().token;
                let response = self
                    .http_client
                    .get(concat!(SERVER!(), $url))
                    .query(&[("token", token.to_string().as_str())])
                    .send()
                    .unwrap();
                let response_str = handle_errors(response)?;
                let yaml = json_to_yaml::<$resp_type>(response_str)
                    .ok_or("Server sent wrong response")?;
                println!($msg);
                println!("{:-^20}", "");
                println!("{}", yaml);
                println!("{:-^20}", "");
                Ok(())
    }

}




type AuthInfo = SessionResponse;

pub struct Client {
    auth_info: Option<AuthInfo>,
    http_client: reqwest::blocking::Client,
}

fn handle_errors(response: reqwest::blocking::Response) -> Result<String, String> {
    match response.status() {
        StatusCode::OK => {
            println!("Success");
            Ok(response.text().unwrap())
        }
        _ => Err(response.text().unwrap()),
    }
}

impl Client {
    pub fn new() -> Self {
        Client {
            auth_info: None,
            http_client: reqwest::blocking::Client::new(),
        }
    }

    pub fn auth_menu(&mut self) -> Menu {
        let mut auth_menu = Menu::new();
        auth_menu
            .set_name("BANK-CLI root menu")
            .set_description("Welcome to bank cli");
        let mut login_action = FnAction::new();
        login_action
            .set_name("Login")
            .set_description("Authorize as an existing user")
            .set_func(|| {
                let login_data = LoginReq::input("Please enter yout credentials. \n", 0)
                    .ok_or("Wrong input")?;
                let response = self
                    .http_client
                    .post(API!("/auth/login"))
                    .body(serde_json::to_string(&login_data).unwrap())
                    .send()
                    .map_err(|err: _| err.to_string())?;
                let resp_str = handle_errors(response)?;
                let token_data: SessionResponse =
                    serde_json::from_str(&resp_str).map_err(|_| "Server sent wrong response")?;
                self.auth_info = Some(token_data);
                println!("Successfully authorized as {}\n", login_data.login);
                Ok(())
            });
        auth_menu.add_action('l' as u8, Box::new(login_action));

        let mut register_action = FnAction::new();
        register_action
            .set_name("Register")
            .set_description("Register as new user")
            .set_func(|| {
                let register_data = RegisterUserReq::input("Register as a new user : \n", 0)
                    .ok_or("Wrong input")?;
                let response = self
                    .http_client
                    .post(API!("/auth/register"))
                    .body(serde_json::to_string(&register_data).unwrap())
                    .send()
                    .map_err(|err: _| err.to_string())?;
                let _ = handle_errors(response)?;
                Ok(())
            });
        auth_menu.add_action('r' as u8, Box::new(register_action));
        auth_menu
    }

    pub fn user_type(&self) -> Option<UserType> {
        Some(self.auth_info.as_ref()?.user_type)
    }

    pub fn client_menu(&self) -> Menu {
        assert!(self.auth_info.is_some());
        let mut root = Menu::new();
        let mut auth_info_action = FnAction::new();
        auth_info_action
            .set_name("Info about current user")
            .set_description("Obtain personal info about current user")
            .set_func(|| {
                let token = self.auth_info.as_ref().unwrap().token;
                let response = self
                    .http_client
                    .get( API!("/auth"))
                    .query(&[("token", token.to_string().as_str())])
                    .send()
                    .unwrap();
                let response_str = handle_errors(response)?;
                let yaml =
                    json_to_yaml::<User>(response_str).ok_or("Server sent wrong response")?;
                println!("{}", yaml);
                Ok(())
            });
        root.add_action('i' as u8, Box::new(auth_info_action));

        root
    }

    pub fn manager_menu(&self) -> Menu {
        assert!(self.auth_info.is_some());

        let mut root = Menu::new();
        let mut get_reg_requests = FnAction::new();
        get_reg_requests
            .set_name("Get registration requests")
            .set_description("Get registration requests")
            .set_func(|| {
                let token = self.auth_info.as_ref().unwrap().token;
                let response = self
                    .http_client
                    .get( API!("/auth/accept"))
                    .query(&[("token", token.to_string().as_str())])
                    .send()
                    .unwrap();
                let response_str = handle_errors(response)?;
                let yaml = json_to_yaml::<Vec<GetRegistrationsReq>>(response_str)
                    .ok_or("Server sent wrong response")?;
                println!("Below is the list of users, requested registration\n");
                println!("{:-^20}", "");
                println!("{}", yaml);
                println!("{:-^20}", "");
                Ok(())
            });
        root.add_action('g' as u8, Box::new(get_reg_requests));

        let mut accept_reg_requests = FnAction::new();
        accept_reg_requests
            .set_name("Accept registration requests")
            .set_description("Accept registration requests")
            .set_func(|| {
                let accept_login = AcceptRegistrationReq::input("Input the login of the user to be accepted: \n", 0)
                    .ok_or("Wrong input")?;
                let token = self.auth_info.as_ref().unwrap().token;
                let response = self
                    .http_client
                    .post( API!("/auth/accept"))
                    .query(&[("token", token.to_string().as_str())])
                    .body(serde_json::to_string(&accept_login).unwrap())
                    .send()
                    .unwrap();
                let _ = handle_errors(response)?;
                Ok(())
            });
        root.add_action('a' as u8, Box::new(accept_reg_requests));
        root
    }
}
