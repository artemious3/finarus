use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TimeAdvanceReq {
    pub time : chrono::DateTime<chrono::Utc>
}


