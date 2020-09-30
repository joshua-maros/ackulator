const TEST_YAML: &'static str = r##"
test:
    test2:
        test3: "value"
"##;

mod yaml;

pub fn do_stuff() {
    let yaml = yaml::parse(TEST_YAML);
    println!("{:#?}", yaml);
}
