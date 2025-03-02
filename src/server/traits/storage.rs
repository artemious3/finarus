
pub trait Storage {
    fn store(&self, dir : std::path::Path);
    fn load(&mut self, dir : std::path::Path);
}
