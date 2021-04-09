use crate::prelude::*;

pub struct Equation {
    pub lhs: Formula,
    pub rhs: Formula,
    pub variables: Vec<Symbol>,
    pub symbol_setup: Vec<(Symbol, Formula)>,
    pub conditions: Vec<CheckableStatement>,
}

impl Equation {
    pub fn new(lhs: Formula, rhs: Formula) -> Self {
        Self {
            lhs,
            rhs,
            variables: Vec::new(),
            symbol_setup: Vec::new(),
            conditions: Vec::new(),
        }
    }
}
