//! Implements the Parser trait for all the AST types.
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{self as character, char as one_char},
    combinator::{all_consuming, map, map_res, recognize},
    error::context,
    multi::{many0, many1, separated_list0},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
};

use crate::{ast::*, parser::*};

/// These can't be used as names in KCL programs.
const RESERVED_KEYWORDS: [&str; 2] = ["let", "in"];

impl<'i> Parser<'i> for AbstractSyntaxTree<'i> {
    fn parse(i: Input<'i>) -> Result<Self> {
        let parser = all_consuming(many0(FnDef::parse));
        context(
            "program root",
            map(parser, |functions| AbstractSyntaxTree { functions }),
        )(i)
    }
}

impl<'i> Parser<'i> for Identifier<'i> {
    fn parse(i: Input<'i>) -> Result<'i, Self> {
        // Checks if the ID is in the reserved keyword list.
        let is_reserved_keyword = |id: Identifier<'i>| {
            let is_reserved = RESERVED_KEYWORDS
                .iter()
                .any(|reserved_kw| reserved_kw == id.0.fragment());
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

impl<'i> Identifier<'i> {
    /// Like `Identifier::parse` except it doesn't check if the identifier is a reserved keyword.
    fn parse_maybe_reserved(i: Input<'i>) -> Result<Self> {
        let parser = preceded(
            // Identifiers cannot start with a number
            nom_unicode::complete::alpha1,
            // But after the first char, they can include numbers.
            nom_unicode::complete::alphanumeric0,
        );
        map(recognize(parser), Self)(i)
    }
}

impl<'i> Parser<'i> for FnDef<'i> {
    /// FnDef looks like
    ///     myCircle = (radius: Distance -> Solid2D) => circle(radius)
    fn parse(i: Input<'i>) -> Result<Self> {
        // Parse the parts of a function definition.
        let parse_parts = tuple((
            context("function name", Identifier::parse),
            context("= between function name and definition", tag(" = ")),
            context(
                "type signature",
                bracketed(tuple((
                    context(
                        "parameter list",
                        separated_list0(tag(", "), Parameter::parse),
                    ),
                    context("return type arrow ->", tag(" -> ")),
                    Identifier::parse,
                ))),
            ),
            context(
                "=> between function header and body",
                terminated(tag(" =>"), character::multispace0),
            ),
            context("function body", Expression::parse),
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

impl<'i> Parser<'i> for Parameter<'i> {
    fn parse(i: Input<'i>) -> Result<Self> {
        // Looks like `radius: Distance`
        let parser = map(
            separated_pair(Identifier::parse, tag(": "), Identifier::parse),
            |(name, kcl_type)| Self { name, kcl_type },
        );
        context("parameter", parser)(i)
    }
}

impl<'i> Parser<'i> for FnInvocation<'i> {
    fn parse(i: Input<'i>) -> Result<Self> {
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

impl<'i> Parser<'i> for Expression<'i> {
    fn parse(i: Input<'i>) -> Result<Self> {
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

impl<'i> Parser<'i> for Operator {
    fn parse(i: Input<'i>) -> Result<Self> {
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

impl<'i> Expression<'i> {
    fn parse_arithmetic(i: Input<'i>) -> Result<Self> {
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

    fn parse_let_in(i: Input<'i>) -> Result<Self> {
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

    fn parse_num(i: Input<'i>) -> Result<Self> {
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

impl<'i> Parser<'i> for Assignment<'i> {
    fn parse(i: Input<'i>) -> Result<Self> {
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

    use tabled::Table;

    use super::*;
    use crate::displayable_error::DisplayableError;

    /// Assert that the given input successfully parses into the expected value.
    fn assert_parse<'i, T>(tests: Vec<(T, Input<'i>)>)
    where
        T: Parser<'i> + std::fmt::Debug + PartialEq,
    {
        for (test_id, (expected, i)) in tests.into_iter().enumerate() {
            let res: Result<T> = T::parse(i);
            let (i, actual) = match res {
                Ok(t) => t,
                Err(nom::Err::Error(e) | nom::Err::Failure(e)) => {
                    eprintln!("Could not parse the test case.");
                    eprintln!("Here's the error chain. Top row is the last parser tried, i.e. the bottom of the parse tree.");
                    eprintln!("The bottom row is the root of the parse tree.");
                    let err_table = Table::new(
                        e.errors
                            .into_iter()
                            .map(|(input, e)| DisplayableError::new(input, e)),
                    );
                    eprintln!("{err_table}");
                    panic!("Could not parse test case");
                }
                Err(nom::Err::Incomplete(_)) => {
                    panic!("These are not streaming parsers")
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
    fn assert_not_parse<'i, T>(i: Input<'i>)
    where
        T: Parser<'i> + std::fmt::Debug + PartialEq,
    {
        let _err = T::parse(i).unwrap_err();
    }

    #[test]
    fn test_expr_number() {
        assert_parse(vec![
            (Expression::Number(123), Input::new("12_3")),
            (Expression::Number(123), Input::new("123")),
            (Expression::Number(1), Input::new("1")),
            (Expression::Number(2), Input::new("2")),
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
                Input::new("(1 + 2)"),
            ),
            (Expression::Number(123), Input::new("123")),
        ]);
    }

    #[test]
    fn valid_function_invocations() {
        assert_parse(vec![(
            Expression::FnInvocation(FnInvocation {
                fn_name: Identifier::from_span("sphere", 0, 1),
                args: vec![Expression::Number(1), Expression::Number(2)],
            }),
            Input::new("sphere(1, 2)"),
        )])
    }

    #[test]
    fn valid_function_definition() {
        assert_parse(vec![
            (
                FnDef {
                    fn_name: Identifier::from_span("bigCircle", 0, 1),
                    params: vec![
                        Parameter {
                            name: Identifier::from_span("radius", 13, 1),
                            kcl_type: Identifier::from_span("Distance", 21, 1),
                        },
                        Parameter {
                            name: Identifier::from_span("center", 31, 1),
                            kcl_type: Identifier::from_span("Point2D", 39, 1),
                        },
                    ],
                    body: Expression::FnInvocation(FnInvocation {
                        fn_name: Identifier::from_span("circle", 62, 1),
                        args: vec![Expression::Name(Identifier::from_span("radius", 69, 1))],
                    }),
                    return_type: Identifier::from_span("Solid2D", 50, 1),
                },
                Input::new(
                    r#"bigCircle = (radius: Distance, center: Point2D -> Solid2D) => circle(radius)"#,
                ),
            ),
            (
                FnDef {
                    fn_name: Identifier::from_span("bigCircle", 0, 1),
                    params: vec![Parameter {
                        name: Identifier::from_span("center", 13, 1),
                        kcl_type: Identifier::from_span("Point2D", 21, 1),
                    }],
                    body: Expression::LetIn {
                        r#let: vec![Assignment {
                            identifier: Identifier::from_span("radius", 84, 3),
                            value: Expression::Arithmetic {
                                lhs: Box::new(Expression::Number(14)),
                                op: Operator::Mul,
                                rhs: Box::new(Expression::Number(100)),
                            },
                        }],
                        r#in: Box::new(Expression::FnInvocation(FnInvocation {
                            fn_name: Identifier::from_span("circle", 123, 4),
                            args: vec![
                                Expression::Name(Identifier::from_span("radius", 130, 4)),
                                Expression::Name(Identifier::from_span("center", 138, 4)),
                            ],
                        })),
                    },
                    return_type: Identifier::from_span("Solid2D", 32, 1),
                },
                Input::new(
                    "\
            bigCircle = (center: Point2D -> Solid2D) =>
                let
                    radius = (14 * 100)
                in circle(radius, center)",
                ),
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
                            identifier: Identifier::from_span("x", 8, 2),
                            value: Expression::Number(1),
                        },
                        Assignment {
                            identifier: Identifier::from_span("y", 18, 3),
                            value: Expression::Name(Identifier::from_span("x", 22, 3)),
                        },
                    ],
                    r#in: Box::new(Expression::Name(Identifier::from_span("y", 27, 4))),
                },
                Input::new(
                    r#"let
    x = 1
    y = x
in y"#,
                ),
            ),
            (
                Expression::LetIn {
                    r#let: vec![Assignment {
                        identifier: Identifier::from_span("radius", 20, 2),
                        value: Expression::Arithmetic {
                            lhs: Box::new(Expression::Number(14)),
                            op: Operator::Mul,
                            rhs: Box::new(Expression::Number(100)),
                        },
                    }],
                    r#in: Box::new(Expression::FnInvocation(FnInvocation {
                        fn_name: Identifier::from_span("circle", 55, 3),
                        args: vec![
                            Expression::Name(Identifier::from_span("radius", 62, 3)),
                            Expression::Name(Identifier::from_span("center", 70, 3)),
                        ],
                    })),
                },
                Input::new(
                    r#"let
                radius = (14 * 100)
            in circle(radius, center)"#,
                ),
            ),
        ])
    }

    #[test]
    fn test_assignment() {
        let valid_lhs = ["n"];
        let tests: Vec<_> = valid_lhs
            .into_iter()
            .flat_map(|lhs| {
                vec![
                    (
                        Assignment {
                            identifier: Identifier::from_span(lhs, 0, 1),
                            value: Expression::FnInvocation(FnInvocation {
                                fn_name: Identifier::from_span("foo", 4, 1),
                                args: vec![Expression::Number(100)],
                            }),
                        },
                        format!("{lhs} = foo(100)"),
                    ),
                    (
                        Assignment {
                            identifier: Identifier::from_span(lhs, 0, 1),
                            value: Expression::Number(100),
                        },
                        format!("{lhs} = 100"),
                    ),
                ]
            })
            .collect();
        let ref_to_tests = tests
            .iter()
            .map(|(exp, input)| (exp.to_owned(), Input::new(input.as_str())))
            .collect();
        assert_parse(ref_to_tests)
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
            assert_not_parse::<Assignment>(Input::from(i.as_ref()))
        }
    }
}
