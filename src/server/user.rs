
use l1::common::user::{Client, UserType, UserData};

#[derive(Debug)]
pub struct InternalUser{
    pub user_type : UserType,
    pub login : String,
    pub password_hash : String,
    pub public_user : UserData
}




