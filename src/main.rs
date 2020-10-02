use ackulator::prelude::*;
use std::collections::HashMap;

fn main() {
    let mut environment = ackulator::env::Environment::new();
    let kilograms = environment
        .find(|unit: &Unit| unit.name == "Kilograms")
        .unwrap();
    let feet = environment.find(|unit: &Unit| unit.name == "Feet").unwrap();

    let length = environment.make_scalar(1.0, feet.into(), 4).into();
    println!("{}", environment.format_value_detailed(&length));
    let weight = environment.make_scalar(5.0, kilograms.into(), 4).into();
    println!("{}", environment.format_value_detailed(&weight));

    let constant_symbol = Symbol::plain("H".to_owned());
    let weird_formula =
        Function::Mul.into_formula(vec![length.clone().into(), weight.clone().into()]);
    let weird_formula =
        Function::Mul.into_formula(vec![constant_symbol.clone().into(), weird_formula]);
    println!("{}", environment.format_formula_detailed(&weird_formula));

    let mut symbol_table = HashMap::new();
    symbol_table.insert(constant_symbol, weight);
    let formula_value = weird_formula
        .try_compute(&environment, &symbol_table)
        .unwrap();
    println!("{}", environment.format_value_detailed(&formula_value));
}
