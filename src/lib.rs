const TEST_YAML: &'static str = r##"
test:
    test2: <Scalar:Length:Meters>
"##;

mod typ;
mod yaml;

pub fn do_stuff() {
    let yaml = yaml::parse(TEST_YAML);
    println!("{:#?}", yaml);
}
