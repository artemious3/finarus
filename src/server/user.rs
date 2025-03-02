
use l1::common::user::{User, UserType};

#[derive(Debug)]
pub struct InternalUser{
    pub user_type : UserType,
    pub login : String,
    pub password_hash : String,
    pub public_user : Option<User>
}
