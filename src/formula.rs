use crate::unit::CompositeUnit;

#[derive(Clone, Debug)]
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

impl Symbol {
    pub fn plain(text: String) -> Self {
        Self {
            text,
            superscript: None,
            subscript: None,
        }
    }
}
