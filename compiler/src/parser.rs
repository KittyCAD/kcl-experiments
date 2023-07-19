//! Parse KCL source into its AST.
// File convention: a parameter `i` means "input to the parser", NOT "index to an array".
// This is a general convention for Nom parsers.

use crate::ast::*;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{self as character, char as one_char},
    combinator::{map, map_res},
    multi::{many1, separated_list0},
    sequence::{preceded, separated_pair, terminated, tuple},
};

pub type Input<'a> = &'a str;
pub type Result<'a, T> = nom::IResult<Input<'a>, T>;

/// Take KCL source code as input and parse it into an AST node.
pub trait Parser: Sized {
    fn parse(i: Input) -> Result<Self>;
}

impl Parser for Identifier {
    fn parse(i: Input) -> Result<Self> {
        // Identifiers cannot start with a number
        let (i, start) = nom_unicode::complete::alpha1(i)?;
        // But after the first char, they can include numbers.
        let (i, end) = nom_unicode::complete::alphanumeric0(i)?;
        // TODO: This shouldn't need to allocate a string, I should be able to return &str here, but
        // the compiler doesn't know that `start` and `end` are contiguous slices. I know there's a
        // way to achieve this, but I don't know what it is yet. Nor is it important enough to worry
        // about yet.
        let mut identifier = String::with_capacity(start.len() + end.len());
        identifier.push_str(start);
        identifier.push_str(end);
        Ok((i, Self(identifier)))
    }
}

impl Parser for FnDef {
    fn parse(i: Input) -> Result<Self> {
        // Looks like myCircle = (radius: Distance -> Solid2D) => circle(radius)
        let parts = tuple((
            Identifier::parse,
            tag(" = "),
            tuple((
                one_char('('),
                separated_list0(tag(", "), Parameter::parse),
                tag(" -> "),
                Identifier::parse,
                one_char(')'),
            )),
            terminated(tag(" =>"), character::multispace0),
            Expression::parse,
        ));
        map(
            parts,
            |(fn_name, _, (_paren_start, params, _, return_type, _paren_end), _, body)| Self {
                fn_name,
                params,
                return_type,
                body,
            },
        )(i)
    }
}

impl Parser for Parameter {
    fn parse(i: Input) -> Result<Self> {
        // Looks like `radius: Distance`
        map(
            separated_pair(Identifier::parse, tag(": "), Identifier::parse),
            |(name, kcl_type)| Self { name, kcl_type },
        )(i)
    }
}

impl Parser for FnInvocation {
    fn parse(i: Input) -> Result<Self> {
        let parts = tuple((
            Identifier::parse,
            one_char('('),
            separated_list0(tag(", "), Expression::parse),
            one_char(')'),
        ));
        map(parts, |(fn_name, _, args, _)| FnInvocation {
            fn_name,
            args,
        })(i)
    }
}

impl Parser for Expression {
    fn parse(i: Input) -> Result<Self> {
        alt((
            Self::parse_let_in,
            Self::parse_num,
            Self::parse_fn_invocation,
            map(Identifier::parse, Self::Name),
        ))(i)
    }
}

impl Expression {
    fn parse_fn_invocation(i: Input) -> Result<Self> {
        map(FnInvocation::parse, Self::FnInvocation)(i)
    }

    fn parse_let_in(i: Input) -> Result<Self> {
        map(
            tuple((
                tag("let"),
                character::newline,
                many1(tuple((
                    character::multispace0,
                    Assignment::parse,
                    character::newline,
                ))),
                terminated(
                    preceded(character::multispace0, tag("in")),
                    character::multispace0,
                ),
                Expression::parse,
            )),
            |(_, _, assignments, _, expr)| Self::LetIn {
                r#let: assignments
                    .into_iter()
                    .map(|(_, assign, _)| assign)
                    .collect(),
                r#in: Box::new(expr),
            },
        )(i)
    }

