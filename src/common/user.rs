
use std::string::String;
use serde::{Serialize, Deserialize};
use crate::common::validate::Validate;
use std::str;


#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum UserType {
    Client,
    Operator,
    Manager,
    EnterpriseSpecialist,
    Administrator
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserPersonalName{
    pub first_name : String,
    pub middle_name : String,
    pub last_name : String
}

impl Validate for UserPersonalName {
    fn validate(&self) -> Result<(), &str> {
        if self.first_name.is_empty(){
            Err("First name is empty")
        }
        else if self.middle_name.is_empty(){
            Err("Middle name is empty")
        }
        else if self.last_name.is_empty(){
            Err("Last name is empty")
        }
        else {
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserPassportData{
    pub series : String,
    pub number : String,
    pub id_number : String
}



impl Validate for UserPassportData
{
    fn validate(&self) -> Result<(), &str> {
        if self.series.len() != 2 || !self.series.chars().all(|c:_|c.is_alphabetic()){
            Err("Wrong passport series")
        } else if self.number.len() != 7 || !self.series.chars().all(|c:_|c.is_digit(10)){
            Err("Wrong passport number")
        } else {
            Ok(())
        }
        //TODO: check id number
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Client{
    pub full_name : UserPersonalName,
    pub passport : UserPassportData,
    pub phone_number : String,
    pub email : String
}

impl Validate for Client{
    fn validate(&self) -> Result<(), &str> {
        self.full_name.validate()?;
        self.passport.validate()?;
        let re_phone_number = regex::Regex::new(r#"^[\+]?[(]?[0-9]{3}[)]?[-\s\.]?[0-9]{3}[-\s\.]?[0-9]{4,6}$"#).unwrap();
        if re_phone_number.is_match(self.phone_number.as_str()){
            Err("Invalid phone number")
        } else {
            Ok(())
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Enterprise{
    pub enterprise_type : String, 
    pub name : String, 
    pub unp : String,
    pub address : String
}

pub type UserID = u64;


#[derive(Debug, Clone)]
pub enum UserData {
    None,
    EnterpriseData(Enterprise),
    ClientData(Client)
}






