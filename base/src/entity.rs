use crate::{
    data::{Data, Describe},
    prelude::{EntityClassId, Instance},
};
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Clone, Debug)]
pub struct EntityClass {
    pub names: Vec<String>,
}

impl Describe for EntityClass {
    fn describe(&self, into: &mut String, _instance: &Instance) {
        write!(into, "{}", self.names[0]).unwrap();
    }
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub properties: HashMap<String, Data>,
    pub classes: Vec<EntityClassId>,
}

impl Describe for Entity {
    fn describe(&self, into: &mut String, instance: &Instance) {
        write!(into, "{{ ").unwrap();
        if self.classes.len() > 0 {
            self.classes[0].describe(into, instance);
            for class in &self.classes[1..] {
                write!(into, ", ").unwrap();
                class.describe(into, instance);
            }
            if self.properties.len() > 0 {
                write!(into, ", ").unwrap();
            }
        }
        let mut iter = self.properties.iter();
        if let Some((name, value)) = iter.next() {
            write!(into, "{}: ", name);
            value.describe(into, instance);
            for (name, value) in iter {
                write!(into, ", ").unwrap();
                write!(into, "{}: ", name);
                value.describe(into, instance);
            }
        }
        write!(into, " }}").unwrap();
    }
}
