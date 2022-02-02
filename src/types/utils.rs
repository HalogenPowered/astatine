macro_rules! named {
    () => {
        pub fn name(&self) -> &str {
            self.name.as_str()
        }
    }
}

macro_rules! describable {
    ($descriptor:ident) => {
        pub fn descriptor(&self) -> &crate::utils::descriptors::$descriptor {
            &self.descriptor
        }
    }
}

macro_rules! optional_string {
    ($name:ident) => {
        pub fn $name(&self) -> Option<&str> {
            self.$name.as_ref().map(|value| value.as_str())
        }
    }
}

macro_rules! generic {
    () => {
        optional_string!(generic_signature);
    }
}

macro_rules! versioned {
    () => {
        optional_string!(version);
    }
}
