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

    let class = ackulator::units::UnitClass {
        names: vec![format!("Time")],
    };
    let class = instance.add_unit_class(class.into()).unwrap();
    let unit = ackulator::units::Unit {
        class: class.into(),
        names: vec![format!("Second"), format!("Seconds")],
        symbol: format!("s"),
        base_ratio: 1.0,
    };
    instance
        .add_unit(unit, UnitPrefixType::PartialMetric)
        .unwrap();
    // println!("{:#?}", instance);

    // let (_, expr) = ackulator::expression::parse_expression("3e-30 * Yottameters").unwrap();
    let (_, statement) = ackulator::statement::parse_statement(
        "show 30 * Kilometers / Second + 1 * Meter / Millisecond",
    )
    .unwrap();
    instance.execute_statement(&statement).unwrap();
}
