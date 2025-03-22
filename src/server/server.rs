use l1::common::account::*;
use l1::common::auth::*;
use l1::common::bank::BIK;
use l1::common::credit::*;
use l1::common::deposit::{DepositNewRequest, DepositWithdrawRequest, DepositWithdrawResponse};
use l1::common::salary::*;
use l1::common::time::TimeAdvanceReq;
use l1::common::transaction::Transaction;
use l1::common::user::UserData;
use l1::common::user::*;

use crate::runner::ServerRunner;
use crate::services::auth::AuthService;
use crate::services::bank::BankService;
use crate::services::time::TimeService;
use crate::traits::dynamic::Dynamic;
use crate::traits::storable::Storable;

use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Utc};

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

const RUNNER_SLEEP_TIME: u64 = 24 * 60 * 60; // 24 hours

pub fn deserialize_request<T>(body: &Request) -> Result<T, ServerError>
where
    T: serde::de::DeserializeOwned,
{
    let body = body.data().expect("Body already retrieved. Server error.");

    serde_json::from_reader(body).map_err(|err: serde_json::Error| {
        error!("Unable to deserialize request : {}", err);
        ServerError::BadRequest("Unable to deserialize request".to_string())
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
    time: Arc<Mutex<TimeService>>,
    dynamic_runner: ServerRunner,
}

impl Server {
    pub fn new() -> Arc<Mutex<Self>> {
        let auth = Arc::new(Mutex::new(AuthService::new()));
        let time = Arc::new(Mutex::new(TimeService::new()));
        let banks = Mutex::new(BankService::new(auth.clone(), time.clone()));
        let server = Arc::new(Mutex::new(Server {
            auth,
            banks,
            time,
            dynamic_runner: ServerRunner::new(),
        }));

        server
            .lock()
            .expect("Mutex")
            .dynamic_runner
            .run(&server, Duration::from_secs(RUNNER_SLEEP_TIME));
        server.clone()
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
                    let login_data: LoginReq = deserialize_request(&req)?;
                    let session_info = auth
                        .init_session(login_data)
                        .map_err(|err: &str| ServerError::Forbidden(err.to_string()))?;
                    Ok(Response::json(&session_info))
                }

                APIV1!("/auth/register") => {
                    let register_data: RegisterUserReq = deserialize_request(&req)?;
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
            CLIENT => self.handle_client(req, params),
            MANAGER => self.handle_manager(req, params),
            ENTERPRISE => self.handle_enterprise_specialist(req, params),
            OPERATOR => self.handle_operator(req, params),
            _ => Err(ServerError::BadRequest("Bad user".to_string())),
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
                    let usr_info = &auth
                        .get_user_by_token(token)
                        .ok_or(ServerError::BadRequest("Bad token".to_string()))?
                        .public_user;

                    if let UserData::ClientData(client) = usr_info {
                        Ok(Response::json(&client))
                    } else {
                        Err(ServerError::InternalError("No client data".to_string()))
                    }
                }
                APIV1!("/banks") => {
                    let banks_service = self.banks.lock().expect("Mutex");
                    let resp = banks_service.banks_get();
                    Ok(Response::json(&resp))
                }

                APIV1!("/account") => {
                    let banks_service = self.banks.lock().expect("Mutex");
                    let accounts_resp = banks_service.accounts_get(params)?;
                    Ok(Response::json(&accounts_resp))
                }

                APIV1!("/deposit") => {
                    let banks_service = self.banks.lock().expect("Mutex");
                    let deposits = banks_service.deposits_get(params)?;
                    Ok(Response::json(&deposits))
                }

                APIV1!("/credit") => {
                    let banks_service = self.banks.lock().expect("Mutex");
                    let credits = banks_service.credit_get(params)?;
                    Ok(Response::json(&credits))
                }

                _ => Err(ServerError::NotFound("".to_string())),
            },

            "POST" => match req.url().as_str() {
                APIV1!("/account/open") => {
                    let mut banks_service = self.banks.lock().expect("Mutex");
                    let resp = banks_service.account_open(params)?;
                    Ok(Response::json(&resp))
                }
                APIV1!("/account/close") => {
                    let mut banks_service = self.banks.lock().expect("Mutex");
                    let close_req: AccountCloseReq = deserialize_request(req)?;
                    banks_service.account_close(close_req, params)?;
                    Ok(Response::text("Ok"))
                }
                APIV1!("/deposit/new") => {
                    let mut banks_service = self.banks.lock().expect("Mutex");
                    let deposit_new_req: DepositNewRequest = deserialize_request(req)?;
                    banks_service.deposit_new(deposit_new_req, params)?;
                    Ok(Response::text("Ok"))
                }
                APIV1!("/deposit/withdraw") => {
                    let mut banks_service = self.banks.lock().expect("Mutex");
                    let deposit_withdraw_req: DepositWithdrawRequest = deserialize_request(req)?;
                    banks_service.deposit_withdraw(deposit_withdraw_req, params)?;
                    Ok(Response::text("Ok"))
                }
                APIV1!("/deposit/possible") => {
                    unimplemented!()
                }
                APIV1!("/transaction") => {
                    let mut banks_service = self.banks.lock().expect("Mutex");
                    let transaction: Transaction = deserialize_request::<Transaction>(req)?;
                    banks_service.transaction(transaction, params)?;
                    Ok(Response::text("Ok"))
                }

                APIV1!("/credit/new") => {
                    let mut banks_service = self.banks.lock().expect("Mutex");
                    let new_req: CreditParams = deserialize_request(req)?;
                    banks_service.credit_new(new_req, params)?;
                    Ok(Response::text("Ok"))
                }
                APIV1!("/credit/clear") => {
                    // unimplemented!();
                    Ok(Response::text("unimplemented"))
                }

                APIV1!("/salary/request") => {
                    let mut banks_service = self.banks.lock().expect("Mutex");
                    let req: SalaryClientRequest = deserialize_request(req)?;
                    banks_service.salary_request(req, params)?;
                    Ok(Response::text("Ok"))
                }

                _ => Err(ServerError::NotFound("".to_string())),
            },

            _ => Ok(Response::text("Method not allowed").with_status_code(405)),
        }
    }

    pub fn handle_manager(
        &mut self,
        req: &Request,
        params: &RequestParams,
    ) -> Result<Response, ServerError> {
        let url = req.url();

        match req.method() {
            "GET" => match url.as_str() {
                APIV1!("/auth/accept") => {
                    let auth = self.auth.lock().expect("Mutex error");
                    let registration_requests = auth.get_registration_requests();
                    Ok(Response::json(&registration_requests))
                }
                APIV1!("/banks") => {
                    let banks_service = self.banks.lock().expect("Mutex");
                    let resp = banks_service.banks_get();
                    Ok(Response::json(&resp))
                }
                APIV1!("/time/get") => {
                    let time = self.time.lock().unwrap().get_time();
                    Ok(Response::json(&time))
                }
                APIV1!("/credit/accept") => {
                    let banks_service = self.banks.lock().expect("Mutex");
                    let credits = banks_service.credit_get_unaccepted(params)?;
                    Ok(Response::json(&credits))
                }
                _ => Err(ServerError::NotFound("".to_string())),
            },

            "POST" => match url.as_str() {
                APIV1!("/time/advance") => {
                    let advance_req: TimeAdvanceReq = deserialize_request(req)?;
                    self.time.lock().unwrap().set_time(&advance_req.time);
                    self.dynamic_runner.force_wakeup();
                    Ok(Response::text("Ok"))
                }
                APIV1!("/transaction") => {
                    let mut banks_service = self.banks.lock().expect("Mutex");
                    let transaction: Transaction = deserialize_request::<Transaction>(req)?;
                    banks_service.transaction_unprotected(transaction, params)?;
                    Ok(Response::text("Ok"))
                }
                APIV1!("/auth/accept") => {
                    let mut auth = self.auth.lock().expect("Mutex error");
                    let accept_registration: AcceptRegistrationReq = deserialize_request(&req)?;
                    auth.accept_registration_request(&accept_registration)
                        .map_err(|err: _| ServerError::BadRequest(err.to_string()))?;
                    Ok(Response::text("Ok").with_status_code(200))
                }
                APIV1!("/credit/accept") => {
                    let mut banks_service = self.banks.lock().expect("Mutex");
                    let accept_req: CreditAcceptRequest = deserialize_request(req)?;
                    banks_service.credit_accept(accept_req, params)?;
                    Ok(Response::text("Ok"))
                }
                _ => Err(ServerError::NotFound("".to_string())),
            },

            _ => Err(ServerError::MethodNotAllowed(
                "Method not allowed".to_string(),
            )),
        }
    }

    pub fn handle_operator(
        &mut self,
        req: &Request,
        params: &RequestParams,
    ) -> Result<Response, ServerError> {
        let url = req.url();

        match req.method() {
            "GET" => match url.as_str() {
                APIV1!("/transaction") => {
                    let banks = self.banks.lock().expect("Mutex");
                    let transactions = banks.transactions_get();
                    Ok(Response::json(transactions))
                }
                APIV1!("/salary/accept_proj") => {
                    let mut banks = self.banks.lock().expect("Mutex");
                    Ok(Response::json(&banks.get_accept_salary_proj()?))
                }
                _ => Err(ServerError::NotFound("".into())),
            },

            "POST" => match url.as_str() {
                APIV1!("/transaction/revert") => {
                    let mut banks = self.banks.lock().expect("Mutex");
                    banks.transaction_revert(params)?;
                    Ok(Response::text("Ok"))
                }
                APIV1!("/salary/accept_proj") => {
                    let mut banks = self.banks.lock().expect("Mutex");
                    let req: SalaryAcceptProjRequest = deserialize_request(req)?;
                    banks.accept_salary_proj(req)?;
                    Ok(Response::text("Ok"))
                }
                _ => Err(ServerError::NotFound("".into())),
            },
            _ => Err(ServerError::MethodNotAllowed("".into())),
        }
    }

    pub fn update(&mut self) {
        let mut banks = self.banks.lock().expect("Mutex");
        let time_service = self.time.lock().unwrap();
        let time = time_service.get_time();
        banks.update(&time);
    }

    pub fn handle_enterprise_specialist(
        &mut self,
        req: &Request,
        params: &RequestParams,
    ) -> Result<Response, ServerError> {
        let url = req.url();

        match req.method() {
            "GET" => match url.as_str() {
                APIV1!("/banks") => {
                    let banks_service = self.banks.lock().expect("Mutex");
                    let resp = banks_service.banks_get();
                    Ok(Response::json(&resp))
                }
                APIV1!("/account") => {
                    let banks_service = self.banks.lock().expect("Mutex");
                    let accounts_resp = banks_service.accounts_get(params)?;
                    Ok(Response::json(&accounts_resp))
                }
                APIV1!("/salary/accept") => {
                    let bank = self.banks.lock().unwrap();
                    let resp = bank.salary_accept_decline_get(params)?;
                    Ok(Response::json(resp))
                }
                APIV1!("/salary/proj") => {
                    let bank = self.banks.lock().unwrap();
                    let enterprise = self
                        .auth
                        .lock()
                        .unwrap()
                        .validate_authentification(params.token.unwrap(), ENTERPRISE)
                        .map_err(|e| ServerError::Forbidden(e.to_string()))?;
                    let resp = bank.get_salary_proj(enterprise)?;
                    Ok(Response::json(&resp))
                }
                _ => Err(ServerError::NotFound("".into())),
            },

            "POST" => match url.as_str() {
                APIV1!("/account/open") => {
                    let mut banks_service = self.banks.lock().expect("Mutex");
                    let resp = banks_service.account_open(params)?;
                    Ok(Response::json(&resp))
                }
                APIV1!("/account/close") => {
                    let mut banks_service = self.banks.lock().expect("Mutex");
                    let close_req: AccountCloseReq = deserialize_request(req)?;
                    banks_service.account_close(close_req, params)?;
                    Ok(Response::text("Ok"))
                }
                APIV1!("/salary/new") => {
                    let mut bank = self.banks.lock().unwrap();
                    let req: SalaryInitProjRequest = deserialize_request(req)?;
                    bank.init_salary_proj(req, params)?;
                    Ok(Response::text("Ok"))
                }
                APIV1!("/salary/accept") => {
                    let mut bank = self.banks.lock().unwrap();
                    let req: SalaryAcceptRequest = deserialize_request(req)?;
                    bank.salary_accept_decline(req, params)?;
                    Ok(Response::text("Ok"))
                }
                APIV1!("/transaction") => {
                    let mut banks_service = self.banks.lock().expect("Mutex");
                    let transaction: Transaction = deserialize_request::<Transaction>(req)?;
                    banks_service.transaction_unprotected(transaction, params)?;
                    Ok(Response::text("Ok"))
                }
                _ => Err(ServerError::NotFound("".into())),
            },

            _ => Err(ServerError::MethodNotAllowed(
                "Method not allowed".to_string(),
            )),
        }
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

impl Storable for Server {
    fn load(&mut self, _dir: &std::path::Path) {
        ()
    }
    fn store(&self, _dir: &std::path::Path) {
        ()
    }
}
