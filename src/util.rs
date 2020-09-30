pub fn to_title_case(text: &str) -> String {
    let mut capitalize = true;
    let mut result = String::with_capacity(text.as_bytes().len());
    for ch in text.chars() {
        if capitalize {
            capitalize = false;
            for ch2 in ch.to_uppercase() {
                result.push(ch2);
            }
        } else {
            for ch2 in ch.to_lowercase() {
                result.push(ch2);
            }
            if ch.is_whitespace() {
                capitalize = true;
            }
        }
    }
    result
}