    fn parse_num(i: Input) -> Result<Self> {
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

impl Parser for Assignment {
    fn parse(i: Input) -> Result<Self> {
        let parts = tuple((
            Identifier::parse,
            nom::bytes::complete::tag(" = "),
            Expression::parse,
        ));
        let mut p = map(parts, |(identifier, _, value)| Self { identifier, value });
        p(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Assert that the given input successfully parses into the expected value.
    fn assert_parse<T, I>(tests: Vec<(T, I)>)
    where
        T: Parser + std::fmt::Debug + PartialEq,
        I: Into<String>,
    {
        for (test_id, (expected, i)) in tests.into_iter().enumerate() {
            let s = i.into();
            let (i, actual) = T::parse(&s).unwrap();
            assert!(
                i.is_empty(),
                "Input should have been empty, but '{i}' remained"
            );
            assert_eq!(
                actual, expected,
                "failed test {test_id}: expected {expected:#?} and got {actual:#?}"
            );
        }
    }

    /// Assert that the given input fails to parse.
    fn assert_not_parse<T>(i: Input)
    where
        T: Parser + std::fmt::Debug + PartialEq,
    {
        let _err = T::parse(i).unwrap_err();
    }

    #[test]
    fn test_expr_number() {
        assert_parse(vec![
            (Expression::Number(123), "12_3"),
            (Expression::Number(123), "123"),
        ]);
    }

    #[test]
    fn valid_function_invocations() {
        assert_parse(vec![(
            Expression::FnInvocation(FnInvocation {
                fn_name: "sphere".into(),
                args: vec![Expression::Number(1), Expression::Number(2)],
            }),
            "sphere(1, 2)",
        )])
    }

    #[test]
    fn valid_function_definition() {
        assert_parse(vec![
            (
                FnDef {
                    fn_name: "bigCircle".into(),
                    params: vec![
                        Parameter {
                            name: "radius".into(),
                            kcl_type: "Distance".into(),
                        },
                        Parameter {
                            name: "center".into(),
                            kcl_type: "Point2D".into(),
                        },
                    ],
                    body: Expression::FnInvocation(FnInvocation {
                        fn_name: "circle".into(),
                        args: vec![Expression::Name("radius".into())],
                    }),
                    return_type: "Solid2D".into(),
                },
                r#"bigCircle = (radius: Distance, center: Point2D -> Solid2D) => circle(radius)"#,
            ),
            (
                FnDef {
                    fn_name: "bigCircle".into(),
                    params: vec![Parameter {
                        name: "center".into(),
                        kcl_type: "Point2D".into(),
                    }],
                    body: Expression::LetIn {
                        r#let: vec![Assignment {
                            identifier: "radius".into(),
                            value: Expression::Number(14),
                        }],
                        r#in: Box::new(Expression::FnInvocation(FnInvocation {
                            fn_name: "circle".into(),
                            args: vec![
                                Expression::Name("radius".into()),
                                Expression::Name("center".into()),
                            ],
                        })),
                    },
                    return_type: "Solid2D".into(),
                },
                "\
bigCircle = (center: Point2D -> Solid2D) =>
    let
        radius = 14
    in circle(radius, center)",
            ),
        ])
    }

    #[test]
    fn let_in() {
        assert_parse(vec![(
            Expression::LetIn {
                r#let: vec![
                    Assignment {
                        identifier: "x".into(),
                        value: Expression::Number(1),
                    },
                    Assignment {
                        identifier: "y".into(),
                        value: Expression::Name("x".into()),
                    },
                ],
                r#in: Box::new(Expression::Name("y".into())),
            },
            r#"let
    x = 1
    y = x
in y"#,
        )])
    }

    #[test]
    fn test_assignment() {
        let valid_lhs = ["亞當", "n", "n123"];
        let tests = valid_lhs
            .into_iter()
            .flat_map(|lhs| {
                vec![
                    (
                        Assignment {
                            identifier: lhs.into(),
                            value: Expression::FnInvocation(FnInvocation {
                                fn_name: "foo".into(),
                                args: vec![Expression::Number(100)],
                            }),
                        },
                        format!("{lhs} = foo(100)"),
                    ),
                    (
                        Assignment {
                            identifier: lhs.into(),
                            value: Expression::Number(100),
                        },
                        format!("{lhs} = 100"),
                    ),
                ]
            })
            .collect();
        assert_parse(tests)
    }

    #[test]
    fn test_invalid_variables() {
        let invalid_binding_names = [
            // These are genuinely invalid.
            "n(",
            "123",
            "n h",
            "0000000aassdfasdfasdfasdf013423452342134234234234",
            // TODO: fix this, it should be valid.
            "n_hello",
        ];
        for identifier in invalid_binding_names {
            let i = format!("{identifier} = 100");
            assert_not_parse::<Assignment>(&i)
        }
    }
}
