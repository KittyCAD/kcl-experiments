///! Abstract syntax tree that KCL files get parsed into.
use std::fmt;

/// For now, a KCL program is just a series of function definitions.
/// TODO: It should support also:
///  - Comments
///  - Import statements
pub type AbstractSyntaxTree = Vec<FnDef>;

/// A KCL identifier can have a value bound to it.
/// Basically, it's anything that can be used as the name of a constant, function or type.
/// E.g. in `x = 1` the identifier is the name `x`.
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Identifier(pub String);

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

// In tests, you can turn a Rust string into an identifier.
// In prod, use the parser, because this does not guarantee that the string is a valid identifier.
#[cfg(test)]
impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// Function definition
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct FnDef {
    pub fn_name: Identifier,
    pub params: Vec<Parameter>,
    pub body: Expression,
}

/// Parameters for declared functions
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Parameter {
    pub name: Identifier,
    pub kcl_type: Identifier,
}

/// Function invocation
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct FnInvocation {
    pub fn_name: Identifier,
    pub args: Vec<Expression>,
}

/// Expressions can be evaluated (producing a value)
/// or bound to identifiers by assignments.
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub enum Expression {
    /// Numbers are expressions
    Number(u64),
    /// Function invocations evaluate to their return value.
    FnInvocation(FnInvocation),
    /// A value bound to a name is an expression.
    /// It evaluates to the bound value.
    Name(Identifier),
    /// Let-in expressions evaluate to the `in` part.
    LetIn {
        r#let: Vec<Assignment>,
        r#in: Box<Expression>,
    },
}

/// Assigning a value to a binding, e.g. `n = 100`.
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Assignment {
    pub identifier: Identifier,
    pub value: Expression,
}