use crate::env::{UnitClassId, UnitId};
use crate::formula::{Scalar, Symbol};
use crate::util;
use std::collections::HashMap;
use std::ops::{Div, Mul};

#[derive(Clone, Debug)]
pub struct UnitClass {
    pub name: String,
    pub base: UnitId,
}

#[derive(Clone, Debug)]
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
            components: vec![(self, 1)].into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug)]
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

impl Mul for CompositeUnit {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self {
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

impl Mul<UnitId> for CompositeUnit {
    type Output = Self;

    fn mul(self, rhs: UnitId) -> Self {
        self * <UnitId as Into<CompositeUnit>>::into(rhs)
    }
}

impl Div for CompositeUnit {
    type Output = Self;

    fn div(mut self, rhs: Self) -> Self {
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

impl Div<UnitId> for CompositeUnit {
    type Output = Self;

    fn div(self, rhs: UnitId) -> Self {
        self / <UnitId as Into<CompositeUnit>>::into(rhs)
    }
}

const METRIC_PREFIXES: [(&'static str, &'static str, f64); 21] = [
    ("Yotta", "Y", 1e24),
    ("Zetta", "Z", 1e21),
    ("Exa", "E", 1e18),
    ("Peta", "P", 1e15),
    ("Tera", "T", 1e12),
    ("Giga", "G", 1e9),
    ("Mega", "M", 1e6),
    ("Kilo", "k", 1e3),
    ("Hecto", "h", 1e2),
    ("Deka", "da", 1e1),
    ("", "", 1e0),
    ("Deci", "d", 1e-1),
    ("Centi", "c", 1e-2),
    ("Milli", "m", 1e-3),
    ("Micro", "μ", 1e-6),
    ("Nano", "n", 1e-9),
    ("Pico", "p", 1e-12),
    ("Femto", "f", 1e-15),
    ("Atto", "a", 1e-18),
    ("Zepto", "z", 1e-21),
    ("Yocto", "y", 1e-24),
];

fn make_metric_unit_templates(
    name: &str,
    symbol: Symbol,
    base_prefix: usize,
) -> Vec<crate::env::UnitTemplate> {
    let base_multiplier = METRIC_PREFIXES[base_prefix].2;
    let mut result: Vec<_> = METRIC_PREFIXES
        .iter()
        .map(|(full, prefix, multiplier)| {
            let mut symbol = symbol.clone();
            symbol.text = format!("{}{}", prefix, symbol.text);
            crate::env::UnitTemplate {
                name: util::to_title_case(&format!("{}{}", full, name)),
                symbol,
                base_ratio: base_multiplier / multiplier,
            }
        })
        .collect();
    result.swap(0, base_prefix);
    result
}

pub fn add_default_units(env: &mut crate::env::Environment) {
    const KILO_BASE: usize = 7;
    assert_eq!(METRIC_PREFIXES[KILO_BASE].2, 1e3);
    const PLAIN_BASE: usize = 10;
    assert_eq!(METRIC_PREFIXES[PLAIN_BASE].2, 1e0);

    {
        // Mass
        let mut templates = Vec::new();
        templates.append(&mut make_metric_unit_templates(
            "grams",
            Symbol::plain("g".to_owned()),
            KILO_BASE,
        ));
        env.build_unit_class("Mass".to_owned(), templates.into_iter());
    }
}
