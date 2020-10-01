use crate::formula::Scalar;
use crate::prelude::*;
use crate::unit::{CompositeUnit, CompositeUnitClass, Unit, UnitClass};
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

    pub fn format_base_unit(&self, base_unit: &CompositeUnitClass) -> String {
        let mut numerator = "".to_owned();
        let mut denominator = "".to_owned();
        for (unit_class_id, power) in base_unit.components.iter() {
            assert!(*power != 0);
            if *power > 0 {
                numerator += &format!("{}^{}", self.borrow(*unit_class_id).0, power);
            } else {
                denominator += &format!("{}^{}", self.borrow(*unit_class_id).0, -power);
            }
        }
        format!("{} / {}", numerator, denominator)
    }

    pub fn format_unit(&self, unit: &CompositeUnit) -> String {
        let mut numerator = "".to_owned();
        let mut denominator = "".to_owned();
        for (unit_id, power) in unit.components.iter() {
            assert!(*power != 0);
            if *power > 0 {
                numerator += &format!("{}^{}", self.borrow(*unit_id).name, power);
            } else {
                denominator += &format!("{}^{}", self.borrow(*unit_id).name, -power);
            }
        }
        format!("{} / {}", numerator, denominator)
    }

    pub fn format_scalar_detailed(&self, scalar: &Scalar) -> String {
        let ratio = self.base_conversion_ratio_of(&scalar.display_unit);
        assert!(scalar.precision > 0);
        format!(
            "{1:.0$e} {2} ({3:.0$e} {4})",
            scalar.precision as usize - 1,
            scalar.base_value / ratio,
            self.format_unit(&scalar.display_unit),
            scalar.base_value,
            self.format_base_unit(&scalar.base_unit)
        )
    }

    /// Returns the base unit of the given unit. For example, Meters^2*Seconds^-1 will return
    /// Length^2*Time^-1. Hz*Area^-1 will return Time^-1*Length^-2.
    pub fn base_unit_of(&self, unit: &CompositeUnit) -> CompositeUnitClass {
        let mut complete_base = CompositeUnitClass::unitless();
        for (unit_id, power) in unit.components.iter() {
            let component_base = &self.borrow(*unit_id).base_class;
            assert!(*power != 0);
            if *power > 0 {
                for _ in 0..*power {
                    complete_base = complete_base * component_base.clone();
                }
            } else {
                for _ in 0..-*power {
                    complete_base = complete_base / component_base.clone();
                }
            }
        }
        complete_base
    }

    /// Calculate the ratio to convert a value of this unit to the base unit. Multiplying the value
    /// times this ratio will give the value expressed in terms of the base unit. Dividing a value
    /// expressed in the base unit by the ratio will give the value expressed in terms of the
    /// inputted unit.
    pub fn base_conversion_ratio_of(&self, unit: &CompositeUnit) -> f64 {
        let mut ratio = 1.0;
        for (unit_id, power) in unit.components.iter() {
            // E.G. if Feet^2 is a component, we will want the ratio to multiply values by
            // (Feet to Base Unit) * (Feet to Base Unit).
            ratio *= self.borrow(*unit_id).base_ratio.powi(*power);
        }
        ratio
    }

    pub fn make_scalar(&self, value: f64, unit: CompositeUnit, precision: u32) -> Scalar {
        let base_unit = self.base_unit_of(&unit);
        let base_value = value * self.base_conversion_ratio_of(&unit);
        Scalar::new(base_value, base_unit, unit, precision)
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
