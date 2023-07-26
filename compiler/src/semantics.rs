use crate::AbstractSyntaxTree;

#[derive(Debug)]
pub struct SourceRange {
    pub start_line: u32,
    pub start_column: usize,
    pub length: usize,
}

impl<'i> AbstractSyntaxTree<'i> {
    /// Iterates over all functions in the file. Returns each function's name,
    /// and the source code range where that name is found.
    pub fn all_functions(&self) -> impl Iterator<Item = (&&'i str, SourceRange)> + '_ {
        self.functions.iter().map(|function| {
            let name = function.fn_name.0.fragment();
            let start_line = function.fn_name.0.location_line();
            let start_column = function.fn_name.0.get_utf8_column();
            (
                name,
                SourceRange {
                    start_line,
                    start_column,
                    length: name.len(),
                },
            )
        })
    }
}
