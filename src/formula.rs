use crate::unit::{CompositeUnit, CompositeUnitClass};

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
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub text: String,
    pub superscript: Option<String>,
    pub subscript: Option<String>,
}

impl Symbol {
    pub fn plain(text: String) -> Self {
        Self {
            text,
            superscript: None,
            subscript: None,
        }
    }
}
