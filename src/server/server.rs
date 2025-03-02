use crate::services::auth::AuthService;
use crate::services::bank::BankService;
use l1::common::auth::*;
use l1::common::bank::BIK;
use l1::common::deposit::{DepositNewRequest, DepositWithdrawRequest, DepositWithdrawResponse};
use l1::common::user::UserType;
use crate::traits::dynamic::Dynamic;
use std::str::FromStr;

use log::*;

use std::sync::{Arc, Mutex};

use log::error;
use rouille::{Request, Response};

pub enum ServerError {
    BadRequest(String),
    Forbidden(String),
    InternalError(String),
    NotFound(String),
    MethodNotAllowed(String),
}

pub fn deserialize_request<T>(body: &Request) -> Result<T, &str>
where
    T: serde::de::DeserializeOwned,
{
    let body = body.data().expect("Body already retrieved. Server error.");

    serde_json::from_reader(body).map_err(|err: serde_json::Error| {
        error!("Unable to deserialize request : {}", err);
        "Unable to deserialize request"
    })
}

pub struct RequestParams {
    pub token: Option<Token>,
    pub bik: Option<BIK>,
}

fn parse_param<T>(param: &str, req: &Request) -> Option<T>
where
    T: FromStr,
{
    let s = req.get_param(param)?;
    s.parse::<T>().ok()
}

impl From<&Request> for RequestParams {
    fn from(req: &Request) -> Self {
        RequestParams {
            token: parse_param("token", req),
            bik: parse_param("bank", req),
        }
    }
}

macro_rules! APIV1 {
    ($url : literal) => {
        concat!("/api/v1", $url)
    };
}

fn map_err_to_response(opt_response: Result<Response, ServerError>) -> Response {
    match opt_response {
        Ok(response) => response,
        Err(ServerError::Forbidden(s)) => {
            error!("Forbidden access.");
            Response::text(format!("Forbidden : {}", s)).with_status_code(403)
        }
        Err(ServerError::BadRequest(s)) => {
            error!("Bad request.");
            Response::text(format!("Bad request : {}", s)).with_status_code(400)
        }
        Err(ServerError::InternalError(s)) => {
            error!("Internal server error.");
            Response::text(format!("Server error : {}", s)).with_status_code(500)
        }
        Err(ServerError::NotFound(s)) => {
            error!("Not found");
            Response::text(format!("Not found : {}", s)).with_status_code(404)
        }
        Err(ServerError::MethodNotAllowed(s)) => {
            error!("Method not allowed");
            Response::text(format!("Not allowed : {}", s)).with_status_code(405)
        }
    }
}

pub struct Server {
    auth: Arc<Mutex<AuthService>>,
    banks: Mutex<BankService>,
}

impl Server {
    pub fn new() -> Self {
        let auth = Arc::new(Mutex::new(AuthService::new()));
        Server {
            auth: auth.clone(),
            banks: Mutex::new(BankService::new(auth.clone())),
        }
    }

    pub fn handle_request(&mut self, req: &Request) -> Response {
        map_err_to_response(self.handle_request_or_error(req))
    }

    pub fn handle_request_or_error(&mut self, req: &Request) -> Result<Response, ServerError> {
        let params = RequestParams::from(req);
        let maybe_token = params.token;

        match maybe_token {
            // if no token, user can only authentificate or register
            None => self.handle_no_user(req, &params),
            Some(_) => self.handle_user(req, &params),
        }
    }

    pub fn handle_no_user(
        &mut self,
        req: &Request,
        _params: &RequestParams,
    ) -> Result<Response, ServerError> {
        let mut auth = self.auth.lock().expect("Mutex error");

        if req.method() == "POST" {
            match req.url().as_str() {
                APIV1!("/auth/login") => {
                    let login_data: LoginReq = deserialize_request(&req)
                        .map_err(|_: &str| ServerError::BadRequest("Bad request".to_string()))?;
                    let session_info = auth
                        .init_session(login_data)
                        .map_err(|err: &str| ServerError::Forbidden(err.to_string()))?;
                    Ok(Response::json(&session_info))
                }

                APIV1!("/auth/register") => {
                    let register_data: RegisterUserReq =
                        deserialize_request(&req).map_err(|_: &str| {
                            ServerError::BadRequest("Invalid registration data".to_string())
                        })?;
                    auth.request_add_user(register_data)
                        .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;
                    Ok(Response::text("Ok").with_status_code(200))
                }

                _ => Err(ServerError::NotFound(String::new())),
            }
        } else {
            Err(ServerError::NotFound(String::new()))
        }
    }

