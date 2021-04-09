use crate::prelude::*;

pub enum ActionableStatement {
    ValueIs,
    FindValueUsing,
    ValueIsA,
}

pub enum CheckableStatement {
    ValueIs,
    ValueIsA,
}
