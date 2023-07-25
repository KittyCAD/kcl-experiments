//! Parse KCL source into its AST.
// File convention: a parameter `i` means "input to the parser", NOT "index to an array".
// This is a general convention for Nom parsers.

use crate::ast::*;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{self as character, char as one_char},
    combinator::{map, map_res},
    error::{context, VerboseError},
    multi::{many1, separated_list0},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
};

pub type Input<'a> = &'a str;
pub type Result<'a, T> = nom::IResult<Input<'a>, T, VerboseError<Input<'a>>>;

/// These can't be used as names in KCL programs.
const RESERVED_KEYWORDS: [&str; 2] = ["let", "in"];

/// Take KCL source code as input and parse it into an AST node.
pub trait Parser: Sized {
    fn parse(i: Input) -> Result<Self>;
}

impl Parser for Identifier {
    fn parse(i: Input) -> Result<Self> {
        // Checks if the ID is in the reserved keyword list.
        let is_reserved_keyword = |id: Identifier| {
            let is_reserved = RESERVED_KEYWORDS
                .iter()
                .any(|reserved_kw| reserved_kw == &&id.0);
            if is_reserved {
                Err(format!("{id} is a reserved keyword and cannot be used as the name of a function, binding, type etc"))
            } else {
                Ok(id)
            }
        };

        let parser = map_res(Self::parse_maybe_reserved, is_reserved_keyword);
        context("identifier", parser)(i)
    }
}

impl Identifier {
    /// Like `Identifier::parse` except it doesn't check if the identifier is a reserved keyword.
    fn parse_maybe_reserved(i: Input) -> Result<Self> {
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
    /// FnDef looks like
    ///     myCircle = (radius: Distance -> Solid2D) => circle(radius)
    fn parse(i: Input) -> Result<Self> {
        // Parse the parts of a function definition.
        let parse_parts = tuple((
            Identifier::parse,
            tag(" = "),
            bracketed(tuple((
                separated_list0(tag(", "), Parameter::parse),
                tag(" -> "),
                Identifier::parse,
            ))),
            terminated(tag(" =>"), character::multispace0),
            Expression::parse,
        ));

        // Convert the parts we actually need into a FnDef, ignoring the parts we don't need.
        let parser = map(
            parse_parts,
            |(fn_name, _, (params, _, return_type), _, body)| Self {
                fn_name,
                params,
                return_type,
                body,
            },
        );
        context("function definition", parser)(i)
    }
}

impl Parser for Parameter {
    fn parse(i: Input) -> Result<Self> {
        // Looks like `radius: Distance`
        let parser = map(
            separated_pair(Identifier::parse, tag(": "), Identifier::parse),
            |(name, kcl_type)| Self { name, kcl_type },
        );
        context("parameter", parser)(i)
    }
}

impl Parser for FnInvocation {
    fn parse(i: Input) -> Result<Self> {
        let parse_parts = tuple((
            Identifier::parse,
            one_char('('),
            separated_list0(tag(", "), Expression::parse),
            one_char(')'),
        ));
        let parser = map(parse_parts, |(fn_name, _, args, _)| FnInvocation {
            fn_name,
            args,
        });
        context("function invocation", parser)(i)
    }
}

impl Parser for Expression {
    fn parse(i: Input) -> Result<Self> {
        let parser = alt((
            Self::parse_arithmetic,
            Self::parse_num,
            Self::parse_let_in,
            map(FnInvocation::parse, Self::FnInvocation),
            map(Identifier::parse, Self::Name),
        ));
        context("expression", parser)(i)
    }
}

impl Parser for Operator {
    fn parse(i: Input) -> Result<Self> {
        let parser = map_res(
            alt((one_char('+'), one_char('-'), one_char('*'), one_char('/'))),
            |symbol| {
                Ok(match dbg!(symbol) {
                    '+' => Self::Add,
                    '-' => Self::Sub,
                    '*' => Self::Mul,
                    '/' => Self::Div,
                    other => return Err(format!("Invalid operator {other}")),
                })
            },
        );
        context("operator", parser)(i)
    }
}

impl Expression {
    fn parse_arithmetic(i: Input) -> Result<Self> {
        let parser = map(
            bracketed(tuple((
                Self::parse,
                delimited(one_char(' '), Operator::parse, one_char(' ')),
                Self::parse,
            ))),
            |(lhs, op, rhs)| Self::Arithmetic {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            },
        );
        context("arithmetic", parser)(i)
    }

