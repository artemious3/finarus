use l1::common::auth::*;
use l1::common::bank::{AccountID, BIK};
use l1::common::transaction::*;
use l1::common::user::*;
use l1::common::deposit::*;
use l1::common::Money;
use std::convert::TryInto;
use std::io::Write;
use std::string::String;
use chrono::{DateTime, Utc, NaiveDateTime};

const SPACE_PER_INDENT:  i32= 3;

fn add_indentation(s: &str, n: usize) -> String {
    let indent = str::repeat(" ", n);
    let ss = indent.to_string() + &String::from(s);
    ss
}

pub trait Inputtable {
    type InputType;
    fn input(invitation: &str, n: i32) -> Option<Self::InputType>;
    fn print_invitation(invitation: &str, n: i32) {
        print!("{}", add_indentation(invitation, n as usize));
        std::io::stdout().flush().expect("Error");
    }
}

fn input_until_valid<T>(invitation: &str, level: i32) -> Option<T::InputType>
where
    T: Inputtable,
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
    fn input(invitation: &str, level: i32) -> Option<String> {
        Self::print_invitation(invitation, level);
        let mut inp = String::new();
        std::io::stdin().read_line(&mut inp).expect("Input error");
        inp = inp.trim_end().to_string();
        Some(inp)
    }
}

impl Inputtable for u64 {
    type InputType = u64;
    fn input(invitation: &str, n: i32) -> Option<Self::InputType> {
        input_until_valid::<String>(invitation, n - SPACE_PER_INDENT)?
            .parse()
            .ok()
    }
}

impl Inputtable for u32 {
    type InputType = u32;
    fn input(invitation: &str, n: i32) -> Option<Self::InputType> {
        input_until_valid::<String>(invitation, n - SPACE_PER_INDENT)?
            .parse()
            .ok()
    }
}

impl Inputtable for i32 {
    type InputType = i32;
    fn input(invitation: &str, n: i32) -> Option<Self::InputType> {
        input_until_valid::<String>(invitation, n - SPACE_PER_INDENT)?
            .parse()
            .ok()
    }
}
impl Inputtable for u8 {
    type InputType = u8;
    fn input(invitation: &str, n: i32) -> Option<Self::InputType> {
        input_until_valid::<String>(invitation, n - SPACE_PER_INDENT)?
            .parse()
            .ok()
    }
}

impl Inputtable for [u8; 2] {
    type InputType = [u8; 2];
    fn input(invitation: &str, level: i32) -> Option<Self::InputType> {
        input_until_valid::<String>(invitation, level - SPACE_PER_INDENT)?
            .into_bytes()
            .try_into()
            .ok()
    }
}

impl Inputtable for [u8; 7] {
    type InputType = [u8; 7];
    fn input(invitation: &str, level: i32) -> Option<Self::InputType> {
        input_until_valid::<String>(invitation, level - SPACE_PER_INDENT)?
            .into_bytes()
            .try_into()
            .ok()
    }
}

impl Inputtable for DateTime<Utc> {
    type InputType = DateTime<Utc>;
    fn input(invitation: &str, n: i32) -> Option<Self::InputType> {
        let s = input_until_valid::<String>(invitation, n - SPACE_PER_INDENT)?;
        let dt = NaiveDateTime::parse_from_str(s.as_str(), "%Y-%m-%d %H:%M:%S").ok()?
                .and_local_timezone(Utc).single()?;
        Some(dt)
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

    fn input(invitation: &str, level: i32) -> Option<Self::InputType> {
        Self::print_invitation(invitation, level);
        Some(UserPassportData {
            series: input_until_valid::<String>("Series of passport (2 chars) : ", level)?,
            number: input_until_valid::<String>("Number of passport (7 chars) : ", level)?,
            id_number: input_until_valid::<String>("ID number : ", level)?,
        })
    }
}

impl Inputtable for UserPersonalName {
    type InputType = UserPersonalName;
    fn input(invitation: &str, level: i32) -> Option<Self::InputType> {
        Self::print_invitation(invitation, level);
        Some(UserPersonalName {
            first_name: input_until_valid::<String>("First name : ", level)?,
            middle_name: input_until_valid::<String>("Middle name : ", level)?,
            last_name: input_until_valid::<String>("Last name : ", level)?,
        })
    }
}

impl Inputtable for Client {
    type InputType = Client;
    fn input(invitation: &str, level: i32) -> Option<Self::InputType> {
        Self::print_invitation(invitation, level);
        Some({
            Client {
                full_name: input_until_valid::<UserPersonalName>("Full name : \n", level)?,
                passport: input_until_valid::<UserPassportData>("Passport data : \n", level)?,
                email: input_until_valid::<String>("Email : ", level)?,
                phone_number: input_until_valid::<String>("Phone number : ", level)?,
            }
        })
    }
}

impl Inputtable for LoginReq {
    type InputType = LoginReq;
    fn input(invitation: &str, level: i32) -> Option<Self::InputType> {
        Self::print_invitation(invitation, level);
        Some(LoginReq {
            login: input_until_valid::<String>("login: ", level)?,
            password: input_until_valid::<String>("password: ", level)?,
        })
    }
}

impl Inputtable for RegisterUserReq {
    type InputType = RegisterUserReq;
    fn input(invitation: &str, level: i32) -> Option<Self::InputType> {
        Self::print_invitation(invitation, level);
        Some(RegisterUserReq {
            login_data: input_until_valid::<LoginReq>("Credentials : \n", level)?,
            user_data: input_until_valid::<Client>("Personal information : \n", level)?,
        })
    }
}

impl Inputtable for AcceptRegistrationReq {
    type InputType = AcceptRegistrationReq;
    fn input(invitation: &str, level: i32) -> Option<Self::InputType> {
        Self::print_invitation(invitation, level);
        Some(AcceptRegistrationReq {
            login: input_until_valid::<String>("Login to accept : ", level)?,
        })
    }
}

impl Inputtable for TransactionEndPoint {
    type InputType = TransactionEndPoint;
    fn input(invitation: &str, level: i32) -> Option<Self::InputType> {
        Self::print_invitation(invitation, level);
        Some(TransactionEndPoint {
            bik: input_until_valid::<BIK>("Bank BIK : ", level)?,
            account_id: input_until_valid::<AccountID>("Account ID : ", level)?,
        })
    }
}


impl Inputtable for DepositNewRequest {
    type InputType = DepositNewRequest;

    fn input(invitation: &str, n: i32) -> Option<Self::InputType> {
        Self::print_invitation(invitation, n);


//TMP!!!
        Some(DepositNewRequest {
            src_account : u64::input("Source account : ", n)?,
            interest_rate : u8::input("Interest rate : ", n)?,
            months_expires : u32::input("Month expires", n)?,
            amount : Money(i32::input("Amount of money : ", n)?)
        })
//TMP!!!

    }
}
