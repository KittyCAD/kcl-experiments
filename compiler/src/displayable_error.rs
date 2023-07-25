use std::fmt;

use nom::error::VerboseErrorKind;

use crate::parser::Input;

#[derive(tabled::Tabled)]
pub struct DisplayableError<'i> {
    pub input: Input<'i>,
    pub error: DisplayErr,
    pub line: u32,
    pub column: usize,
}

impl<'i> DisplayableError<'i> {
    pub fn new(input: Input<'i>, error: VerboseErrorKind) -> Self {
        Self {
            input,
            error: DisplayErr::from(error),
            line: input.location_line(),
            column: input.get_utf8_column(),
        }
    }
}

pub struct DisplayErr(pub VerboseErrorKind);

impl From<VerboseErrorKind> for DisplayErr {
    fn from(value: VerboseErrorKind) -> Self {
        Self(value)
    }
}

impl From<DisplayErr> for VerboseErrorKind {
    fn from(value: DisplayErr) -> Self {
        value.0
    }
}

impl fmt::Display for DisplayErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            VerboseErrorKind::Context(s) => s.fmt(f),
            VerboseErrorKind::Char(c) => c.fmt(f),
            VerboseErrorKind::Nom(kind) => kind.description().fmt(f),
        }
    }
}
