
use l1::common::auth::Login;
use l1::common::Money;
use l1::common::deposit::Deposit;
use crate::traits::dynamic::Dynamic;
use std::collections::hash_map::HashMap;
use log::*;
use chrono::{DateTime, Utc};


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


type UnitSize = i32;

pub fn signed_month_difference(start: &DateTime<Utc>, end: &DateTime<Utc>) -> UnitSize {
    let end_naive = end.date_naive();
    let start_naive = start.date_naive();

    let month_diff = end_naive.month() as UnitSize - start_naive.month() as UnitSize;
    let years_diff = (end_naive.year() - start_naive.year()) as UnitSize;
    if month_diff >= 0 {
        (years_diff * 12) + month_diff
    } else {
        (years_diff - 1) * 12 + (month_diff + 12)
    }
}


impl Dynamic for DepositService {
    fn update(&mut self, now : &chrono::DateTime<chrono::Utc>) {
        for (_, vec) in &mut self.deposits {
            for deposit in vec {
                
                //relative to last update
                let months_to_now = signed_month_difference(&deposit.last_update, now);
                let months_to_end = signed_month_difference(&deposit.last_update, &deposit.end_date);

                if months_to_end < 0 {
                    info!("Deposit skipped since it is overdue");
                    continue;
                } else if months_to_now < 0 {
                    error!("Now is the past, last update is the future!");
                    error!("Something is wrong. Skipping deposit.");
                    continue;
                } else  {
                    let months = std::cmp::min(months_to_now, months_to_end);
                    deposit.last_update = now.clone();
                    let koef = (1.0 + (deposit.interest_rate as f64)/100.0/12.0).powi(months);
                    deposit.current_amount = Money( ((*deposit.current_amount as f64) * koef).floor() as i32 );
                }

            }
        }
        
    }
}






