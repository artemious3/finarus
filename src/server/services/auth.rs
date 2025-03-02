use std::collections::{hash_map::Entry, HashMap};

use crate::user::InternalUser;
use l1::common::auth::*;
use l1::common::user::UserType;
use rand::prelude::Rng;

use sha2::Digest;
use std::option::Option;
use std::convert::Into;


use log::{error, info};

use std::string::ToString;

pub struct AuthService {
    sessions: HashMap<Token, Login>,
    registration_requests: HashMap<Login, InternalUser>,
    users: HashMap<Login, InternalUser>,
}

#[derive(PartialEq)]
enum LoginDataStatus {
    InvalidLogin,
    InvalidPassword,
    NotAccepted,
    Valid,
}
impl Into<&str> for LoginDataStatus {
    fn into(self) -> &'static str {
        match self {
            LoginDataStatus::Valid => "Ok",
            LoginDataStatus::NotAccepted => "Not accepted",
            LoginDataStatus::InvalidLogin => "Invalid login",
            LoginDataStatus::InvalidPassword => "Incalid password",
        }
    }
}

impl AuthService {
    pub fn new() -> Self {
        let mut service = AuthService {
            sessions: HashMap::new(),
            registration_requests: HashMap::new(),
            users: HashMap::new(),
        };

        // TMP!!!
        let mut hasher = sha2::Sha256::new();
        hasher.update("manager");
        let hash = format!("{:x}", hasher.finalize());
        service.users.insert(
            "manager".to_string(),
            InternalUser {
                user_type: UserType::Manager,
                login: "manager".to_string(),
                password_hash: hash,
                public_user: None,
            },
        );
        // TMP!!!

        service
    }

    pub fn validate_authentification(&self, token: Token, role: UserType) -> Result<Login, String> {
        let usr = self
            .get_user_by_token(token)
            .ok_or("No session with given token".to_string())?;
        if usr.user_type != role {
            Err("Permission denied".to_string())
        } else {
            Ok(usr.login.clone())
        }
    }

    fn validate_login_data(&self, login_data: &LoginReq) -> LoginDataStatus {
        let usr_opt = self.users.get(&login_data.login);
        match usr_opt {
            Some(usr)=>{
                let mut hasher = sha2::Sha256::new();
                hasher.update(login_data.password.as_str());
                let hash = hasher.finalize();
                if format!("{:x}", hash) == usr.password_hash {
                    LoginDataStatus::Valid
                } else {
                    LoginDataStatus::InvalidPassword
                }
            }
            None => {
                if self.registration_requests.contains_key(&login_data.login){
                    LoginDataStatus::NotAccepted
                } else {
                    LoginDataStatus::InvalidLogin
                }
            }
        }
    }

    pub fn init_session(&mut self, login_data: LoginReq) -> Result<SessionResponse, &str> {
        let login_data_status = self.validate_login_data(&login_data);
    
        if login_data_status == LoginDataStatus::Valid {
            let mut rnd = rand::rng();
            let new_token = rnd.random::<u64>();
            match self.sessions.entry(new_token) {
                Entry::Vacant(entry) => {
                    entry.insert(login_data.login.clone());
                    let user_type = self.users.get(&login_data.login).unwrap().user_type;
                    info!(
                        "User `{}` initiated a session. Token : {}",
                        login_data.login.as_str(),
                        new_token
                    );
                    Ok(SessionResponse {
                        token: new_token,
                        user_type: user_type,
                    })
                }
                Entry::Occupied(_) => {
                    error!("You are really lucky! This token already exists!");
                    Err("Token already exists. Just try again.")
                }
            }
        } else {
            Err(login_data_status.into())
        }
    }

    pub fn get_user_by_token(&self, token: Token) -> Option<&InternalUser> {
        self.sessions
            .get(&token)
            .map_or(None, |login: &String| Some(self.users.get(login).unwrap()))
    }

