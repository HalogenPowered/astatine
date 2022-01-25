use std::collections::HashMap;
use crate::types::class::Class;

pub struct ClassLoader<'a> {
    classes: HashMap<String, Class<'a>>
}

impl<'a> ClassLoader<'a> {
    pub fn new() -> ClassLoader<'a> {
        ClassLoader { classes: HashMap::new() }
    }

    pub fn get_class(&self, name: &str) -> Option<&Class<'a>> {
        self.classes.get(name)
    }

    pub fn load_class(&'a mut self, name: &str) -> &Class<'a> {
        self.classes.entry(String::from(name)).or_insert_with(|| Class::parse(self, name))
    }
}
