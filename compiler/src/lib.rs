mod ast;
pub mod displayable_error;
mod parser;

pub use ast::AbstractSyntaxTree;
use displayable_error::DisplayableError;
use nom::Finish;
use parser::{Input, Parser};

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
