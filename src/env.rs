use crate::unit::{Unit, UnitClass};
use crate::util::{ItemStorage, StorageHolder};

#[derive(Debug)]
pub struct Environment {
    unit_classes: ItemStorage<UnitClass>,
    units: ItemStorage<Unit>,
}

impl Environment {
    pub fn new() -> Self {
        let mut result = Self {
            unit_classes: ItemStorage::new(),
            units: ItemStorage::new(),
        };
        crate::unit::add_default_units(&mut result);
        result
    }
}

impl StorageHolder<UnitClass> for Environment {
    fn borrow_storage(&self) -> &ItemStorage<UnitClass> {
        &self.unit_classes
    }

    fn borrow_storage_mut(&mut self) -> &mut ItemStorage<UnitClass> {
        &mut self.unit_classes
    }
}

impl StorageHolder<Unit> for Environment {
    fn borrow_storage(&self) -> &ItemStorage<Unit> {
        &self.units
    }

    fn borrow_storage_mut(&mut self) -> &mut ItemStorage<Unit> {
        &mut self.units
    }
}
