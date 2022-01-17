use std::collections::HashMap;
use crate::parse_class;
use crate::types::class::Class;

pub struct ClassLoader {
    classes: HashMap<String, Class>
}

impl ClassLoader {
    pub fn new() -> ClassLoader {
        ClassLoader { classes: HashMap::new() }
    }

    pub fn load_class(&mut self, name: &str) -> &mut Class {
        if self.classes.contains_key(name) {
            return &mut self.classes.get(name).unwrap();
        }
        let mut loaded = parse_class(name);
        self.classes.insert(String::from(name), loaded);
        &mut loaded
    }
}
