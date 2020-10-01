use crate::formula::{Symbol};
use crate::prelude::*;
use crate::util::{self, ItemId};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Div, Mul};

pub type UnitId = ItemId<Unit>;
pub type UnitClassId = ItemId<UnitClass>;

#[derive(Clone, Debug)]
pub struct UnitClass(pub String);

#[derive(Clone, Debug)]
pub struct Unit {
    pub name: String,
    pub symbol: Symbol,
    /// How many of the base unit is represented by 1 of this unit.
    pub base_ratio: f64,
    /// Use Length^2 for area, Time^-1 for frequency, etc.
    pub base_class: CompositeUnitClass,
}

#[derive(Clone, Debug)]
pub struct Composite<T> {
    pub components: HashMap<T, i32>,
}

pub type CompositeUnit = Composite<UnitId>;
pub type CompositeUnitClass = Composite<UnitClassId>;
impl<T> Composite<T> {
    pub fn unitless() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
}

impl<T: Eq + Hash> From<T> for Composite<T> {
    fn from(other: T) -> Self {
        Self {
            components: vec![(other, 1)].into_iter().collect(),
        }
    }
}

impl<T: Eq + Hash> Mul for Composite<T> {
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

impl<T: Eq + Hash + Into<Composite<T>>> Mul<T> for Composite<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
        self * rhs.into()
    }
}

impl Mul for UnitId {
    type Output = CompositeUnit;

    fn mul(self, rhs: Self) -> CompositeUnit {
        <UnitId as Into<CompositeUnit>>::into(self) * rhs
    }
}

impl Mul for UnitClassId {
    type Output = CompositeUnitClass;

    fn mul(self, rhs: Self) -> CompositeUnitClass {
        <UnitClassId as Into<CompositeUnitClass>>::into(self) * rhs
    }
}

impl<T: Eq + Hash> Div for Composite<T> {
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

impl<T: Eq + Hash + Into<Composite<T>>> Div<T> for Composite<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self {
        self / rhs.into()
    }
}

impl Div for UnitId {
    type Output = CompositeUnit;

    fn div(self, rhs: Self) -> CompositeUnit {
        <UnitId as Into<CompositeUnit>>::into(self) / rhs
    }
}

impl Div for UnitClassId {
    type Output = CompositeUnitClass;

    fn div(self, rhs: Self) -> CompositeUnitClass {
        <UnitClassId as Into<CompositeUnitClass>>::into(self) / rhs
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

fn add_metric_units(
    env: &mut crate::env::Environment,
    name: &str,
    symbol: Symbol,
    prefixless_ratio: f64,
    base_class: CompositeUnitClass,
) {
    for (full, prefix, multiplier) in &METRIC_PREFIXES {
        let mut symbol = symbol.clone();
        symbol.text = format!("{}{}", prefix, symbol.text);
        env.store(Unit {
            name: util::to_title_case(&format!("{}{}", full, name)),
            symbol,
            base_ratio: prefixless_ratio * multiplier,
            base_class: base_class.clone(),
        });
    }
}

pub fn add_default_units(env: &mut crate::env::Environment) {
    let mass: CompositeUnitClass = env.store(UnitClass("Mass".to_owned())).into();
    add_metric_units(
        env,
        "grams",
        Symbol::plain("g".to_owned()),
        1e-3, // The base unit is actually kilograms, so there are 1e-3 kilograms per gram.
        mass.clone(),
    );

    let length: CompositeUnitClass = env.store(UnitClass("Length".to_owned())).into();
    add_metric_units(
        env,
        "meters",
        Symbol::plain("m".to_owned()),
        1e0,
        length.clone(),
    );
    const METERS_PER_FOOT: f64 = 1.0 / 3.28084;
    env.store(Unit {
        name: "Feet".to_owned(),
        symbol: Symbol::plain("ft".to_owned()),
        base_ratio: METERS_PER_FOOT,
        base_class: length.clone(),
    });
    env.store(Unit {
        name: "Inches".to_owned(),
        symbol: Symbol::plain("in".to_owned()),
        base_ratio: METERS_PER_FOOT / 12.0,
        base_class: length.clone(),
    });
    env.store(Unit {
        name: "Yards".to_owned(),
        symbol: Symbol::plain("yd".to_owned()),
        base_ratio: 3.0 * METERS_PER_FOOT,
        base_class: length.clone(),
    });
    env.store(Unit {
        name: "Chains".to_owned(),
        symbol: Symbol::plain("ch".to_owned()),
        base_ratio: 22.0 * 3.0 * METERS_PER_FOOT,
        base_class: length.clone(),
    });
    env.store(Unit {
        name: "Furlongs".to_owned(),
        symbol: Symbol::plain("fr".to_owned()),
        base_ratio: 220.0 * 3.0 * METERS_PER_FOOT,
        base_class: length.clone(),
    });
    env.store(Unit {
        name: "Miles".to_owned(),
        symbol: Symbol::plain("mi".to_owned()),
        base_ratio: 5280.0 * METERS_PER_FOOT,
        base_class: length.clone(),
    });
    env.store(Unit {
        name: "Leagues".to_owned(),
        symbol: Symbol::plain("lea".to_owned()),
        base_ratio: 15840.0 * METERS_PER_FOOT,
        base_class: length.clone(),
    });
    env.store(Unit {
        name: "Fathoms".to_owned(),
        symbol: Symbol::plain("ftm".to_owned()),
        base_ratio: 1.852,
        base_class: length.clone(),
    });
    env.store(Unit {
        name: "Cables".to_owned(),
        symbol: Symbol::plain("cables".to_owned()),
        base_ratio: 185.2,
        base_class: length.clone(),
    });
    env.store(Unit {
        name: "Nautical Miles".to_owned(),
        symbol: Symbol::plain("nautical miles".to_owned()),
        base_ratio: 1852.0,
        base_class: length.clone(),
    });
}
