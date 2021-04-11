use crate::expression::Expression;

#[derive(Clone, Debug)]
pub enum Statement {
    MakeUnitClass(Vec<String>),
    MakeBaseUnit(Vec<String>, Expression),
    MakeDerivedUnit(Vec<String>, Expression),
    MakeEntityClass(Vec<String>, Expression),
    MakeLabel(Vec<String>, Expression),
    MakeValue(Vec<String>, Expression),

    Show(Expression),
}

mod parse {
    use crate::expression;

    use super::*;
    use nom::{
        branch::alt,
        bytes::complete::{tag, take, take_while, take_while1},
        character::complete::{char, one_of},
        combinator::{not, opt, peek},
        error::{make_error, ErrorKind},
        multi::separated_list1,
        sequence::delimited,
        IResult,
    };
    use Statement::*;

    type PlainError<'i> = nom::error::Error<&'i str>;

    fn whitespace(mut input: &str) -> IResult<&str, ()> {
        let mut comment = false;
        loop {
            if let Ok((ni, _)) = one_of::<_, _, PlainError>(" \t\r")(input) {
                input = ni;
            } else if let Ok((ni, _)) = tag::<_, _, PlainError>("//")(input) {
                input = ni;
                comment = true;
            } else if let Ok((ni, _)) = tag::<_, _, PlainError>("\n")(input) {
                input = ni;
                comment = false;
            } else if comment && input.len() > 0 {
                input = take(1usize)(input)?.0;
            } else {
                break;
            }
        }
        Ok((input, ()))
    }

    fn identifier(input: &str) -> IResult<&str, String> {
        let (input, _) = not(one_of("0123456789"))(input)?;
        let (input, value) = take_while1(char::is_alphanumeric)(input)?;
        Ok((input, value.to_owned()))
    }

    fn parse_make(input: &str) -> IResult<&str, Statement> {
        let (input, _) = tag("make")(input)?;
        let (input, _) = whitespace(input)?;
        let (input, label) = alt((
            tag("unit_class"),
            tag("base_unit"),
            tag("derived_unit"),
            tag("entity_class"),
            tag("label"),
            tag("value"),
        ))(input)?;
        let (input, _) = whitespace(input)?;
        let (input, _) = tag("called")(input)?;
        let (input, names) =
            separated_list1(char(','), delimited(whitespace, identifier, whitespace))(input)?;
        let (input, value) = if label == "unit_class" {
            (input, None)
        } else {
            opt(delimited(
                whitespace,
                crate::expression::parse_expression,
                whitespace,
            ))(input)?
        };
        macro_rules! ret_error {
            () => {
                return Err(nom::Err::Error(make_error(input, ErrorKind::Alt)))
            };
        }
        Ok((
            input,
            match (label, value) {
                ("unit_class", _) => MakeUnitClass(names),
                ("base_unit", Some(value)) => MakeBaseUnit(names, value),
                ("base_unit", None) => ret_error!(),
                ("derived_unit", Some(value)) => MakeDerivedUnit(names, value),
                ("derived_unit", None) => ret_error!(),
                ("entity_class", value) => MakeEntityClass(
                    names,
                    value.unwrap_or_else(|| Expression::BuildEntity {
                        properties: Vec::new(),
                        class_names: Vec::new(),
                    }),
                ),
                ("label", Some(value)) => MakeLabel(names, value),
                ("label", None) => ret_error!(),
                ("value", Some(value)) => MakeValue(names, value),
                ("value", None) => ret_error!(),
                _ => unreachable!()
            },
        ))
    }

    fn parse_show(input: &str) -> IResult<&str, Statement> {
        let (input, _) = tag("show")(input)?;
        let (input, _) = whitespace(input)?;
        let (input, value) = expression::parse_expression(input)?;
        Ok((input, Show(value)))
    }

    pub fn parse_statement(input: &str) -> IResult<&str, Statement> {
        let (input, result) =
            delimited(whitespace, alt((parse_make, parse_show)), whitespace)(input)?;
        Ok((input, result))
    }
}

pub use parse::parse_statement;
