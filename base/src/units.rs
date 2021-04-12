use crate::{data::Describe, prelude::*, storage::StorageId};
use std::{
    cmp::Ordering,
    fmt::{Debug, Formatter, Write},
    hash::Hash,
    ops::{Div, DivAssign, Index, Mul, MulAssign},
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

#[derive(Clone, Debug, PartialEq)]
struct QuantityBag<T: Ord> {
    items: Vec<(f64, T)>,
}

impl<T: Ord> QuantityBag<T> {
    fn item_index(&self, item: &T) -> Result<usize, usize> {
        self.items
            .binary_search_by(|candidate| candidate.1.cmp(item))
    }

    fn add(&mut self, item: T, amount: f64) {
        match self.item_index(&item) {
            Ok(exists_index) => {
                self.items[exists_index].0 += amount;
                if self.items[exists_index].0.abs() < 1e-10 {
                    self.items.remove(exists_index);
                }
            }
            Err(insert_at) => self.items.insert(insert_at, (amount, item)),
        }
    }

    fn mul(&mut self, factor: f64) {
        if factor == 0.0 {
            self.items.clear();
        }
        for (quantity, _) in &mut self.items {
            *quantity *= factor;
        }
    }

    fn union(self, other: Self) -> Self
    where
        T: Clone,
    {
        if self.items.len() == 0 {
            return other;
        }
        if other.items.len() == 0 {
            return self;
        }
        let mut self_items = self.items.into_iter();
        let mut other_items = other.items.into_iter();
        let mut new_items = Vec::new();
        let mut a = self_items.next().unwrap();
        let mut b = other_items.next().unwrap();
        loop {
            match a.1.cmp(&b.1) {
                Ordering::Equal => {
                    let new_quantity = a.0 + b.0;
                    if new_quantity.abs() > 1e-10 {
                        new_items.push((new_quantity, a.1));
                    }
                    match (self_items.next(), other_items.next()) {
                        (Some(na), Some(nb)) => {
                            a = na;
                            b = nb;
                        }
                        (Some(na), None) => {
                            new_items.push(na);
                            break;
                        }
                        (None, Some(nb)) => {
                            new_items.push(nb);
                            break;
                        }
                        (None, None) => break,
                    }
                }
                Ordering::Greater => {
                    new_items.push(b);
                    match other_items.next() {
                        Some(item) => b = item,
                        None => {
                            new_items.push(a);
                            break;
                        }
                    }
                }
                Ordering::Less => {
                    new_items.push(a);
                    match self_items.next() {
                        Some(item) => a = item,
                        None => {
                            new_items.push(b);
                            break;
                        }
                    }
                }
            }
        }
        for extra in self_items {
            new_items.push(extra);
        }
        for extra in other_items {
            new_items.push(extra);
        }
        debug_assert!({
            let mut sorted = new_items.clone();
            sorted.sort_by(|a, b| a.1.cmp(&b.1));
            sorted == new_items
        });
        Self { items: new_items }
    }

    fn get(&self, item: &T) -> f64 {
        if let Ok(exists_at) = self.item_index(item) {
            self.items[exists_at].0
        } else {
            0.0
        }
    }
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
pub struct Composite<I: Ord + Eq + Copy + Debug> {
    factors: QuantityBag<I>,
}

pub type CompositeUnitClass = Composite<UnitClassId>;
pub type CompositeUnit = Composite<UnitId>;

impl<I: Ord + Eq + Copy + Debug> Composite<I> {
    pub fn identity() -> Self {
        Self {
            factors: QuantityBag { items: Vec::new() },
        }
    }

    pub fn is_identity(&self) -> bool {
        self.factors.items.len() == 0
    }

    pub fn pow(&mut self, exp: f64) {
        self.factors.mul(exp);
    }
}

fn describe_factor<T>(
    into: &mut String,
    instance: &Instance,
    factor: &(f64, StorageId<T>),
    factor_describer: impl Fn(&T) -> &str,
) where
    Instance: Index<StorageId<T>, Output = T>,
{
    let power = factor.0;
    write!(into, "{}", factor_describer(&instance[factor.1])).unwrap();
    if (power - 1.0).abs() > 1e-10 {
        write!(into, "^{}", power).unwrap();
    }
}

impl Describe for CompositeUnitClass {
    fn describe(&self, into: &mut String, instance: &Instance) {
        if self.is_identity() {
            return;
        }
        let mut numerator = Vec::new();
        let mut denominator = Vec::new();
        for item in &self.factors.items {
            if item.0 > 0.0 {
                numerator.push(item);
            } else {
                denominator.push(item);
            }
        }
        if numerator.len() == 0 {
            write!(into, "1").unwrap();
        } else {
            describe_factor(into, instance, numerator[0], |uc| &uc.names[0][..]);
            for factor in &numerator[1..] {
                write!(into, " * ").unwrap();
                describe_factor(into, instance, factor, |uc| &uc.names[0][..]);
            }
        }
        if denominator.len() > 0 {
            write!(into, "/").unwrap();
            let factor = (-denominator[0].0, denominator[0].1);
            describe_factor(into, instance, &factor, |uc| &uc.names[0][..]);
            for factor in &denominator[1..] {
                let factor = (-factor.0, factor.1);
                write!(into, " * ").unwrap();
                describe_factor(into, instance, &factor, |uc| &uc.names[0][..]);
            }
        }
    }
}

impl CompositeUnit {
    pub fn base_ratio(&self, instance: &Instance) -> f64 {
        let mut ratio = 1.0;
        for (power, unit) in &self.factors.items {
            ratio *= instance[*unit].base_ratio.powf(*power);
        }
        ratio
    }

    pub fn unit_class(&self, instance: &Instance) -> CompositeUnitClass {
        let mut result = CompositeUnitClass::identity();
        for (power, unit_id) in &self.factors.items {
            let mut class = instance[*unit_id].class.clone();
            class.factors.mul(*power);
            result.factors = result.factors.union(class.factors);
        }
        result
    }

    pub fn as_scalar(&self, instance: &Instance) -> Scalar {
        Scalar::new(
            self.base_ratio(instance),
            Precision::Exact,
            self.unit_class(instance),
            self.clone(),
        )
    }
}

impl Describe for CompositeUnit {
    fn describe(&self, into: &mut String, instance: &Instance) {
        if self.is_identity() {
            return;
        }
        let mut numerator = Vec::new();
        let mut denominator = Vec::new();
        for item in &self.factors.items {
            if item.0 > 0.0 {
                numerator.push(item);
            } else {
                denominator.push(item);
            }
        }

        let nl = numerator.len();
        for factor in numerator {
            describe_factor(into, instance, factor, |u| &u.symbol[..]);
        }
        if nl == 0 {
            write!(into, "1").unwrap();
        }
        if denominator.len() > 0 {
            write!(into, "/").unwrap();
        }
        for factor in denominator {
            let factor = (-factor.0, factor.1);
            describe_factor(into, instance, &factor, |u| &u.symbol[..]);
        }
    }
}

impl<I: Ord + Eq + Copy + Debug> Debug for Composite<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for (power, item) in &self.factors.items {
            if first {
                first = false;
            } else {
                write!(f, " * ")?;
            }
            write!(f, "({:?})^{}", item, power)?;
        }
        if self.factors.items.len() == 0 {
            write!(f, "1")?;
        }
        Ok(())
    }
}

