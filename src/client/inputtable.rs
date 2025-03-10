use std::io::Write;
use std::string::String;
use l1::common::user::*;
use l1::common::auth::*;
use l1::common::transaction::*;
use l1::common::bank::{BIK, AccountID};
use std::convert::TryInto;


const SPACE_PER_INDENT : usize = 3;

fn add_indentation(s : &str, n : usize) -> String{
    let indent = str::repeat(" ", n);
    let ss = indent.to_string() + &String::from(s);
    ss
}

pub trait Inputtable {
    type InputType;
    fn input(invitation: &str, n : usize) -> Option<Self::InputType>;
    fn print_invitation(invitation: &str, n : usize){
        print!("{}",add_indentation(invitation, n));
        std::io::stdout().flush().expect("Error");
    }
}



fn input_until_valid<T>(invitation : &str, level : usize) -> Option<T::InputType>
where T : Inputtable
{

    loop {
        let maybe_input = T::input(invitation, level + SPACE_PER_INDENT);
        if maybe_input.is_some() {
            return maybe_input;
        } 
        
        println!("WRONG INPUT");
        println!("Do you want to cancel input?[y/n]");
        let mut s = String::new();
        std::io::stdin().read_line(&mut s).expect("Error");
        if s.chars().nth(0).unwrap_or('\0') == 'y' {
            return None;
        }
    }
}


/// ----------- INPUTTABLE for basic types ----------- ///

impl Inputtable for String {
    type InputType = String;
    fn input(invitation: &str, level : usize) -> Option<String> {
        Self::print_invitation(invitation, level);
        let mut inp = String::new();
        std::io::stdin().read_line(&mut inp).expect("Input error");
        inp = inp.trim_end().to_string();
        Some(inp)
    }
}


impl Inputtable for u64 {
    type InputType = u64;
    fn input(invitation: &str, n : usize) -> Option<Self::InputType> {
        input_until_valid::<String>(invitation, n-SPACE_PER_INDENT)?.parse().ok()
    }
}

impl Inputtable for [u8;2]{
    type InputType = [u8;2];
    fn input(invitation: &str, level : usize) -> Option<Self::InputType> {
        input_until_valid::<String>(invitation, level-SPACE_PER_INDENT)?.into_bytes().try_into().ok()
    }    
}

impl Inputtable for [u8;7]{
    type InputType = [u8;7];
    fn input(invitation: &str, level : usize) -> Option<Self::InputType> {
        input_until_valid::<String>(invitation, level-SPACE_PER_INDENT)?.into_bytes().try_into().ok()
    }
    
}

// macro_rules! impl_Inputtable {
//     (for $($t:ty),+) => {
//     $(
//
//     impl Inputtable for $t{
//         type InputType = $t;
//         fn input(invitation : &str) ->  Option<Self::InputType>{
//             print!("{}",invitation);
//             let mut inp = String::new();
//             std::io::stdin().read_line(&mut inp).expect("Input error");
//             let inp_parsed = inp.parse::<$t>().ok()?;
//             Some(inp_parsed)
//         }
//     }
//     )*
//     }
// }
//
// impl_Inputtable!(for u64);



/// ----------- INPUTTABLE for realm structs ----------- ///

impl Inputtable for UserPassportData {
    type InputType = UserPassportData;

    fn input(invitation: &str, level : usize) -> Option<Self::InputType> {
        Self::print_invitation(invitation, level);
        Some(UserPassportData {
            series : input_until_valid::<String>("Series of passport (2 chars) : ", level)?,
            number : input_until_valid::<String>("Number of passport (7 chars) : ", level)?,
            id_number : input_until_valid::<String>("ID number : ", level)?,
        })
    }
}

impl Inputtable for UserPersonalName {
    type InputType = UserPersonalName;
    fn input(invitation: &str, level : usize) -> Option<Self::InputType> {
        Self::print_invitation(invitation, level);
        Some(UserPersonalName {
            first_name : input_until_valid::<String>("First name : ", level)?,
            middle_name : input_until_valid::<String>("Middle name : ", level)?,
            last_name : input_until_valid::<String>("Last name : ", level)?,
        })
    }
}


impl Inputtable for User {
    type InputType = User;
    fn input(invitation: &str, level : usize) -> Option<Self::InputType> {
        Self::print_invitation(invitation, level);
        Some({
            User{
                full_name : input_until_valid::<UserPersonalName>("Full name : \n", level)?,
                passport : input_until_valid::<UserPassportData>("Passport data : \n", level)?,
                email : input_until_valid::<String>("Email : ", level)?,
                phone_number :input_until_valid::<String>("Phone number : ", level)?,
            }
        })
    }
}


impl Inputtable for LoginReq {
    type InputType = LoginReq;
    fn input(invitation: &str, level : usize) -> Option<Self::InputType> {
        Self::print_invitation(invitation, level);
        Some(LoginReq {
            login : input_until_valid::<String>("login: ", level)?,
            password : input_until_valid::<String>("password: ", level)?,
        })
    }
}

impl Inputtable for RegisterUserReq {
    type InputType = RegisterUserReq;
    fn input(invitation: &str, level : usize) -> Option<Self::InputType> {
        Self::print_invitation(invitation, level);
        Some(RegisterUserReq {
            login_data : input_until_valid::<LoginReq>("Credentials : \n", level)?,
            user_data: input_until_valid::<User>("Personal information : \n", level)?,
        })

    }
}


impl Inputtable for AcceptRegistrationReq {
    type InputType = AcceptRegistrationReq;
    fn input(invitation: &str, level : usize) -> Option<Self::InputType> {
        Self::print_invitation(invitation, level);
        Some(AcceptRegistrationReq {
            login : input_until_valid::<String>("Login to accept : ", level)?,
        })
    }
}


impl Inputtable for TransactionEndPoint {
    type InputType = TransactionEndPoint;
    fn input(invitation: &str, level : usize) -> Option<Self::InputType> {
        Self::print_invitation(invitation, level);
       Some( TransactionEndPoint {
            bik : input_until_valid::<BIK>("Bank BIK : ", level)?, 
            account_id : input_until_valid::<AccountID>("Account ID", level)?,
        })
    }
}


