
use std::collections::HashMap;
use l1::common::auth::Login;
use l1::common::credit::*;
use crate::traits::dynamic::Dynamic;


#[derive(Default)]
pub struct CreditService {
    pub accepted_credits : HashMap<Login, Vec<Credit>>,
    pub unaccepted_credits : Vec<CreditUnaccepted>
}


impl CreditService {
    pub fn add_client(&mut self, client : Login){
        self.accepted_credits.insert(client, Vec::new());
    }
}

impl  Dynamic for CreditService {
    fn update(&mut self, time :  &chrono::DateTime<chrono::Utc>) {
        unimplemented!()
    }
}
