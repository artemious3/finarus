use std::sync::{OnceLock};
use chrono::{DateTime, Utc};

static REAL_TIME : OnceLock<DateTime<Utc>>  = OnceLock::new();
static VIRTUAL_TIME : OnceLock<DateTime<Utc>>  = OnceLock::new();

pub fn set_time(dt : &DateTime<Utc>){
    REAL_TIME.set(chrono::Utc::now()).expect("Time error");
    VIRTUAL_TIME.set(dt.clone()).expect("Time error");
}

pub fn get_time() -> DateTime<Utc>{
    match VIRTUAL_TIME.get() {
        Some(virtual_time) => {
            virtual_time.clone() + (Utc::now() - REAL_TIME.get().unwrap())
        },
        None => {
            Utc::now()
        }
    }
}

