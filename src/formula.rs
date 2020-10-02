use crate::prelude::*;

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

#[derive(Clone, Debug)]
pub enum FormulaError {
    MismatchedUnits,
    WrongNumberOfArgs,
}

#[derive(Clone, Copy, Debug)]
pub enum Function {
    Add,
    Sub,
    Mul,
    Div,
    Sin,
    Cos,
    Tan,
}

impl Function {
    pub fn num_args(self) -> usize {
        match self {
            Self::Add | Self::Sub | Self::Mul | Self::Div => 2,
            Self::Sin | Self::Cos | Self::Tan => 1,
        }
    }

    pub fn debug_name(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Div => "div",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
        }
    }

    pub fn into_formula(self, args: Vec<Formula>) -> Formula {
        assert!(args.len() == self.num_args());
        Formula::PlainFunction { fun: self, args }
    }
}

impl From<Value> for Formula {
    fn from(value: Value) -> Self {
        Formula::Value(value)
    }
}

#[derive(Clone, Debug)]
pub enum Formula {
    Value(Value),
    PlainFunction { fun: Function, args: Vec<Formula> },
}

impl Formula {
    pub fn try_compute(&self) -> Result<Value, FormulaError> {
        match self {
            Self::Value(value) => Ok(value.clone()),
            Self::PlainFunction { fun, args } => {
                if args.len() != fun.num_args() {
                    return Err(FormulaError::WrongNumberOfArgs);
                }
                let mut arg_values = args
                    .iter()
                    .map(Formula::try_compute)
                    .collect::<Result<Vec<_>, _>>()?;
                use Function::*;
                match fun {
                    Add => arg_values[0].try_add(&arg_values[1]),
                    Sub => arg_values[0].try_sub(&arg_values[1]),
                    Mul => arg_values[0].try_mul(&arg_values[1]),
                    Div => arg_values[0].try_div(&arg_values[1]),
                    Sin => Ok(arg_values.pop().unwrap().map(f64::sin)),
                    Cos => Ok(arg_values.pop().unwrap().map(f64::cos)),
                    Tan => Ok(arg_values.pop().unwrap().map(f64::tan)),
                }
            }
        }
    }
}
