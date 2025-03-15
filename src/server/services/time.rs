use chrono::{DateTime, Utc};

pub struct TimeService {
    real_time: DateTime<Utc>,
    virtual_time: Option<DateTime<Utc>>,
}

impl TimeService {
    pub fn new() -> TimeService {
        TimeService {
            real_time: chrono::Utc::now(),
            virtual_time: None,
        }
    }

    pub fn set_time(&mut self, dt: &DateTime<Utc>) {
        self.real_time = chrono::Utc::now();
        self.virtual_time = Some(dt.clone());
    }

    pub fn get_time(&self) -> DateTime<Utc> {
        match self.virtual_time {
            Some(vt) => vt.clone() + (Utc::now() - self.real_time),
            None => Utc::now(),
        }
    }
}
