//! File convention: a parameter `i` means "input to the parser", NOT "index to an array".
//! This is a general convention for Nom parsers.

use nom::{
    character::complete as character,
    combinator::{map, map_res},
    sequence::tuple,
};

pub type Input<'a> = &'a str;
pub type Result<'a, T> = nom::IResult<Input<'a>, T>;

/// Expressions can be evaluated (producing a value)
/// or bound to identifiers by assignments.
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
enum Expression {
    Number(u64),
}

impl Expression {
    fn parse(i: Input) -> Result<Self> {
        // Numbers are a sequence of digits and underscores.
        let allowed_chars = character::one_of("0123456789_");
        let number = nom::multi::many1(allowed_chars);
        map_res(number, |chars| {
            let digits_only = chars
                .into_iter()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>();
            digits_only.parse().map(Self::Number)
        })(i)
    }
}

/// Assigning a value to a binding, e.g. `n = 100`.
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
struct Assignment {
    identifier: String,
    value: Expression,
}

impl Assignment {
    fn parse(i: Input) -> Result<Self> {
        let parts = tuple((
            character::alpha1,
            nom::bytes::complete::tag(" = "),
            Expression::parse,
        ));
        let mut p = map(parts, |(identifier, _, value)| Self {
            identifier: identifier.to_string(),
            value,
        });
        p(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr_number() {
        let expected = Expression::Number(123);
        let i = "12_3";
        let (i, actual) = Expression::parse(i).unwrap();
        assert!(i.is_empty());
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_assignment() {
        let expected = Assignment {
            identifier: "n".to_string(),
            value: Expression::Number(100),
        };
        let i = "n = 100";
        let (i, actual) = Assignment::parse(i).unwrap();
        assert!(i.is_empty());
        assert_eq!(actual, expected);
    }
}
