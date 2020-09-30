use crate::formula::{Scalar, Symbol};
use crate::unit::{CompositeUnit, Unit, UnitClass};
use std::ops::{Mul, Div};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnitClassId(usize);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnitId(usize);

impl Mul<CompositeUnit> for UnitId {
    type Output = CompositeUnit;

    fn mul(self, rhs: CompositeUnit) -> CompositeUnit {
        <UnitId as Into<CompositeUnit>>::into(self) * rhs
    }
}

impl Mul for UnitId {
    type Output = CompositeUnit;

    fn mul(self, rhs: Self) -> CompositeUnit {
        <UnitId as Into<CompositeUnit>>::into(self) * <UnitId as Into<CompositeUnit>>::into(rhs)
    }
}

impl Div<CompositeUnit> for UnitId {
    type Output = CompositeUnit;

    fn div(self, rhs: CompositeUnit) -> CompositeUnit {
        <UnitId as Into<CompositeUnit>>::into(self) / rhs
    }
}

impl Div for UnitId {
    type Output = CompositeUnit;

    fn div(self, rhs: Self) -> CompositeUnit {
        <UnitId as Into<CompositeUnit>>::into(self) / <UnitId as Into<CompositeUnit>>::into(rhs)
    }
}

pub struct UnitTemplate {
    pub name: String,
    pub symbol: Symbol,
    /// How many of the base unit is represented by 1 of this unit.
    pub base_ratio: f64,
}

#[derive(Debug)]
pub struct Environment {
    unit_classes: Vec<UnitClass>,
    units: Vec<Unit>,
}

impl Environment {
    pub fn new() -> Self {
        let mut result = Self {
            unit_classes: Vec::new(),
            units: Vec::new(),
        };
        crate::unit::add_default_units(&mut result);
        result
    }

    pub fn build_unit_class(
        &mut self,
        name: String,
        mut unit_templates: impl Iterator<Item = UnitTemplate>,
    ) -> (UnitClassId, Vec<UnitId>) {
        // Create IDs for the base unit and the unit class. At the time of creation they are invalid
        // as the list does not contain any item at the index they contain.
        let base_unit = UnitId(self.units.len());
        let unit_class = UnitClassId(self.unit_classes.len());
        let base_unit_template = unit_templates
            .next()
            .expect("Cannot build unit class with zero units!");
        assert_eq!(
            base_unit_template.base_ratio, 1.0,
            "The base ratio of the base unit must be 1.0"
        );
        // Add the base unit and unit class so now the IDs are valid.
        self.units.push(Unit {
            class: unit_class,
            name: base_unit_template.name,
            symbol: base_unit_template.symbol,
            base_ratio: Scalar::max_precision(1.0, CompositeUnit::unitless()),
        });
        self.unit_classes.push(UnitClass {
            name,
            base: base_unit,
        });

        let mut unit_ids = vec![base_unit];
        for template in unit_templates {
            let unit_id = UnitId(self.units.len());
            let base_ratio = Scalar::max_precision(template.base_ratio, base_unit / unit_id);
            self.units.push(Unit {
                class: unit_class,
                name: template.name,
                symbol: template.symbol,
                base_ratio,
            });
            unit_ids.push(unit_id);
        }
        (unit_class, unit_ids)
    }
}
