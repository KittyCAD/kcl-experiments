mod ast;
pub mod displayable_error;
mod parser;
pub mod semantics;

pub use ast::AbstractSyntaxTree;
use displayable_error::DisplayableError;
use nom::Finish;
use parser::{Input, Parser};

/// Parse the AST.
/// If successful, returns the remaining unparsed input and the AST.
/// If error, return the "parser tree trace", i.e. a the stack trace of all parsers which
/// were attempting to parse when the deepest one failed. Ordered from deepest to root.
pub fn parse(
    source_code: &str,
) -> Result<(parser::Input, ast::AbstractSyntaxTree), Vec<DisplayableError>> {
    let input = Input::new(source_code);
    AbstractSyntaxTree::parse(input).finish().map_err(|e| {
        e.errors
            .into_iter()
            .map(|(input, e)| DisplayableError::new(input, e))
            .collect()
    })
}
