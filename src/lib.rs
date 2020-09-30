use std::collections::HashMap;

pub struct Scalar {
    value: f64,
    value_unit: CompositeUnit,
    display_unit: CompositeUnit,
    precision: u32,
}

impl Scalar {
    pub fn new(value: f64, precision: u32, unit: CompositeUnit) -> Self {
        Self {
            value,
            value_unit: unit.clone(),
            display_unit: unit,
            precision,
        }
    }

    pub fn max_precision(value: f64, unit: CompositeUnit) -> Self {
        Self {
            value,
            value_unit: unit.clone(),
            display_unit: unit,
            precision: std::u32::MAX,
        }
    }

    pub fn unitless(value: f64, precision: u32) -> Self {
        Self {
            value,
            value_unit: CompositeUnit::unitless(),
            display_unit: CompositeUnit::unitless(),
            precision,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub text: String,
    pub superscript: Option<String>,
    pub subscript: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnitClassId(usize);
pub struct UnitClass {
    pub name: String,
    pub base: UnitId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnitId(usize);
pub struct Unit {
    pub class: UnitClassId,
    pub name: String,
    pub symbol: Symbol,
    /// How many of the base unit is represented by 1 of this unit.
    pub base_ratio: Scalar,
}

impl Into<CompositeUnit> for UnitId {
    fn into(self) -> CompositeUnit {
        CompositeUnit {
            components: vec![(self, 1)],
        }
    }
}

pub struct UnitTemplate {
    pub name: String,
    pub symbol: Symbol,
    /// How many of the base unit is represented by 1 of this unit.
    pub base_ratio: f64,
}

#[derive(Clone)]
pub struct CompositeUnit {
    pub components: HashMap<UnitId, i32>,
}

impl CompositeUnit {
    pub fn unitless() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
}

use std::ops::{Div, Mul};

impl Mul for CompositeUnit {
    type Output = Self;

    fn mul(mut self, mut rhs: Self) -> Self {
        for (unit, rhs_value) in rhs.components.into_iter() {
            if let Some(lhs_value) = self.components.get_mut(&unit) {
                *lhs_value += rhs_value;
                if *lhs_value == 0 {
                    self.components.remove(&unit);
                }
            } else {
                self.components.insert(unit, rhs_value);
            }
        }
        self
    }
}

impl Div for CompositeUnit {
    type Output = Self;

    fn div(mut self, mut rhs: Self) -> Self {
        for (unit, rhs_value) in rhs.components.into_iter() {
            if let Some(lhs_value) = self.components.get_mut(&unit) {
                *lhs_value -= rhs_value;
                if *lhs_value == 0 {
                    self.components.remove(&unit);
                }
            } else {
                self.components.insert(unit, -rhs_value);
            }
        }
        self
    }
}

pub struct Environment {
    unit_classes: Vec<UnitClass>,
    units: Vec<Unit>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            unit_classes: Vec::new(),
            units: Vec::new(),
        }
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
            let base_ratio = Scalar::max_precision(template.base_ratio, unimplemented!());
        }
        (unit_class, unit_ids)
    }
}
