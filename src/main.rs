use ackulator::prelude::*;

fn main() {
    let mut environment = ackulator::env::Environment::new();
    let kilograms = environment
        .find(|unit: &Unit| unit.name == "Kilograms")
        .unwrap();
    let feet = environment.find(|unit: &Unit| unit.name == "Feet").unwrap();
    let weird_unit = kilograms / feet;
    let weird_value = environment.make_scalar(12.0, weird_unit, 4);
    println!("{}", environment.format_scalar_detailed(&weird_value));
    let length = environment.make_scalar(1.0, feet.into(), 4);
    println!("{}", environment.format_scalar_detailed(&length));
    let weight = environment.make_scalar(1.0, kilograms.into(), 4);
    println!("{}", environment.format_scalar_detailed(&weight));
}
