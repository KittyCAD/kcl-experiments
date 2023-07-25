//! Parse KCL source into its AST.

use nom::error::VerboseError;

mod implementations;

// -----------------------------------
//           CORE TYPES
// -----------------------------------

/// Input to a parser.
///
/// All parsers read from a string containing the KCL source code. They only need to borrow the
/// string -- after all, parsers just read the string, they don't modify it or deallocate it.
/// So there's no need to take ownership. So, they borrow the string (take &str) instead of
/// owning it (take String).
///
/// Here the lifetime 'i represents the lifetime of the input (the KCL source code string).
pub type Input<'i> = &'i str;

/// Result of running a parser.
///
/// Parsers are generic over:
///  - 'i, the lifetime of the input. The parser doesn't care how long the input lasts for, so, it
///    can run on any borrowed string input, regardless of how long the borrow lasts.
///  - O, the Output type. If the parser is successful, it returns this output (O) type.
///    This is generic because different parsers return different types, e.g. a FunctionDefinition,
///    or a FunctionInvocation, or a ParameterList or an Identifier.
///
/// Parsers return a result, because the parser might succeed or fail.
///
/// The result's Ok branch is (Input, O) -- the remaining input, and the Output value which was
/// successfully parsed.
///
/// The result's Err branch is a Nom error describing which parsers Nom tried, and why they failed.
pub type Result<'i, O> = nom::IResult<Input<'i>, O, VerboseError<Input<'i>>>;

/// Take KCL source code as input and parse it into an AST node.
/// By convention, Nom parsers take a parameter `i` which means "input to the parser",
/// NOT "index to an array".
pub trait Parser<'i>: Sized {
    fn parse(i: Input<'i>) -> Result<Self>;
}
