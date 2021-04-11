use nom::IResult;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Negate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    NumericLiteral(f64),
    LookupName(String),
    UnaryExpr(UnaryOp, Box<Expression>),
    BinaryExpr(Box<Expression>, BinaryOp, Box<Expression>),
    ApplyFunction {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
}

mod parsing {
    use std::f64::NAN;

    use super::*;
    use nom::{
        branch::alt,
        bytes::complete::{take_while, take_while1},
        character::complete::{char, one_of},
        combinator::{not, opt},
        error::make_error,
        multi::{fold_many0, many0, many1},
        sequence::{delimited, pair, tuple},
    };

    fn whitespace(input: &str) -> IResult<&str, ()> {
        let (input, _) = take_while(char::is_whitespace)(input)?;
        Ok((input, ()))
    }

    fn collect_digits(input: &str) -> IResult<&str, String> {
        let (input, chars) = many1(one_of("0123456789"))(input)?;
        Ok((input, chars.into_iter().collect()))
    }

    fn numeric_literal(input: &str) -> IResult<&str, Expression> {
        let mut problem = false;

        let (input, sign) = opt(one_of("+-"))(input)?;
        let negative = sign == Some('-');

        let (input, integer_part) = opt(collect_digits)(input)?;

        let (input, fractional_part) = opt(pair(char('.'), collect_digits))(input)?;

        if integer_part.is_none() && fractional_part.is_none() {
            Err(nom::Err::Error(make_error(
                input,
                nom::error::ErrorKind::Digit,
            )))?;
        }

        let (input, exponent) = opt(tuple((char('e'), opt(one_of("+-")), collect_digits)))(input)?;
        let exponent = exponent.map(|(_e, sign, digits)| {
            let negative = sign == Some('-');
            let amount = digits.parse::<i32>();
            problem = problem || amount.is_err();
            let res = amount.unwrap_or_default();
            if negative {
                -res
            } else {
                res
            }
        });

        let mut result = 0f64;
        if let Some(part) = integer_part {
            result += part
                .parse::<f64>()
                .map_err(|_| problem = true)
                .unwrap_or_default();
        }
        if let Some((_dot, part)) = fractional_part {
            result += format!(".{}", part)
                .parse::<f64>()
                .map_err(|_| problem = true)
                .unwrap_or_default();
        }
        if let Some(power) = exponent {
            result *= 10f64.powi(power);
        }
        if negative {
            result = -result;
        }
        if problem {
            Ok((input, Expression::NumericLiteral(NAN)))
        } else {
            Ok((input, Expression::NumericLiteral(result)))
        }
    }

    fn lookup_name(input: &str) -> IResult<&str, Expression> {
        let (input, _) = not(collect_digits)(input)?;
        let (input, name) = take_while1(char::is_alphanumeric)(input)?;
        Ok((input, Expression::LookupName(name.to_owned())))
    }

    /// This should always be called with delimited(whitespace, this, whitespace) because it is a
    /// consistent and efficent position to handle that.
    fn expr_priority50(input: &str) -> IResult<&str, Expression> {
        alt((
            numeric_literal,
            lookup_name,
            delimited(char('('), expr_priority10, char(')')),
        ))(input)
    }

    fn fn_args(input: &str) -> IResult<&str, Vec<Expression>> {
        if let Ok((input, first_arg)) = expr_priority10(input) {
            let mut args = vec![first_arg];
            let (input, others) = many0(pair(char(','), expr_priority10))(input)?;
            for (comma, arg) in others {
                debug_assert_eq!(comma, ',');
                args.push(arg);
            }
            // Trailing comma
            if let Ok((input, _)) = char::<_, nom::error::Error<_>>(',')(input) {
                Ok((input, args))
            } else {
                Ok((input, args))
            }
        } else {
            Ok((input, Vec::new()))
        }
    }

    fn expr_priority40(input: &str) -> IResult<&str, Expression> {
        let (input, term) = delimited(whitespace, expr_priority50, whitespace)(input)?;
        if let Ok((input, arguments)) = delimited(char('('), fn_args, char(')'))(input) {
            Ok((
                input,
                Expression::ApplyFunction {
                    function: Box::new(term),
                    arguments,
                },
            ))
        } else {
            Ok((input, term))
        }
    }

    fn expr_priority30(input: &str) -> IResult<&str, Expression> {
        let (input, first_term) = expr_priority40(input)?;
        let terms = vec![first_term];
        let (input, mut terms) = fold_many0(
            pair(char('^'), expr_priority40),
            terms,
            |mut list, (op, rhs): (char, Expression)| {
                debug_assert_eq!(op, '^');
                list.push(rhs);
                list
            },
        )(input)?;
        let mut expr = terms.pop().unwrap();
        for lhs in terms.into_iter().rev() {
            expr = Expression::BinaryExpr(Box::new(lhs), BinaryOp::Pow, Box::new(expr));
        }
        Ok((input, expr))
    }

    fn expr_priority20(input: &str) -> IResult<&str, Expression> {
        let (input, first_term) = expr_priority30(input)?;
        fold_many0(
            pair(one_of("*/"), expr_priority30),
            first_term,
            |lhs, (op, rhs): (char, Expression)| {
                let op = match op {
                    '*' => BinaryOp::Mul,
                    '/' => BinaryOp::Div,
                    _ => unreachable!(),
                };
                Expression::BinaryExpr(Box::new(lhs), op, Box::new(rhs))
            },
        )(input)
    }

    fn expr_priority10(input: &str) -> IResult<&str, Expression> {
        let (input, first_term) = expr_priority20(input)?;
        fold_many0(
            pair(one_of("+-"), expr_priority20),
            first_term,
            |lhs, (op, rhs): (char, Expression)| {
                let op = match op {
                    '+' => BinaryOp::Add,
                    '-' => BinaryOp::Sub,
                    _ => unreachable!(),
                };
                Expression::BinaryExpr(Box::new(lhs), op, Box::new(rhs))
            },
        )(input)
    }

    pub fn parse_expression(input: &str) -> IResult<&str, Expression> {
        expr_priority10(input)
    }
}

pub use parsing::parse_expression;