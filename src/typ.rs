#[derive(Clone, Debug)]
pub struct UnresolvedUnitSpecifier {
    pub class: String,
    pub unit: Option<String>,
}

#[derive(Clone, Debug)]
pub enum UnresolvedType {
    Entity {
        template: Option<String>,
    },
    Scalar {
        unit: Option<UnresolvedUnitSpecifier>,
    },
}

pub(crate) fn parse_typ(data: &mut impl Iterator<Item = char>) -> UnresolvedType {
    enum State {
        ReadingFirstPart(String),
        ReadingEntityTemplate(String),
        ReadingScalarUnitClass(String),
        ReadingScalarUnit(String, String),
    }
    let mut state = State::ReadingFirstPart("".to_owned());
    for ch in data {
        if let State::ReadingFirstPart(first_part_so_far) = &mut state {
            if ch == ':' {
                if first_part_so_far == "Entity" {
                    state = State::ReadingEntityTemplate("".to_owned());
                } else if first_part_so_far == "Scalar" {
                    state = State::ReadingScalarUnitClass("".to_owned());
                } else {
                    panic!("{} is not a valid type", first_part_so_far)
                }
            } else if ch == '>' {
                if first_part_so_far == "Entity" {
                    return UnresolvedType::Entity { template: None };
                } else if first_part_so_far == "Scalar" {
                    return UnresolvedType::Scalar { unit: None };
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
        } else if let State::ReadingScalarUnitClass(mut class_so_far) = state {
            if ch == '>' {
                return UnresolvedType::Scalar {
                    unit: Some(UnresolvedUnitSpecifier {
                        class: class_so_far,
                        unit: None,
                    }),
                };
            } else if ch == ':' {
                state = State::ReadingScalarUnit(class_so_far, "".to_owned());
            } else {
                class_so_far.push(ch);
                state = State::ReadingScalarUnitClass(class_so_far);
            }
        } else if let State::ReadingScalarUnit(class, mut unit_so_far) = state {
            if ch == '>' {
                return UnresolvedType::Scalar {
                    unit: Some(UnresolvedUnitSpecifier {
                        class,
                        unit: Some(unit_so_far),
                    })
                };
            } else {
                unit_so_far.push(ch);
                state = State::ReadingScalarUnit(class, unit_so_far);
            }
        } else {
            unreachable!();
        }
    }
    panic!("Unexpected EOF while parsing type.");
}
