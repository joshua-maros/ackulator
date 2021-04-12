const BOOTSTRAP: &str = r#"
make entity_class called metric
make entity_class called partial_metric

make unit_class called Length
make base_unit called Meter, Meters {
    class: Length,
    symbol: "m",
    metric,
}
make derived_unit called Foot, Feet {
    symbol: "ft",
    value: 0.3048 * Meters,
}

make unit_class called Time
make base_unit called Second, Seconds {
    class: Time,
    symbol: "s",
    partial_metric,
}

make label called Velocity for Length / Time 
make label called Acceleration for Velocity / Time

show (1 * Meter / Second ^ 2) is Acceleration
"#;

fn main() {
    let mut instance = ackulator::instance::Instance::new();

    // println!("{}", &BOOTSTRAP[37..]);
    let (remainder, statements) = ackulator::statement::parse_statements(BOOTSTRAP).unwrap();
    assert_eq!(remainder.len(), 0, "{}", remainder);

    for statement in statements {
        // println!("{:#?}", statement);
        instance.execute_statement(statement).unwrap();
    }

    // let res = ackulator::expression::parse_expression("1 * Meter + 1 * Feet").unwrap();
    // println!("{:#?}", instance.resolve_expression(&res.1, Default::default()));
}
