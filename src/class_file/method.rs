pub struct BootstrapMethod {
    pub method_ref: u16,
    pub arguments: Vec<u16>
}

pub struct MethodParameter {
    pub name_index: u16,
    pub access_flags: u16
}
