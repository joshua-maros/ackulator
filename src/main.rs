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
    let mass = environment.make_scalar(5.0, kilograms.into(), 4).into();
    println!("{}", environment.format_value_detailed(&mass));
    let vacuum_permittivity = environment
        .find_global_symbol(&Symbol::plain("Vacuum Permittivity".to_owned()))
        .unwrap();
    println!(
        "{}",
        environment.format_value_detailed(&vacuum_permittivity)
    );

    let mut sphere = Entity::new();
    sphere.add_property(Symbol::plain("Radius".to_owned()), length.clone());
    sphere.add_property(Symbol::plain("Mass".to_owned()), mass.clone());
    println!("{}", environment.format_value_detailed(&sphere.clone().into()));

    let constant_symbol = Symbol::plain("H".to_owned());
    let get_length = Formula::GetEntityProperty {
        from: Box::new(constant_symbol.clone().into()),
        prop_name: Symbol::plain("Radius".to_owned()),
    };
    let weird_formula = Function::Mul.into_formula(vec![get_length, mass.clone().into()]);
    println!("{}", environment.format_formula_detailed(&weird_formula));

    let mut symbols = HashMap::new();
    symbols.insert(constant_symbol, sphere.clone().into());
    let parent = environment.borrow_global_symbols();
    let table = SymbolTable::child(&parent, &symbols);
    let formula_value = weird_formula.try_compute(&environment, table).unwrap();
    println!("{}", environment.format_value_detailed(&formula_value));
}
