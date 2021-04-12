const BOOTSTRAP: &str = r#"
make unit_class called Length
make entity_class called metric
make entity_class called partial_metric
make base_unit called Meter, Meters {
    class: Length,
    symbol: "m",
    metric,
}
make derived_unit called Foot, Feet {
    symbol: "ft",
    value: 0.3048 * Meters,
}
show 1 * Foot * 1 * Meter
"#;

fn main() {
    let mut instance = ackulator::instance::Instance::new();

    // println!("{}", &BOOTSTRAP[37..]);
    // let res = ackulator::expression::parse_expression("{ s: \"asdf\",\nclass, }").unwrap();
    // println!("{:#?}", res);
    let (remainder, statements) = ackulator::statement::parse_statements(BOOTSTRAP).unwrap();
    assert_eq!(remainder.len(), 0, "{}", remainder);

    for statement in statements {
        // println!("{:#?}", statement);
        instance.execute_statement(statement).unwrap();
    }
}
