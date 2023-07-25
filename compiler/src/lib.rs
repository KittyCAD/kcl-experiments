mod ast;
pub mod displayable_error;
mod parser;

pub use ast::AbstractSyntaxTree;
use displayable_error::{DisplayErr, DisplayableError};
use nom::Finish;
use parser::Parser;

pub fn parse(source_code: &str) -> Result<(&str, ast::AbstractSyntaxTree), Vec<DisplayableError>> {
    AbstractSyntaxTree::parse(source_code)
        .finish()
        .map_err(|e| {
            e.errors
                .into_iter()
                .map(|(input, e)| DisplayableError {
                    input,
                    error: DisplayErr::from(e),
                })
                .collect()
        })
}
