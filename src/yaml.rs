use crate::typ::UnresolvedType;

#[derive(Clone, Debug)]
pub struct YamlNode {
    pub label: String,
    pub value: YamlValue,
}

#[derive(Clone, Debug)]
pub enum YamlValue {
    Tree(Vec<YamlNode>),
    String(String),
    KeywordBase,
    Type(UnresolvedType),
    // Expression(UnresolvedExpression),
}

// Inserts the last item into the children of the second to last item.
fn collapse_yaml_parser_stack(stack: &mut Vec<(String, Vec<YamlNode>, usize)>) {
    assert!(stack.len() >= 2);
    let child = stack.pop().unwrap();
    let parent = stack.last_mut().unwrap();
    parent.1.push(YamlNode {
        label: child.0,
        value: YamlValue::Tree(child.1),
    });
}

fn parse_string(data: &mut impl Iterator<Item = char>) -> YamlValue {
    let mut string = "".to_owned();
    for ch in data {
        if ch == '"' {
            break;
        } else {
            string.push(ch);
        }
    }
    YamlValue::String(string)
}

fn parse_keyword(data: &mut impl Iterator<Item = char>, first_char: char) -> YamlValue {
    let mut keyword = first_char.to_string();
    let mut data = data.peekable();
    loop {
        let ch = if let Some(ch) = data.peek() {
            *ch
        } else {
            break;
        };
        if ch == ' ' || ch == '\n' {
            break;
        } else {
            keyword.push(ch);
            data.next();
        }
    }
    match &keyword[..] {
        "base" => YamlValue::KeywordBase,
        _ => panic!("Bad keyword"),
    }
}

fn parse_top_level(data: &mut impl Iterator<Item = char>) -> Vec<YamlNode> {
    let mut indent_chars_on_this_line = 0;
    // Stack used for parsing YAML tree. String is the label of the node in the tree, Vec<> is its
    // children, and usize is how indented its children should be.
    let mut stack: Vec<(String, Vec<YamlNode>, usize)> = vec![("ROOT".to_owned(), Vec::new(), 0)];
    enum State {
        LookingForLabel,
        ReadingLabel(String),
        LookingForValue(String),
        ReadingTreeStartIndentation(String),
    }
    let mut state = State::LookingForLabel;

    while let Some(ch) = data.next() {
        if ch == '\r' {
            continue;
        } else if ch == '\n' {
            indent_chars_on_this_line = 0;
            // If we were expecting a value and encounter a newline, that means the user is going
            // to specify children instead of a value.
            if let State::LookingForValue(label) = state {
                state = State::ReadingTreeStartIndentation(label);
            }
        } else if let State::LookingForLabel = state {
            if ch == ' ' || ch == '\t' {
                indent_chars_on_this_line += 1;
            } else {
                let mut expected_indentation = stack.last().unwrap().2;
                if indent_chars_on_this_line > expected_indentation {
                    panic!("Unexpected indentation.");
                }
                // Check for valid dedentation.
                while indent_chars_on_this_line < expected_indentation {
                    if stack.len() == 1 {
                        panic!("Invalid dedentation.");
                    }
                    collapse_yaml_parser_stack(&mut stack);
                    expected_indentation = stack.last().unwrap().2;
                }
                state = State::ReadingLabel(ch.to_string());
            }
        } else if let State::ReadingLabel(mut label_so_far) = state {
            if ch == ':' {
                state = State::LookingForValue(label_so_far);
            } else {
                label_so_far.push(ch);
                state = State::ReadingLabel(label_so_far);
            }
        } else if let State::LookingForValue(label) = state {
            if ch == ' ' || ch == '\t' {
                state = State::LookingForValue(label);
                continue;
            } else {
                let value = if ch == '$' {
                    unimplemented!("Expression");
                } else if ch == '<' {
                    YamlValue::Type(crate::typ::parse_typ(data))
                } else if ch == '"' {
                    parse_string(data)
                } else {
                    parse_keyword(data, ch)
                };
                stack.last_mut().unwrap().1.push(YamlNode { label, value });
                state = State::LookingForLabel;
            }
        } else if let State::ReadingTreeStartIndentation(label) = state {
            if ch == ' ' || ch == '\t' {
                indent_chars_on_this_line += 1;
                state = State::ReadingTreeStartIndentation(label);
            } else {
                if indent_chars_on_this_line <= stack.last().unwrap().2 {
                    panic!("Expected additional indentation.");
                }
                stack.push((label, Vec::new(), indent_chars_on_this_line));
                state = State::ReadingLabel(ch.to_string());
            }
        }
    }

    while stack.len() > 1 {
        collapse_yaml_parser_stack(&mut stack);
    }
    stack.pop().unwrap().1
}

pub fn parse(data: &str) -> Vec<YamlNode> {
    parse_top_level(&mut data.chars())
}
