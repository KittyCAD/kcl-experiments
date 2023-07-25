use nom::error::VerboseErrorKind;
use std::fmt;

#[derive(tabled::Tabled)]
pub struct DisplayableError<'i> {
    pub input: &'i str,
    pub error: DisplayErr,
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