    pub fn handle_user(
        &mut self,
        req: &Request,
        params: &RequestParams,
    ) -> Result<Response, ServerError> {
        let token = params.token.expect("`handle_user` called without user");
        let usr_type = {
            let auth = self.auth.lock().expect("Mutex error");
            let usr = auth
                .get_user_by_token(token)
                .ok_or(ServerError::Forbidden("Bad token".to_string()))?;
            usr.user_type
        };

        match usr_type {
            UserType::Client => self.handle_client(req, params),
            UserType::Manager => self.handle_manager(req, params),
            UserType::EnterpriseSpecialist => sefl
            _ => unimplemented!(),
        }
    }

    pub fn handle_client(
        &mut self,
        req: &Request,
        params: &RequestParams,
    ) -> Result<Response, ServerError> {
        let token = params.token.expect("`client` with no token");

        match req.method() {
            "GET" => match req.url().as_str() {

                APIV1!("/auth") => {
                    let auth = self.auth.lock().expect("Mutex error");
                    let usr_info = auth
                        .get_user_by_token(token)
                        .ok_or(ServerError::BadRequest("Bad token".to_string()))?
                        .public_user
                        .clone()
                        .expect("Not a client");
                    Ok(Response::json(&usr_info))
                }

                _ => unimplemented!(),
            },

            "POST" => match req.url().as_str() {
                APIV1!("/account/open") => {
                    unimplemented!();
                }
                APIV1!("/account/close") => {
                    unimplemented!();
                }
                APIV1!("/deposit/new") => {
                    let mut banks_service = self.banks.lock().expect("Mutex");
                    let deposit_new_req : DepositNewRequest = deserialize_request(req)
                        .map_err(|_| ServerError::BadRequest("".to_string()))?;
                    banks_service.deposit_new(deposit_new_req, params)?;
                    Ok(Response::text("Ok"))
                },
                APIV1!("/deposit/withdraw") => {
                    let mut banks_service = self.banks.lock().expect("Mutex");
                    let deposit_withdraw_req : DepositWithdrawRequest = deserialize_request(req)
                        .map_err(|_| ServerError::BadRequest("".to_string()))?;
                    banks_service.deposit_withdraw(deposit_withdraw_req, params)?;
                    Ok(Response::text("Ok"))
                }

                APIV1!("/credit/new") => {
                    unimplemented!();
                }
                APIV1!("/credit/clear") => {
                    unimplemented!();
                }


                _ => Err(ServerError::NotFound("".to_string())),
            },

            _ => Ok(Response::text("Method not allowed").with_status_code(405)),
        }
    }

    pub fn handle_manager(
        &mut self,
        req: &Request,
        _params: &RequestParams,
    ) -> Result<Response, ServerError> {
        let mut auth = self.auth.lock().expect("Mutex error");
        let url = req.url();

        match req.method() {
            "GET" => match url.as_str(){
                APIV1!("/auth/accept") => {
                    let registration_requests = auth.get_registration_requests();
                    Ok(Response::json(&registration_requests))
                }
                APIV1!("/credit/accept")=> {
                    unimplemented!()
                }
                _ => Err(ServerError::NotFound("".to_string())),
            },

            "POST" => match url.as_str() {
                APIV1!("/auth/accept") => {
                    let accept_registration: AcceptRegistrationReq = deserialize_request(&req)
                        .map_err(|_| ServerError::BadRequest("Bad request".to_string()))?;
                    auth.accept_registration_request(&accept_registration)
                        .map_err(|err: _| ServerError::BadRequest(err.to_string()))?;
                    Ok(Response::text("Ok").with_status_code(200))
                }
                APIV1!("/credit/accept") => {
                    unimplemented!()
                }
                _ => Err(ServerError::NotFound("".to_string())),
            },

            _ => Err(ServerError::MethodNotAllowed(
                "Method not allowed".to_string(),
            )),
        }
    }


    pub fn handle_enterprise_specialist(
        &mut self, 
        req: &Request,
        params : &RequestParams
    ) -> Result<Response, ServerError> {





    }

