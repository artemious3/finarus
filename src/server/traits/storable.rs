
pub trait Storable {
    fn store(&self, dir : &std::path::Path);
    fn load(&mut self, dir : &std::path::Path);
}



