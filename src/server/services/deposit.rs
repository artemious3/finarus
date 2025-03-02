
use l1::common::auth::Login;
use l1::common::Money;
use l1::common::deposit::Deposit;
use crate::traits::dynamic::Dynamic;
use std::collections::hash_map::HashMap;
use log::*;


use chrono::Datelike;


// individual per bank
#[derive(Default)]
pub struct DepositService {
     deposits : HashMap<Login, Vec<Deposit>>,
}


impl DepositService {
    pub fn add_client (&mut self, client : Login){
        let prev_val = self.deposits.insert(client, Vec::new());
        if prev_val.is_some(){
            panic!("Client with this login already existed");
        }
    }

    pub fn add_deposit(&mut self, client : Login, deposit : Deposit){
        let deposits = self.deposits.get_mut(&client).expect("Client does not exist");
        deposits.push(deposit);
    }

    pub fn get(&self, client : Login) -> &Vec<Deposit>{
        self.deposits.get(&client).expect("Client does not exist")
    }


    pub fn withdraw(&mut self, client : Login, idx : usize, now : chrono::DateTime<chrono::Utc>) -> Result<Money, &str>{
        let deposits = self.deposits.get_mut(&client).ok_or("Client does not exist")?;
        let deposit =  deposits.get(idx).ok_or("Deposit at specified index does not exist")?;
        if now < deposit.end_date {
            Err("Deposit can't be withdrawn before the end date")
        } else {
            let deposit = deposits.remove(idx);
            let money = deposit.current_amount;
            Ok(money)
        }
    }

}


impl Dynamic for DepositService {
    fn update(&mut self, now : &chrono::DateTime<chrono::Utc>) {
        for (_, vec) in &mut self.deposits {
            for deposit in vec {
                
                let months = now.month0() as i32 - deposit.last_update.month0() as i32 - 
                            (if now.day0() < deposit.last_update.day0() {1} else {0});
                if months < 0 {
                    error!("Now is the past, last update is the future!");
                    error!("Skipping deposit.");
                    continue;
                } else if months > 0 {
                    deposit.last_update = now.clone();
                    let koef = (1.0 + (deposit.interest_rate as f64)/100.0/12.0).powi(months);
                    deposit.current_amount = ((deposit.current_amount as f64) * koef).floor() as Money;
                }

            }
        }
        
    }
}






