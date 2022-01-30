use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use internship::IStr;
use crate::types::class::Class;

#[derive(Debug)]
pub struct ClassLoader {
    classes: Mutex<HashMap<IStr, Arc<Class>>>
}

impl ClassLoader {
    pub fn new() -> Self {
        ClassLoader { classes: Mutex::new(HashMap::new()) }
    }

    pub fn get_class(&self, name: &str) -> Option<Arc<Class>> {
        self.classes.lock()
            .unwrap()
            .get(name)
            .map(|value| Arc::clone(value))
    }

    pub fn load_class(self: Arc<ClassLoader>, name: &str) -> Arc<Class> {
        Arc::clone(self.classes.lock()
            .unwrap()
            .entry(IStr::new(name))
            .or_insert_with(|| Arc::new(Class::parse(Arc::clone(&self), name))))
    }
}
