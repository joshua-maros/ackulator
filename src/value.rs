use crate::formula::FormulaError;
use crate::prelude::*;
use crate::unit::CompositeUnitClass;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Scalar {
    pub(crate) base_value: f64,
    pub(crate) base_unit: CompositeUnitClass,
    pub(crate) display_unit: CompositeUnit,
    pub(crate) precision: u32,
}

impl Scalar {
    pub(crate) fn new(
        base_value: f64,
        base_unit: CompositeUnitClass,
        display_unit: CompositeUnit,
        precision: u32,
    ) -> Self {
        assert!(precision > 0);
        Self {
            base_value,
            base_unit,
            display_unit,
            precision,
        }
    }

    pub fn unitless(value: f64, precision: u32) -> Self {
        assert!(precision > 0);
        Self {
            base_value: value,
            base_unit: CompositeUnitClass::unitless(),
            display_unit: CompositeUnit::unitless(),
            precision,
        }
    }

    pub fn map_in_place(&mut self, fun: &impl Fn(f64) -> f64) {
        self.base_value = fun(self.base_value);
    }
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub(crate) properties: HashMap<Symbol, Value>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    pub fn add_property(&mut self, name: Symbol, value: Value) {
        self.properties.insert(name, value);
    }

    pub fn get_property(&self, name: &Symbol) -> Option<&Value> {
        self.properties.get(name)
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Scalar(Scalar),
    Vector,
    Entity(Entity),
}

impl From<Scalar> for Value {
    fn from(value: Scalar) -> Value {
        Value::Scalar(value)
    }
}

impl From<Entity> for Value {
    fn from(value: Entity) -> Value {
        Value::Entity(value)
    }
}

impl Value {
    fn try_op<B, D, P>(
        &self,
        rhs: &Self,
        op: impl FnOnce(f64, f64) -> f64,
        base_unit_transform: B,
        display_unit_transform: D,
        precision_transform: P,
    ) -> Result<Self, FormulaError>
    where
        B: FnOnce(
            &CompositeUnitClass,
            &CompositeUnitClass,
        ) -> Result<CompositeUnitClass, FormulaError>,
        D: FnOnce(&CompositeUnit, &CompositeUnit) -> CompositeUnit,
        P: FnOnce(f64, u32, f64, u32) -> u32,
    {
        use Value as V;
        if let (V::Scalar(lhs), V::Scalar(rhs)) = (self, rhs) {
            let precision =
                precision_transform(lhs.base_value, lhs.precision, rhs.base_value, rhs.precision);
            Ok(V::Scalar(Scalar {
                base_value: op(lhs.base_value, rhs.base_value),
                base_unit: base_unit_transform(&lhs.base_unit, &rhs.base_unit)?,
                display_unit: display_unit_transform(&lhs.display_unit, &rhs.display_unit),
                precision,
            }))
        } else if let V::Entity(..) = self {
            Err(FormulaError::MathOnEntity)
        } else if let V::Entity(..) = rhs {
            Err(FormulaError::MathOnEntity)
        } else {
            unreachable!()
        }
    }

    /// Checks that the base units are the same, then returns the first one.
    fn additive_base_unit_transform(
        lhs: &CompositeUnitClass,
        rhs: &CompositeUnitClass,
    ) -> Result<CompositeUnitClass, FormulaError> {
        if lhs != rhs {
            Err(FormulaError::MismatchedUnits)
        } else {
            Ok(lhs.clone())
        }
    }

    /// Returns the exponent of value when expressed in scientific notation. It will be at least
    /// -9999.
    fn sci_exp_of(value: f64) -> i32 {
        (value.abs().log10().floor() as i32).max(-9999)
    }

    /// Returns the precision of the value that would result from adding the two values.
    fn additive_precision_transform(lhsv: f64, lhsp: u32, rhsv: f64, rhsp: u32) -> u32 {
        // The digit where lhsv or rhsv starts.
        let lhs_start = Self::sci_exp_of(lhsv);
        let rhs_start = Self::sci_exp_of(rhsv);
        // The precision we pick should start where the biggest input starts.
        let start = lhs_start.max(rhs_start);
        // The last digit where lhsv or rhsv is still precise.
        let lhs_end = lhs_start - lhsp as i32;
        let rhs_end = rhs_start - rhsp as i32;
        // The precision we pick should only go to the farthest decimal place that is still precise
        // in both inputs.
        let end = lhs_end.max(rhs_end);
        let precision = start - end;
        assert!(precision > 0);
        precision as u32
    }

    pub fn try_add(&self, rhs: &Self) -> Result<Self, FormulaError> {
        self.try_op(
            rhs,
            std::ops::Add::add,
            Self::additive_base_unit_transform,
            |lhs, _rhs| lhs.clone(),
            Self::additive_precision_transform,
        )
    }

    pub fn try_sub(&self, rhs: &Self) -> Result<Self, FormulaError> {
        self.try_op(
            rhs,
            std::ops::Sub::sub,
            Self::additive_base_unit_transform,
            |lhs, _rhs| lhs.clone(),
            Self::additive_precision_transform,
        )
    }

    pub fn try_mul(&self, rhs: &Self) -> Result<Self, FormulaError> {
        self.try_op(
            rhs,
            std::ops::Mul::mul,
            |lhs, rhs| Ok(lhs.clone() * rhs.clone()),
            |lhs, rhs| lhs.clone() * rhs.clone(),
            |_, lhsp, _, rhsp| lhsp.min(rhsp),
        )
    }

    pub fn try_div(&self, rhs: &Self) -> Result<Self, FormulaError> {
        self.try_op(
            rhs,
            std::ops::Div::div,
            |lhs, rhs| Ok(lhs.clone() / rhs.clone()),
            |lhs, rhs| lhs.clone() / rhs.clone(),
            |_, lhsp, _, rhsp| lhsp.min(rhsp),
        )
    }

    pub fn map_in_place(&mut self, fun: &impl Fn(f64) -> f64) {
        match self {
            Self::Scalar(scalar) => scalar.map_in_place(fun),
            Self::Vector => unimplemented!(),
            Self::Entity(entity) => {
                for (_key, value) in entity.properties.iter_mut() {
                    value.map_in_place(fun);
                }
            }
        }
    }

    pub fn map(mut self, fun: &impl Fn(f64) -> f64) -> Self {
        self.map_in_place(fun);
        self
    }
}