    // fn handle_get(&mut self, req: &Request) -> Result<Response, ServerError> {
    //     let req_params = RequestParams::from(req);
    //     let auth = self.auth.lock().expect("Mutex error");
    //     match req.url().as_str() {
    //
    //         APIV1!("/auth") => {
    //             // TODO : auth data for not clients
    //             let token = req_params
    //                 .token
    //                 .ok_or(ServerError::BadRequest("No token".to_string()))?;
    //             let resp_data = auth
    //                 .get_user_by_token(token)
    //                 .ok_or(ServerError::BadRequest("Bad token".to_string()))?;
    //             Ok(Response::json(&resp_data.public_user))
    //         }
    //
    //         "/auth/accept" => {
    //             let token = req_params
    //                 .token
    //                 .ok_or(ServerError::BadRequest("No token".to_string()))?;
    //             auth.validate_authentification(token, UserType::Manager)
    //                 .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;
    //             let registration_requests = &self.auth.get_registration_requests();
    //             Ok(Response::json(registration_requests))
    //         }
    //
    //         "/deposit" => {
    //             let token = req_params
    //                 .token
    //                 .ok_or(ServerError::BadRequest("No token".to_string()))?;
    //             auth.validate_authentification(token, UserType::Client)
    //                 .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;
    //
    //             let bank_service = self.banks.lock().expect("Mutex error");
    //             unimplemented!()
    //         }
    //         _ => Err(ServerError::NotFound("".to_string())),
    //     }
    // }
    //
    // fn handle_post(&mut self, req: &Request) -> Result<Response, ServerError> {
    //     let req_params = RequestParams::from(req);
    //     if req.url().starts_with("/auth") {
    //         let mut auth = self.auth.lock().expect("Mutex error");
    //
    //         match req.url().as_str() {
    //             "/auth/register" => {
    //                 let register_data: RegisterUserReq =
    //                     deserialize_request(&req).map_err(|_: &str| {
    //                         ServerError::BadRequest("Invalid registration data".to_string())
    //                     })?;
    //                 auth.request_add_user(register_data)
    //                     .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;
    //                 Ok(Response::text("Ok").with_status_code(200))
    //             }
    //
    //             "/auth/login" => {
    //                 let login_data: LoginReq = deserialize_request(&req)
    //                     .map_err(|_: &str| ServerError::BadRequest("Bad request".to_string()))?;
    //                 let session_info = auth
    //                     .init_session(login_data)
    //                     .map_err(|err: &str| ServerError::Forbidden(err.to_string()))?;
    //                 Ok(Response::json(&session_info))
    //             }
    //             "/auth/accept" => {
    //                 let token = req_params
    //                     .token
    //                     .ok_or(ServerError::BadRequest("No token".to_string()))?;
    //                 auth.validate_authentification(token, UserType::Manager)
    //                     .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;
    //                 let accept_registration: AcceptRegistrationReq = deserialize_request(&req)
    //                     .map_err(|_| ServerError::BadRequest("Bad request".to_string()))?;
    //                 auth.accept_registration_request(&accept_registration)
    //                     .map_err(|err: _| ServerError::BadRequest(err.to_string()))?;
    //                 Ok(Response::text("Ok").with_status_code(200))
    //             }
    //
    //             _ => Err(ServerError::NotFound("".to_string())),
    //         }
    //     } else {
    //         let mut bank_service = self.banks.lock().expect("Mutex error");
    //         let token = req_params
    //             .token
    //             .ok_or(ServerError::BadRequest("No token".to_string()))?;
    //         let auth = self.auth.lock().expect("Mutex error");
    //
    //         match req.url().as_str() {
    //             "/deposit/new" => {
    //                 auth.validate_authentification(token, UserType::Client)
    //                     .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;
    //                 let deposit_new_req: DepositNewRequest = deserialize_request(req)
    //                     .map_err(|_| ServerError::BadRequest("Bad depositr request".to_string()))?;
    //                 bank_service.deposit_new(deposit_new_req, &req_params)?;
    //                 Ok(Response::text("Ok"))
    //             }
    //
    //             "/deposit/withdraw" => {
    //                 unimplemented!()
    //             }
    //
    //             _ => Err(ServerError::NotFound("".to_string())),
    //         }
    //     }
    // }
}



impl Dynamic for Server {
    fn update(&mut self, time :  &chrono::DateTime<chrono::Utc>) {
        let mut banks = self.banks.lock().expect("Mutex");
        banks.update(time);
    }
}
