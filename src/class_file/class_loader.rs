use std::rc::Rc;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::types::class::Class;

pub struct ClassLoader {
    classes: Mutex<HashMap<String, Rc<Class>>>
}

impl ClassLoader {
    pub fn new() -> Self {
        ClassLoader { classes: Mutex::new(HashMap::new()) }
    }

    pub fn get_class(&self, name: &str) -> Option<Rc<Class>> {
        self.classes.lock().unwrap().get(name).map(|value| Rc::clone(value))
    }

    pub fn load_class(&self, name: &str) -> Rc<Class> {
        Rc::clone(self.classes.lock().unwrap().entry(String::from(name)).or_insert_with(|| Rc::new(Class::parse(self, name))))
    }
}
