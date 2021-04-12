use crate::{data::Describe, prelude::*};
use std::{
    fmt::Write,
    ops::{Div, Mul, Neg},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Precision {
    SigFigs(i32),
    PercentError(f64),
    Exact,
}

impl Precision {
    pub fn percent_error(self, for_value: f64) -> f64 {
        match self {
            Self::SigFigs(sf) => {
                // The accurate range of the original value is +- this number.
                let range = 10f64.powi(for_value.log10().trunc() as i32 + 1 - sf);
                range / for_value
            }
            Self::PercentError(p) => p,
            Self::Exact => 0.0,
        }
    }
}

#[scones::make_constructor]
#[derive(Clone, Debug)]
pub struct Scalar {
    value: f64,
    precision: Precision,
    unit: CompositeUnitClass,
    display_unit: CompositeUnit,
}

impl Scalar {
    pub fn add(&self, other: &Self) -> Result<Self, ()> {
        use Precision::*;
        if self.unit != other.unit {
            return Err(());
        }
        let new_value = self.value + other.value;
        let lhs_order = self.value.log10() as i32;
        let rhs_order = other.value.log10() as i32;
        let new_order = new_value.log10() as i32;
        let new_precision = match ((self.value, self.precision), (other.value, other.precision)) {
            ((_, Exact), (_, Exact)) => Exact,
            ((_, Exact), (_, other)) => other,
            ((_, other), (_, Exact)) => other,
            ((_, SigFigs(lhs_sf)), (_, SigFigs(rhs_sf))) => {
                let last_known_digit = i32::max(lhs_order - lhs_sf, rhs_order - rhs_sf);
                SigFigs((new_order - last_known_digit).max(0))
            }
            ((pct_value, PercentError(pct)), (other_value, other))
            | ((other_value, other), (pct_value, PercentError(pct))) => {
                let pct_range = pct_value * pct;
                let other_range = other_value * other.percent_error(other_value);
                let new_range = pct_range + other_range;
                PercentError(new_range / new_value)
            }
        };
        Ok(Self {
            value: new_value,
            precision: new_precision,
            unit: self.unit.clone(),
            display_unit: self.display_unit.clone(),
        })
    }

    pub fn sub(&self, other: &Self) -> Result<Self, ()> {
        self.add(&-other.clone())
    }

    pub fn pow(&self, other: &Self, instance: &Instance) -> Result<Self, ()> {
        let mut res = self.clone();
        let exp = other.display_value(instance);
        res.value = res.value.powf(exp);
        res.unit.pow(exp);
        res.display_unit.pow(exp);
        Ok(res)
    }

    pub fn unit(&self) -> &CompositeUnitClass {
        &self.unit
    }

    pub fn display_unit(&self) -> &CompositeUnit {
        &self.display_unit
    }

    pub fn set_display_unit(&mut self, display_unit: CompositeUnit) {
        self.display_unit = display_unit;
    }

    pub fn display_value(&self, instance: &Instance) -> f64 {
        self.value / self.display_unit.base_ratio(instance)
    }

    pub fn raw_value(&self) -> f64 {
        self.value
    }
}

impl Describe for Scalar {
    fn describe(&self, into: &mut String, instance: &Instance) {
        macro_rules! put {
            ($($t:tt)*) => {
                write!(into, $($t)*).unwrap();
            }
        }
        put!("{}", self.value / self.display_unit.base_ratio(instance));
        if !self.display_unit.is_identity() {
            self.display_unit.describe(into, instance);
        }
    }
}

impl Mul for Scalar {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        use Precision::*;
        // https://www.utm.edu/staff/cerkal/Lect4.html
        let new_precision = match ((self.value, self.precision), (rhs.value, rhs.precision)) {
            ((_, Exact), (_, Exact)) => Exact,
            ((_, Exact), (_, other)) => other,
            ((_, other), (_, Exact)) => other,
            ((_, SigFigs(lhs_sf)), (_, SigFigs(rhs_sf))) => SigFigs(lhs_sf.min(rhs_sf)),
            ((_, PercentError(pct)), (rhs_value, rhs))
            | ((rhs_value, rhs), (_, PercentError(pct))) => {
                let rhs_pct = rhs.percent_error(rhs_value);
                PercentError((pct * pct + rhs_pct * rhs_pct).sqrt())
            }
        };
        Self {
            value: self.value * rhs.value,
            precision: new_precision,
            unit: self.unit * rhs.unit,
            display_unit: self.display_unit * rhs.display_unit,
        }
    }
}

impl Div for Scalar {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        use Precision::*;
        // https://www.utm.edu/staff/cerkal/Lect4.html
        let new_precision = match ((self.value, self.precision), (rhs.value, rhs.precision)) {
            ((_, Exact), (_, Exact)) => Exact,
            ((_, Exact), (_, other)) => other,
            ((_, other), (_, Exact)) => other,
            ((_, SigFigs(lhs_sf)), (_, SigFigs(rhs_sf))) => SigFigs(lhs_sf.min(rhs_sf)),
            ((_, PercentError(pct)), (rhs_value, rhs))
            | ((rhs_value, rhs), (_, PercentError(pct))) => {
                let rhs_pct = rhs.percent_error(rhs_value);
                PercentError((pct * pct + rhs_pct * rhs_pct).sqrt())
            }
        };
        Self {
            value: self.value / rhs.value,
            precision: new_precision,
            unit: self.unit / rhs.unit,
            display_unit: self.display_unit / rhs.display_unit,
        }
    }
}

impl Neg for Scalar {
    type Output = Self;
    fn neg(mut self) -> Self {
        self.value = -self.value;
        self
    }
}
