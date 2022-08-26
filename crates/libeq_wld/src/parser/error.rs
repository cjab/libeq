use nom::error::{ContextError, ErrorKind, ParseError};

use super::FragmentHeader;

#[derive(Debug)]
pub enum WldDocError<'a> {
    Parse {
        input: &'a [u8],
        message: String,
    },
    ParseFragment {
        index: usize,
        offset: usize,
        header: FragmentHeader<'a>,
        message: String,
    },
    UnknownFragment {
        index: usize,
        header: FragmentHeader<'a>,
    },
}

impl ContextError<&'_ [u8]> for WldDocError<'_> {
    fn add_context(_input: &'_ [u8], ctx: &'static str, other: Self) -> Self {
        match other {
            Self::Parse { input, message } => Self::Parse {
                input,
                message: format!("{}: {}", ctx, message),
            },
            _ => panic!("Only WldDocError::Parse errors can be given context"),
        }
    }
}

impl<'a> From<nom::Err<WldDocError<'a>>> for WldDocError<'a> {
    fn from(e: nom::Err<WldDocError<'a>>) -> Self {
        match e {
            nom::Err::Error(e) => e,
            nom::Err::Failure(e) => e,
            // No need for `Incomplete`, we can assume this parser will
            // always be given the full data.
            nom::Err::Incomplete(_needed) => panic!("No support for parsing of incomplete data."),
        }
    }
}

impl<'a> ParseError<&'a [u8]> for WldDocError<'a> {
    fn from_error_kind(input: &'a [u8], kind: ErrorKind) -> Self {
        Self::Parse {
            input,
            message: format!("{:?}", kind),
        }
    }

    fn append(input: &'a [u8], kind: ErrorKind, other: Self) -> Self {
        match other {
            Self::Parse { message, .. } => Self::Parse {
                input,
                message: format!("{:?} -> {}", kind, message),
            },
            _ => panic!("Only WldDocError::Parse errors can be appended to"),
        }
    }

    fn from_char(_input: &[u8], _: char) -> Self {
        panic!("This is a binary parser");
    }

    fn or(self, other: Self) -> Self {
        other
    }
}
