use crate::prelude::*;
use std::ops::{Div, Mul};

#[derive(Clone, Debug)]
pub struct UnitClass {
    pub names: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub names: Vec<String>,
    pub class: CompositeUnitClass,
    pub symbol: String,
    // Multiply a value in the base unit by this number to get the value in this unit.
    pub base_ratio: f64,
}

#[derive(Clone, Debug)]
pub struct Composite<I: Eq + Copy> {
    pub numerator_factors: Vec<I>,
    pub denominator_factors: Vec<I>,
}

pub type CompositeUnitClass = Composite<UnitClassId>;
pub type CompositeUnit = Composite<UnitId>;

impl<I: Eq + Copy> Composite<I> {
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

impl<I: Eq + Copy> From<I> for Composite<I> {
    fn from(item: I) -> Self {
        Self {
            numerator_factors: vec![item],
            denominator_factors: Vec::new(),
        }
    }
}

impl<I: Eq + Copy> Mul for Composite<I> {
    type Output = Self;
    fn mul(mut self, mut rhs: Self) -> Self::Output {
        self.numerator_factors.append(&mut rhs.numerator_factors);
        self.denominator_factors
            .append(&mut rhs.denominator_factors);
        self.simplify();
        self
    }
}

impl<I: Eq + Copy> Div for Composite<I> {
    type Output = Self;
    fn div(mut self, mut rhs: Self) -> Self::Output {
        self.numerator_factors.append(&mut rhs.denominator_factors);
        self.denominator_factors.append(&mut rhs.numerator_factors);
        self.simplify();
        self
    }
}

impl<I: Eq + Copy> PartialEq for Composite<I> {
    fn eq(&self, other: &Self) -> bool {
        (self.clone() / other.clone()).is_identity()
    }
}

impl<I: Eq + Copy> Eq for Composite<I> {}
