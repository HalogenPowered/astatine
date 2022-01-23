use std::collections::HashMap;
use crate::types::class::Class;

pub struct ClassLoader {
    classes: HashMap<String, Class>
}

impl ClassLoader {
    pub fn new() -> ClassLoader {
        ClassLoader { classes: HashMap::new() }
    }

    pub fn load_class(&mut self, name: &str) -> &Class {
        self.classes.entry(String::from(name)).or_insert_with(|| Class::parse(name))
    }
}
