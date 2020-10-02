use std::fmt::{self, Debug};
use crate::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Eq, Hash, PartialEq)]
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

impl Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)?;
        if let Some(sup) = &self.superscript {
            write!(f, "^{}", sup)?;
        }
        if let Some(sub) = &self.subscript {
            write!(f, "_{}", sub)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SymbolTable<'a> {
    parent: Option<&'a SymbolTable<'a>>,
    symbols: &'a HashMap<Symbol, Value>,
}

impl<'a> SymbolTable<'a> {
    pub fn new(symbols: &'a HashMap<Symbol, Value>) -> Self {
        Self {
            parent: None,
            symbols,
        }
    }

    pub fn child(parent: &'a SymbolTable<'a>, symbols: &'a HashMap<Symbol, Value>) -> Self {
        Self {
            parent: Some(parent),
            symbols,
        }
    }

    pub fn get(&self, symbol: &Symbol) -> Option<&Value> {
        if let Some(value) = self.symbols.get(symbol) {
            Some(value)
        } else if let Some(parent) = &self.parent {
            parent.get(symbol)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub enum FormulaError {
    MismatchedUnits,
    WrongNumberOfArgs,
    UndefinedSymbol,
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

impl From<Symbol> for Formula {
    fn from(symbol: Symbol) -> Self {
        Formula::Symbol(symbol)
    }
}

#[derive(Clone, Debug)]
pub enum Formula {
    Value(Value),
    PlainFunction { fun: Function, args: Vec<Formula> },
    Symbol(Symbol),
}

struct Ctx<'a> {
    environment: &'a Environment,
    symbol_table: SymbolTable<'a>,
}

impl Formula {
    pub fn try_compute(
        &self,
        environment: &Environment,
        symbol_table: SymbolTable<'_>,
    ) -> Result<Value, FormulaError> {
        let ctx = Ctx {
            environment,
            symbol_table,
        };
        self.try_compute_impl(&ctx)
    }

    fn try_compute_impl(&self, ctx: &Ctx<'_>) -> Result<Value, FormulaError> {
        match self {
            Self::Value(value) => Ok(value.clone()),
            Self::PlainFunction { fun, args } => {
                if args.len() != fun.num_args() {
                    return Err(FormulaError::WrongNumberOfArgs);
                }
                let mut arg_values = args
                    .iter()
                    .map(|f| f.try_compute_impl(ctx))
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
            },
            Self::Symbol(symbol) => {
                if let Some(value) = ctx.symbol_table.get(symbol) {
                    Ok(value.clone())
                } else {
                    Err(FormulaError::UndefinedSymbol)
                }
            }
        }
    }
}