    pub fn request_add_user(
        &mut self,
        user: l1::common::auth::RegisterUserReq,
    ) -> Result<(), &str> {
        if self.users.contains_key(&user.login_data.login) {
            info!(
                "Attempt to register another user with login {}",
                user.login_data.login.as_str()
            );
            Err("This login already exists")
        } else {
            let mut hasher = sha2::Sha256::new();
            hasher.update(&user.login_data.password);
            let hash = format!("{:x}", hasher.finalize());
            let internal_user = InternalUser {
                user_type: UserType::Client, // by default only client is manually registered.
                login: user.login_data.password,
                password_hash: hash,
                public_user: Some(user.user_data),
            };
            info!(
                "Requested to add new user with login {}",
                internal_user.login.as_str()
            );
            self.registration_requests
                .insert(internal_user.login.clone(), internal_user);
            Ok(())
        }
    }

    pub fn get_registration_requests(&self) -> Vec<GetRegistrationsReq> {
        // (Login, InternalUser) => (Login, User)
        self.registration_requests.iter().map(|kv : _|{
            GetRegistrationsReq {
                login : kv.0.clone(),
                user : kv.1.public_user.clone().expect("This is a special user (Admin, Manager...), which cannot be requested to register")
            }
        }).collect()
    }

    pub fn accept_registration_request(&mut self, req: &AcceptRegistrationReq) -> Result<(), &str> {
        let user = self
            .registration_requests
            .remove(&req.login)
            .ok_or("No registration requests with given login")?;
        self.users.insert(req.login.clone(), user);
        Ok(())
    }

    fn validate_user(&self, token: Token, desired_role: UserType) -> Result<(), &str> {
        let login = self.sessions.get(&token).ok_or("Invalid token")?;
        let user = self
            .users
            .get(login)
            .expect("Session initiated, but user does not exist");
        if user.user_type == desired_role {
            Ok(())
        } else {
            Err("Operation not permitted")
        }
    }

    // pub fn handle_post(&mut self, req: &Request) -> Result<Response, ServerError> {
    //     assert_eq!(req.method(), "POST");
    //
    //     match req.url().as_str() {
    //         "/auth/login" => {
    //             let login_data: LoginData = deserialize_request(&req)
    //                 .map_err(|_: &str| ServerError::BadRequest("Bad request".to_string()))?;
    //             let session_info = self
    //                 .init_session(login_data)
    //                 .map_err(|err: &str| ServerError::Forbidden(err.to_string()))?;
    //             Ok(Response::json(&AuthentificationData {
    //                 token: session_info.0,
    //                 user_type: session_info.1,
    //             }))
    //         }
    //
    //         "/auth/register" => {
    //             let register_data: RegisterUserReq =
    //                 deserialize_request(&req).map_err(|_: &str| {
    //                     ServerError::BadRequest("Invalid registration data".to_string())
    //                 })?;
    //             self.request_add_user(register_data)
    //                 .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;
    //             Ok(Response::text("Ok").with_status_code(200))
    //         }
    //
    //         "/auth/accept" => {
    //             self.validate_authentification(req, UserType::Manager)
    //                 .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;
    //             let accept_registration: AcceptRegistrationReq = deserialize_request(&req)
    //                 .map_err(|_| ServerError::BadRequest("Bad request".to_string()))?;
    //             self.accept_registration_request(&accept_registration.login)
    //                 .map_err(|err: _| ServerError::BadRequest(err.to_string()))?;
    //             Ok(Response::text("Ok").with_status_code(200))
    //         }
    //
    //         _ => Err(ServerError::NotFound("".to_string())),
    //     }
    // }
    //
    // pub fn handle_get(&mut self, req: &Request) -> Result<Response, ServerError> {
    //     match req.url().as_str() {
    //         "/auth" => {
    //             let session_token = req
    //                 .get_param("token")
    //                 .and_then(|v| v.parse::<u64>().ok())
    //                 .ok_or(ServerError::BadRequest("Bad token".to_string()))?;
    //             let user_data = self
    //                 .get_user_by_token(session_token)
    //                 .ok_or(ServerError::Forbidden("Invalid token".to_string()))?;
    //             Ok(Response::json(&user_data.public_user))
    //         }
    //         "/auth/accept" => {
    //             self.validate_authentification(req, UserType::Manager)
    //                 .map_err(|err: _| ServerError::Forbidden(err.to_string()))?;
    //             let registration_requests = &self.get_registration_requests();
    //             Ok(Response::json(registration_requests))
    //         }
    //
    //         _ => Err(ServerError::NotFound("".to_string())),
    //     }
    // }
}
