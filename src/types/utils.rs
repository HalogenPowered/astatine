pub trait Nameable {
    fn name(&self) -> &str;
}

pub trait Generic {
    fn generic_signature(&self) -> Option<&str>;
}