impl<I: Ord + Eq + Copy + Debug> From<I> for Composite<I> {
    fn from(item: I) -> Self {
        Self {
            factors: QuantityBag {
                items: vec![(1.0, item)],
            },
        }
    }
}

impl<I: Ord + Eq + Copy + Debug> Mul for Composite<I> {
    type Output = Self;
    fn mul(mut self, rhs: Self) -> Self::Output {
        self.factors = self.factors.union(rhs.factors);
        self
    }
}

impl<I: Ord + Eq + Copy + Debug> MulAssign for Composite<I> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.clone() * rhs;
    }
}

impl<I: Ord + Eq + Copy + Debug> Div for Composite<I> {
    type Output = Self;
    fn div(mut self, mut rhs: Self) -> Self::Output {
        rhs.factors.mul(-1.0);
        self.factors = self.factors.union(rhs.factors);
        self
    }
}

impl<I: Ord + Eq + Copy + Debug> DivAssign for Composite<I> {
    fn div_assign(&mut self, rhs: Self) {
        *self = self.clone() * rhs;
    }
}

impl<I: Ord + Eq + Copy + Debug> PartialEq for Composite<I> {
    fn eq(&self, other: &Self) -> bool {
        (self.clone() / other.clone()).is_identity()
    }
}

impl<I: Ord + Eq + Copy + Debug> Eq for Composite<I> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_operation() {
        let mut set1 = QuantityBag { items: Vec::new() };
        set1.add(7, 10.0);
        set1.add(9, 10.0);
        set1.add(3, 1.0);
        set1.add(5, 1.0);
        set1.add(2, 1.0);
        let mut set2 = QuantityBag { items: Vec::new() };
        set2.add(1, 1.0);
        set2.add(2, 1.0);
        set2.add(3, 1.0);
        let set = set1.union(set2);
        assert_eq!(set.items.len(), 6);
        assert_eq!(set.get(&2), 2.0);
        assert_eq!(set.get(&9), 10.0);
    }

    #[test]
    fn fully_disjoint_union_operation() {
        let mut set1 = QuantityBag { items: Vec::new() };
        set1.add(1, 10.0);
        let mut set2 = QuantityBag { items: Vec::new() };
        set2.add(2, 20.0);
        let set = set1.clone().union(set2.clone());
        assert_eq!(set.items.len(), 2);
        assert_eq!(set.get(&1), 10.0);
        assert_eq!(set.get(&2), 20.0);

        // Check that the operator is symmetric
        assert_eq!(set, set2.union(set1));
    }

    #[test]
    fn union_operation_cancel_out() {
        let mut set1 = QuantityBag { items: Vec::new() };
        set1.add(1, 10.0);
        let mut set2 = set1.clone();
        set2.mul(-1.0);
        // The two sets should cancel out to make an empty set.
        let set = set1.union(set2);
        assert_eq!(set.items.len(), 0);
    }
}
