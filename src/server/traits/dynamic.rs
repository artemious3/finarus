
pub trait Dynamic {
    fn update(&mut self, time :  &chrono::DateTime<chrono::Utc>);
}