    fn parse_let_in(i: Input) -> Result<Self> {
        let parser = map(
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
        );
        context("let-in", parser)(i)
    }

    fn parse_num(i: Input) -> Result<Self> {
        // Numbers are a sequence of digits and underscores.
        let allowed_chars = character::one_of("0123456789_");
        let number = nom::multi::many1(allowed_chars);
        let parser = map_res(number, |chars| {
            let digits_only = chars
                .into_iter()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>();
            digits_only.parse().map(Self::Number)
        });
        context("number", parser)(i)
    }
}

impl Parser for Assignment {
    fn parse(i: Input) -> Result<Self> {
        let parts = tuple((
            Identifier::parse,
            nom::bytes::complete::tag(" = "),
            Expression::parse,
        ));
        let parser = map(parts, |(identifier, _, value)| Self { identifier, value });
        context("assignment", parser)(i)
    }
}

/// Utility parser combinator.
/// Parses a '(', then the child parser, then a ')'.
pub fn bracketed<I, O2, E: nom::error::ParseError<I>, G>(
    mut child_parser: G,
) -> impl FnMut(I) -> nom::IResult<I, O2, E>
where
    I: nom::Slice<std::ops::RangeFrom<usize>> + nom::InputIter,
    G: nom::Parser<I, O2, E>,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    use nom::Parser;
    move |input: I| {
        let (input, _) = one_char('(').parse(input)?;
        let (input, o2) = child_parser.parse(input)?;
        one_char(')').parse(input).map(|(i, _)| (i, o2))
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
            let (i, actual) = match T::parse(&s) {
                Ok(t) => t,
                Err(e) => {
                    panic!("Could not parse: {e:#?}");
                }
            };
            assert!(
                i.is_empty(),
                "Input should have been empty, but '{i}' remained. Parsed {actual:#?}."
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
            (Expression::Number(1), "1"),
            (Expression::Number(2), "2"),
        ]);
    }

    #[test]
    fn test_expr_arith() {
        assert_parse(vec![
            (
                Expression::Arithmetic {
                    lhs: Box::new(Expression::Number(1)),
                    op: Operator::Add,
                    rhs: Box::new(Expression::Number(2)),
                },
                "(1 + 2)",
            ),
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
                            value: Expression::Arithmetic {
                                lhs: Box::new(Expression::Number(14)),
                                op: Operator::Mul,
                                rhs: Box::new(Expression::Number(100)),
                            },
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
                    radius = (14 * 100)
                in circle(radius, center)",
            ),
        ])
    }

    #[test]
    fn let_in() {
        assert_parse(vec![
            (
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
            ),
            (
                Expression::LetIn {
                    r#let: vec![Assignment {
                        identifier: Identifier("radius".into()),
                        value: Expression::Arithmetic {
                            lhs: Box::new(Expression::Number(14)),
                            op: Operator::Mul,
                            rhs: Box::new(Expression::Number(100)),
                        },
                    }],
                    r#in: Box::new(Expression::FnInvocation(FnInvocation {
                        fn_name: Identifier("circle".into()),
                        args: vec![
                            Expression::Name(Identifier("radius".into())),
                            Expression::Name(Identifier("center".into())),
                        ],
                    })),
                },
                r#"let
            radius = (14 * 100)
        in circle(radius, center)"#,
            ),
        ])
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
            "let",
            "in",
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
