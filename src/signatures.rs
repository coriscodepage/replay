use std::{error::Error, panic::Location};

use crate::{file, parser};

#[derive(Debug)]
pub enum FunctionSignatureError {
    ParserError(&'static Location<'static>, parser::ParserError),
    SnappyError(&'static Location<'static>, file::SnappyError),
}

impl std::fmt::Display for FunctionSignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FunctionSignatureError::ParserError(loc, err) => {
                write!(f, "Parser error: {} at {}:{}", err, loc.file(), loc.line())
            }
            FunctionSignatureError::SnappyError(loc, err) => {
                write!(f, "Snappy error: {} at {}:{}", err, loc.file(), loc.line())
            }
        }
    }
}

impl FunctionSignatureError {
    #[track_caller]
    fn parser_error(error: parser::ParserError) -> Self {
        Self::ParserError(Location::caller(), error)
    }
    #[track_caller]
    fn snappy_error(error: file::SnappyError) -> Self {
        Self::SnappyError(Location::caller(), error)
    }
}

impl Error for FunctionSignatureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FunctionSignatureError::ParserError(_, parser_error) => Some(parser_error),
            FunctionSignatureError::SnappyError(_, snappy_error) => Some(snappy_error),
        }
    }
}

impl From<parser::ParserError> for FunctionSignatureError {
    fn from(value: parser::ParserError) -> Self {
        FunctionSignatureError::parser_error(value)
    }
}

impl From<file::SnappyError> for FunctionSignatureError {
    fn from(value: file::SnappyError) -> Self {
        FunctionSignatureError::snappy_error(value)
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct FunctionSignature {
    pub id: usize,
    pub name: String,
    pub num_args: usize,
    pub arg_names: Vec<String>,
    pub flag: Option<u16>,
    pub state: Option<file::Position>
}

#[derive(Debug, Clone, Default)]
pub(crate) struct EnumValue {
    pub name: String,
    pub value: i64,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct EnumSignature {
    pub id: usize,
    pub num_values: usize,
    pub values: Vec<EnumValue>,
    pub state: Option<file::Position>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct StructSignature {
    pub id: usize,
    pub name: String,
    pub num_members: usize,
    pub member_names: Vec<String>,
    pub state: Option<file::Position>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct BitmaskFlag {
    pub name: String,
    pub value: usize,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct BitmaskSignature {
    pub id: usize,
    pub num_flags: usize,
    pub bitmask_flags: Vec<BitmaskFlag>,
    pub state: Option<file::Position>,
}