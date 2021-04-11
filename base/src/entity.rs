use crate::prelude::{Data, EntityClassId};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct EntityClass {
    pub names: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub properties: HashMap<String, Data>,
    pub classes: Vec<EntityClassId>,
}
