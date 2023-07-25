//! Abstract syntax tree that KCL files get parsed into.
use std::fmt;

/// For now, a KCL program is just a series of function definitions.
/// TODO: It should support also:
///  - Comments
///  - Import statements
pub type AbstractSyntaxTree<'i> = Vec<FnDef<'i>>;

/// A KCL identifier can have a value bound to it.
/// Basically, it's anything that can be used as the name of a constant, function or type.
/// E.g. in `x = 1` the identifier is the name `x`.
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq, Clone))]
pub struct Identifier<'i>(pub &'i str);

impl<'i> fmt::Display for Identifier<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

// In tests, you can turn a Rust string into an identifier.
// In prod, use the parser, because this does not guarantee that the string is a valid identifier.
#[cfg(test)]
impl<'i> From<&'i str> for Identifier<'i> {
    fn from(value: &'i str) -> Self {
        Self(value)
    }
}

/// Function definition
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct FnDef<'i> {
    pub fn_name: Identifier<'i>,
    pub params: Vec<Parameter<'i>>,
    pub return_type: Identifier<'i>,
    pub body: Expression<'i>,
}

/// Parameters for declared functions
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Parameter<'i> {
    pub name: Identifier<'i>,
    pub kcl_type: Identifier<'i>,
}

/// Function invocation
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq, Clone))]
pub struct FnInvocation<'i> {
    pub fn_name: Identifier<'i>,
    pub args: Vec<Expression<'i>>,
}

/// Expressions can be evaluated (producing a value)
/// or bound to identifiers by assignments.
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq, Clone))]
pub enum Expression<'i> {
    /// Numbers are expressions
    Number(u64),
    /// Function invocations evaluate to their return value.
    FnInvocation(FnInvocation<'i>),
    /// A value bound to a name is an expression.
    /// It evaluates to the bound value.
    Name(Identifier<'i>),
    /// Let-in expressions evaluate to the `in` part.
    LetIn {
        r#let: Vec<Assignment<'i>>,
        r#in: Box<Expression<'i>>,
    },
    Arithmetic {
        lhs: Box<Expression<'i>>,
        op: Operator,
        rhs: Box<Expression<'i>>,
    },
}

/// Expressions can be evaluated (producing a value)
/// or bound to identifiers by assignments.
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq, Clone))]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

/// Assigning a value to a binding, e.g. `n = 100`.
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq, Clone))]
pub struct Assignment<'i> {
    pub identifier: Identifier<'i>,
    pub value: Expression<'i>,
}
