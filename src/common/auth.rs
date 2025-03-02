use serde::{Deserialize, Serialize};
use crate::common::user::{User, UserType};

pub type Token = u64;
pub type Login = String;


#[derive(Serialize, Deserialize)]
pub struct LoginReq {
    pub login: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct SessionResponse{
    pub token : u64,
    pub user_type : UserType,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfoResonse {
    pub user_type : UserType,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterUserReq{
    pub login_data : LoginReq,
    pub user_data : User,
}

#[derive(Serialize, Deserialize)]
pub struct AcceptRegistrationReq {
    pub login : String
}

#[derive(Serialize, Deserialize)]
pub struct GetRegistrationsReq {
    pub login : String,
    pub user : User
}


