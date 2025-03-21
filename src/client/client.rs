use crate::inputtable::Inputtable;
use crate::menu::*;
use l1::common::auth::SessionResponse;
use l1::common::user:: UserType;
use l1::common::bank:: BIK;

use crate::auth_actions::*;
use crate::client_actions::*;
use crate::manager_actions::*;
use crate::operator_actions::*;
use crate::enterprise_action::*;



use std::sync::{Arc, Mutex};


type AuthInfo = SessionResponse;

pub struct ClientContext {
    pub auth_info: Option<AuthInfo>,
    pub login : Option<String>,
    pub bik: Option<BIK>,
}

pub struct Client<'a> {
    ctx: Arc<Mutex<ClientContext>>,
    auth_menu: Menu<'a>,
    manager_menu: Menu<'a>,
    client_menu: Menu<'a>,
    operator_menu: Menu<'a>,
    enterprise_menu: Menu<'a>,
}


impl<'a> Client<'a> {
    pub fn new() -> Self {
        let mut client = Client {
            ctx: Arc::new(Mutex::new(ClientContext {
                auth_info: None,
                login:None,
                bik: None,
            })),
            auth_menu: Menu::new(),
            client_menu: Menu::new(),
            manager_menu: Menu::new(),
            operator_menu: Menu::new(),
            enterprise_menu: Menu::new(),
        };

        client.build_auth_menu();
        client.build_client_menu();
        client.build_manager_menu();
        client.build_operator_menu();
        client.build_enterprise_menu();

        client
    }


    fn build_auth_menu(&mut self) {
        self.auth_menu.add_action('l' as u8, Box::new(LoginAction{}));
        self.auth_menu.add_action('r' as u8, Box::new(RegisterAction{}));
    }

    pub fn user_type(&self) -> Option<UserType> {
        let ctx = self.ctx.lock().expect("Mutex");
        let auth_info = ctx.auth_info.as_ref()?;
        Some(auth_info.user_type.clone())
    }

    pub fn build_client_menu(&mut self) {

        self.client_menu.add_action('t' as u8, Box::new(TransacionAction{}));

        let mut acc_menu = Menu::new();
        acc_menu.set_name("ACCOUNT");
        acc_menu.add_action('g' as u8, Box::new(AccountsGetAction{}));
        acc_menu.add_action('o' as u8, Box::new(AccountOpenAction{}));
        self.client_menu.add_action('a' as u8, Box::new(acc_menu));


        let mut deposit_menu = Menu::new();
        deposit_menu.set_name("DEPOSITS");
        deposit_menu.add_action('o' as u8, Box::new(DepositOpen{}));
        deposit_menu.add_action('g' as u8, Box::new(DepositGet{}));
        deposit_menu.add_action('w' as u8, Box::new(DepositWithdrawAction{}));
        self.client_menu.add_action('d' as u8, Box::new(deposit_menu));


        let mut credit_menu = Menu::new();
        credit_menu.set_name("CREDIT");
        credit_menu.add_action('g' as u8, Box::new(CreditGetAction{}));
        credit_menu.add_action('n' as u8, Box::new(CreditNewAction{}));
        self.client_menu.add_action('c' as u8, Box::new(credit_menu));


        let mut salary_menu = Menu::new();
        salary_menu.set_name("SALARY");
        salary_menu.add_action('r' as u8, Box::new(SalaryRequestAction{}));
        self.client_menu.add_action('s' as u8, Box::new(salary_menu));


        self.client_menu.add_action('b' as u8, Box::new(SelectBankAction{}));
        self.client_menu.add_action('i' as u8, Box::new(GetAuthInfoAction{}));

    }

    pub fn build_manager_menu(&mut self) {
        self.manager_menu.add_action('b' as u8 , Box::new(SelectBankAction{}));
        self.manager_menu.add_action('a' as u8 , Box::new(AcceptRegistrationRequestsAction{}));
        self.manager_menu.add_action('t' as u8 , Box::new(AdvanceTimeAction{}));
        self.manager_menu.add_action('e' as u8 , Box::new(GetTimeAction{}));
        self.manager_menu.add_action('c' as u8 , Box::new(CreditAcceptAction{}));
        self.manager_menu.add_action('T' as u8 , Box::new(TransactionUnprotecredAction{}));
    }



    pub fn build_operator_menu(&mut self){
        self.operator_menu.add_action('t' as u8, Box::new(TransactionsGetAction{}));
        self.operator_menu.add_action('r' as u8, Box::new(TransactionsRevertAction{}));
    }


    pub fn build_enterprise_menu(&mut self){
        self.enterprise_menu.add_action('i' as u8, Box::new(SalaryInitProjectAction{}));
        self.enterprise_menu.add_action('a' as u8, Box::new(SalaryAcceptAction{}));
        self.enterprise_menu.add_action('a' as u8, Box::new(AccountOpenAction{}));
        self.enterprise_menu.add_action('a' as u8, Box::new(AccountsGetAction{}));
    }

    pub fn run(&mut self) {
        loop {
            let _ = self.auth_menu.exec(self.ctx.clone());
            if self.user_type().is_some() {
                break;
            }
        }

        let user_menu = match self.user_type().unwrap() {
            UserType::Client => &mut self.client_menu,
            UserType::Manager => &mut self.manager_menu,
            UserType::Operator => &mut self.operator_menu,
            UserType::EnterpriseSpecialist => &mut self.enterprise_menu,
            _ => unimplemented!()
        };
        loop {
            let _ = user_menu.exec(self.ctx.clone());
        }
    }
}
