use std::collections::HashMap;
use crate::types::class::Class;

pub struct ClassLoader {
    classes: HashMap<String, Box<Class>>
}

impl ClassLoader {
    pub fn new() -> ClassLoader {
        ClassLoader { classes: HashMap::new() }
    }

    pub fn load_class(&mut self, name: &str) -> &Box<Class> {
        self.classes.entry(String::from(name)).or_insert_with(|| Box::new(Class::parse(name)))
    }
}
