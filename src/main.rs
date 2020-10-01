use ackulator::prelude::*;

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
    let weird_value = length.try_div(&weight).unwrap();
    println!("{}", environment.format_value_detailed(&weird_value));

    let weight2 = environment.make_scalar(100.0, kilograms.into(), 3).into();
    println!("{}", environment.format_value_detailed(&weight2));
    let weight3 = weight.try_add(&weight2).unwrap();
    println!("{}", environment.format_value_detailed(&weight3));
}
