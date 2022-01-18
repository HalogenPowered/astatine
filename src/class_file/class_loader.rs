use std::collections::HashMap;
//use crate::parse_class;
use crate::types::class::Class;

pub struct ClassLoader {
    classes: HashMap<String, Box<Class>>
}

impl ClassLoader {
    pub fn new() -> ClassLoader {
        ClassLoader { classes: HashMap::new() }
    }

    pub fn load_class(&mut self, name: &str) -> &Box<Class> {
        if self.classes.contains_key(name) {
            return self.classes.get(name).unwrap();
        }
        panic!("Could not find class with name {}! Fix this!", name);
        //let class = Box::new(parse_class(name));
        //self.classes.insert(String::from(name), class);
        //&class
    }
}
