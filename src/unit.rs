use crate::formula::Symbol;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composite<T: Eq + Hash> {
    pub components: HashMap<T, i32>,
}

pub type CompositeUnit = Composite<UnitId>;
pub type CompositeUnitClass = Composite<UnitClassId>;
impl<T: Eq + Hash> Composite<T> {
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

fn add_ranged_metric_units(
    env: &mut crate::env::Environment,
    name: &str,
    symbol: &str,
    prefixless_ratio: f64,
    base_class: &CompositeUnitClass,
    start_prefix: usize,
    num_prefixes: usize,
) {
    let mut remaining = num_prefixes;
    for (full, prefix, multiplier) in METRIC_PREFIXES.iter().skip(start_prefix) {
        let symbol = Symbol::plain(format!("{}{}", prefix, symbol));
        env.store(Unit {
            name: util::to_title_case(&format!("{}{}", full, name)),
            symbol,
            base_ratio: prefixless_ratio * multiplier,
            base_class: base_class.clone(),
        });
        remaining -= 1;
        if remaining == 0 {
            break;
        }
    }
}

fn add_metric_units(
    env: &mut crate::env::Environment,
    name: &str,
    symbol: &str,
    prefixless_ratio: f64,
    base_class: &CompositeUnitClass,
) {
    add_ranged_metric_units(
        env,
        name,
        symbol,
        prefixless_ratio,
        base_class,
        0,
        METRIC_PREFIXES.len(),
    );
}

/// Do not add sub-unit divisions like milli or micro. Useful for discrete quantities like bytes.
fn add_upper_metric_units(
    env: &mut crate::env::Environment,
    name: &str,
    symbol: &str,
    prefixless_ratio: f64,
    base_class: &CompositeUnitClass,
) {
    debug_assert!(METRIC_PREFIXES[METRIC_PREFIXES.len() / 2].2 == 1e0);
    add_ranged_metric_units(
        env,
        name,
        symbol,
        prefixless_ratio,
        base_class,
        0,
        METRIC_PREFIXES.len() / 2 + 1,
    );
}

/// Do not add upper divisions like kilo, mega. Used for seconds.
fn add_lower_metric_units(
    env: &mut crate::env::Environment,
    name: &str,
    symbol: &str,
    prefixless_ratio: f64,
    base_class: &CompositeUnitClass,
) {
    debug_assert!(METRIC_PREFIXES[METRIC_PREFIXES.len() / 2].2 == 1e0);
    add_ranged_metric_units(
        env,
        name,
        symbol,
        prefixless_ratio,
        base_class,
        METRIC_PREFIXES.len() / 2,
        METRIC_PREFIXES.len() / 2 + 1,
    );
}

fn add_weird_units(
    env: &mut crate::env::Environment,
    info: &[(&str, &str, f64)],
    base_class: &CompositeUnitClass,
) {
    for (name, symbol, base_ratio) in info {
        env.store(Unit {
            name: name.to_string(),
            symbol: Symbol::plain(symbol.to_string()),
            base_ratio: *base_ratio,
            base_class: base_class.clone(),
        });
    }
}

pub fn add_default_units(env: &mut crate::env::Environment) {
    // =============================================================================================
    // BASE UNITS
    // =============================================================================================

    let mass: CompositeUnitClass = env.store(UnitClass("Mass".to_owned())).into();
    // The base unit is actually kilograms, so there are 1e-3 kilograms per gram.
    add_metric_units(env, "grams", "g", 1e-3, &mass);

    let length: CompositeUnitClass = env.store(UnitClass("Length".to_owned())).into();
    add_metric_units(env, "meters", "m", 1e0, &length);
    const METERS_PER_FOOT: f64 = 1.0 / 3.28084;
    add_weird_units(
        env,
        &[
            ("Feet", "ft", METERS_PER_FOOT),
            ("Inches", "in", METERS_PER_FOOT / 12.0),
            ("Yards", "yd", 3.0 * METERS_PER_FOOT),
            ("Chains", "ch", 22.0 * 3.0 * METERS_PER_FOOT),
            ("Furlongs", "fr", 220.0 * 3.0 * METERS_PER_FOOT),
            ("Miles", "mi", 5280.0 * METERS_PER_FOOT),
            ("Leagues", "lea", 15840.0 * METERS_PER_FOOT),
            ("Fathoms", "ftm", 1.852),
            ("Cables", "cables", 185.2),
            ("Nautical Miles", "nautical miles", 1852.0),
        ],
        &length,
    );

    let time: CompositeUnitClass = env.store(UnitClass("Time".to_owned())).into();
    add_lower_metric_units(env, "seconds", "s", 1e0, &time);
    add_weird_units(
        env,
        &[
            ("Minutes", "m", 60.0),
            ("Hours", "h", 60.0 * 60.0),
            ("Days", "d", 24.0 * 60.0 * 60.0),
        ],
        &time,
    );

    let current: CompositeUnitClass = env.store(UnitClass("Current".to_owned())).into();
    // Technically it's "Amperes" but everyone I have ever heard say or write any words uses "Amps"
    add_metric_units(env, "amps", "A", 1e0, &current);

    // Thermodynamic temperature is a temperature scale where zero is the point where there is no
    // heat energy in an object. E.G. the Kelvin scale.
    let thermodynamic_temperature: CompositeUnitClass = env
        .store(UnitClass("Thermodynamic Temperature".to_owned()))
        .into();
    add_weird_units(
        env,
        &[("Degrees Kelvin", "K", 1.0)],
        &thermodynamic_temperature,
    );

    // =============================================================================================
    // COMPOSITE UNITS
    // =============================================================================================

    let speed = length.clone() / time.clone();
    let acceleration = speed.clone() / time.clone();
    let force = mass.clone() * acceleration.clone();
    add_metric_units(env, "newtons", "N", 1.0, &force);
    add_weird_units(env, &[("Pound-Force", "lbs", 4.44822)], &force);
    let energy = force.clone() * length.clone();
    add_metric_units(env, "joules", "J", 1.0, &energy);

    let electric_charge = current.clone() * time.clone();
    add_metric_units(env, "coulumbs", "C", 1.0, &electric_charge);

    let electric_potential = energy.clone() / electric_charge.clone();
    add_metric_units(env, "volts", "V", 1.0, &electric_potential);

    let capacitance = electric_charge.clone() / electric_potential.clone();
    add_metric_units(env, "farads", "F", 1.0, &capacitance);
}
