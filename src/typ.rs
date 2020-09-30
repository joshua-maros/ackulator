#[derive(Clone, Debug)]
pub struct UnresolvedUnitComponent {
    pub class: String,
    pub unit: Option<String>,
    pub exponent: i32,
}

#[derive(Clone, Debug)]
pub enum UnresolvedType {
    Entity {
        template: Option<String>,
    },
    Scalar {
        units: Option<Vec<UnresolvedUnitComponent>>,
    },
}

// Parses unit specifiers like Length:Meters ^ 2 * Mass / Time ^ 2
pub(crate) fn parse_unit_specifier(
    data: &mut impl Iterator<Item = char>,
) -> Vec<UnresolvedUnitComponent> {
    #[derive(PartialEq)]
    enum State {
        LookingForUnitClass,
        ReadingUnitClass,
        ReadingUnit,
        LookingForExponentOrCombinator(bool),
        ReadingNumber,
    }
    let mut current_class = "".to_owned();
    let mut current_unit = "".to_owned();
    let mut number_buffer = "".to_owned();
    let mut invert = false;
    let mut state = State::ReadingUnitClass;
    let mut components = Vec::new();
    let mut data = data.peekable();
    let mut first = true;
    while let Some(ch) = data.peek() {
        let ch = *ch;
        if first {
            first = false;
        } else {
            data.next();
        }
        if let State::LookingForUnitClass = state {
            if ch.is_alphabetic() || ch.is_numeric() {
                state = State::ReadingUnitClass;
            } else {
                continue;
            }
        }
        if let State::ReadingUnitClass = state {
            if ch == ':' {
                current_class = current_class.trim().to_owned();
                state = State::ReadingUnit;
                continue;
            } else if ch.is_alphabetic() || ch.is_numeric() || ch.is_whitespace() {
                current_class.push(ch);
                continue;
            } else {
                state = State::LookingForExponentOrCombinator(true);
                // Do not continue so we can check if the character we are looking at is already an
                // exponent ^ or a combinator * /
            }
        }
        if let State::ReadingUnit = state {
            if ch.is_alphabetic() || ch.is_numeric() || ch.is_whitespace() {
                current_unit.push(ch);
                continue;
            } else {
                current_unit = current_unit.trim().to_owned();
                state = State::LookingForExponentOrCombinator(true);
            }
        }
        if let State::ReadingNumber = state {
            if ch.is_whitespace() || ch.is_numeric() || ch == '.' || ch == '-' {
                number_buffer.push(ch);
                continue;
            } else {
                state = State::LookingForExponentOrCombinator(false);
            }
        }
        if let State::LookingForExponentOrCombinator(exponent_ok) = state {
            if ch.is_whitespace() {
                continue;
            } else if ch == '*' || ch == '/' {
                components.push(UnresolvedUnitComponent {
                    class: current_class,
                    unit: if current_unit.len() > 0 {
                        Some(current_unit)
                    } else {
                        None
                    },
                    exponent: if invert { -1 } else { 1 }
                        * number_buffer.trim().parse().ok().unwrap_or(1),
                });
                current_class = "".to_owned();
                current_unit = "".to_owned();
                number_buffer = "".to_owned();
                state = State::LookingForUnitClass;
                if ch == '/' {
                    invert = !invert;
                }
                continue;
            } else if ch == '^' {
                if !exponent_ok {
                    panic!("Duplicate exponent in unit");
                }
                state = State::ReadingNumber;
                continue;
            } else {
                break;
            }
        }
    }
    if let State::ReadingUnitClass = state {
        components.push(UnresolvedUnitComponent {
            class: current_class.trim().to_owned(),
            unit: None,
            exponent: 1,
        });
    } else if let State::ReadingUnit = state {
        components.push(UnresolvedUnitComponent {
            class: current_class,
            unit: Some(current_unit.trim().to_owned()),
            exponent: 1,
        });
    } else if let State::LookingForExponentOrCombinator(..) = state {
        components.push(UnresolvedUnitComponent {
            class: current_class,
            unit: if current_unit.len() > 0 {
                Some(current_unit)
            } else {
                None
            },
            exponent: if invert { -1 } else { 1 } * number_buffer.trim().parse().ok().unwrap_or(1),
        });
    }
    components
}

pub(crate) fn parse_typ(data: &mut impl Iterator<Item = char>) -> UnresolvedType {
    enum State {
        ReadingFirstPart(String),
        ReadingEntityTemplate(String),
    }
    let mut state = State::ReadingFirstPart("".to_owned());
    while let Some(ch) = data.next() {
        if let State::ReadingFirstPart(first_part_so_far) = &mut state {
            if ch == '~' {
                let first_part = first_part_so_far.trim();
                if first_part == "Entity" {
                    state = State::ReadingEntityTemplate("".to_owned());
                } else if first_part == "Scalar" {
                    return UnresolvedType::Scalar {
                        units: Some(parse_unit_specifier(data)),
                    };
                } else {
                    panic!("{} is not a valid type", first_part_so_far)
                }
            } else if ch == '>' {
                if first_part_so_far == "Entity" {
                    return UnresolvedType::Entity { template: None };
                } else if first_part_so_far == "Scalar" {
                    return UnresolvedType::Scalar { units: None };
                } else {
                    panic!("{} is not a valid type", first_part_so_far)
                }
            } else {
                first_part_so_far.push(ch);
            }
        } else if let State::ReadingEntityTemplate(mut template_so_far) = state {
            if ch == '>' {
                return UnresolvedType::Entity {
                    template: Some(template_so_far),
                };
            } else {
                template_so_far.push(ch);
                state = State::ReadingEntityTemplate(template_so_far);
            }
        }
    }
    panic!("Unexpected EOF while parsing type.");
}
