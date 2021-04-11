use ackulator::units::UnitPrefixType;

fn main() {
    let mut instance = ackulator::instance::Instance::new();
    let class = ackulator::units::UnitClass {
        names: vec![format!("Length")],
    };
    let class = instance.add_unit_class(class.into()).unwrap();
    let unit = ackulator::units::Unit {
        class: class.into(),
        names: vec![format!("Meter"), format!("Meters")],
        symbol: format!("m"),
        base_ratio: 1.0,
    };
    instance.add_unit(unit, UnitPrefixType::Metric).unwrap();
    println!("{:#?}", instance);

    println!("{:#?}", ackulator::expression::parse_expression("1 * (2 + Hello) * World * 1 ^ 2 ^ 3"));
}
