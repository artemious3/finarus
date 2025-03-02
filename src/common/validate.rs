


pub trait Validate {
    fn validate(&self) -> Result<(), &str>;
}
