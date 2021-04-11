use crate::prelude::*;
use std::{
    fmt::{Debug, Formatter},
    ops::{Div, Mul},
};

#[derive(Clone, Debug)]
pub struct UnitClass {
    pub names: Vec<String>,
}

pub const METRIC_PREFIXES: &[(&str, &str, f64)] = &[
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
    // ----------------
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
pub const SMALL_PREFIXES_START: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitPrefixType {
    /// A plain unit.
    None,
    /// Add prefixes like milli, giga, etc.
    Metric,
    /// Add prefixes that reduce magnitude but not ones that increase it. This is basically only
    /// used for seconds because milliseconds are a thing but kiloseconds aren't.
    PartialMetric,
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub names: Vec<String>,
    pub class: CompositeUnitClass,
    pub symbol: String,
    // Multiply a value in the base unit by this number to get the value in this unit.
    pub base_ratio: f64,
}

#[derive(Clone)]
pub struct Composite<I: Eq + Copy + Debug> {
    pub numerator_factors: Vec<I>,
    pub denominator_factors: Vec<I>,
}

pub type CompositeUnitClass = Composite<UnitClassId>;
pub type CompositeUnit = Composite<UnitId>;

impl<I: Eq + Copy + Debug> Composite<I> {
    pub fn is_identity(&self) -> bool {
        self.numerator_factors.len() == 0 && self.denominator_factors.len() == 0
    }

    pub fn simplify(&mut self) {
        for ni in (0..self.numerator_factors.len()).rev() {
            for di in (0..self.denominator_factors.len()).rev() {
                if self.numerator_factors[ni] == self.denominator_factors[di] {
                    self.numerator_factors.remove(ni);
                    self.denominator_factors.remove(di);
                    break;
                }
            }
        }
    }
}
impl<I: Eq + Copy + Debug> Debug for Composite<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.numerator_factors.len() == 0 {
            write!(f, "1")?;
        } else if self.numerator_factors.len() == 1 {
            write!(f, "{:?}", self.numerator_factors[0])?;
        } else {
            write!(f, "({:?}", self.numerator_factors[0])?;
            for factor in &self.numerator_factors[1..] {
                write!(f, " * {:?} ", factor)?;
            }
            write!(f, ")")?;
        }
        if self.denominator_factors.len() == 1 {
            write!(f, "/ {:?}", self.denominator_factors[0])?;
        } else if self.denominator_factors.len() > 1 {
            write!(f, "/ ({:?}", self.denominator_factors[0])?;
            for factor in &self.denominator_factors[1..] {
                write!(f, " * {:?} ", factor)?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl<I: Eq + Copy + Debug> From<I> for Composite<I> {
    fn from(item: I) -> Self {
        Self {
            numerator_factors: vec![item],
            denominator_factors: Vec::new(),
        }
    }
}

impl<I: Eq + Copy + Debug> Mul for Composite<I> {
    type Output = Self;
    fn mul(mut self, mut rhs: Self) -> Self::Output {
        self.numerator_factors.append(&mut rhs.numerator_factors);
        self.denominator_factors
            .append(&mut rhs.denominator_factors);
        self.simplify();
        self
    }
}

impl<I: Eq + Copy + Debug> Div for Composite<I> {
    type Output = Self;
    fn div(mut self, mut rhs: Self) -> Self::Output {
        self.numerator_factors.append(&mut rhs.denominator_factors);
        self.denominator_factors.append(&mut rhs.numerator_factors);
        self.simplify();
        self
    }
}

impl<I: Eq + Copy + Debug> PartialEq for Composite<I> {
    fn eq(&self, other: &Self) -> bool {
        (self.clone() / other.clone()).is_identity()
    }
}

impl<I: Eq + Copy + Debug> Eq for Composite<I> {}
