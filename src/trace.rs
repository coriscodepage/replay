use std::convert::TryInto;
use std::ops::BitOr;
use std::panic::Location;
use std::error::Error;

use regex::Regex;

use crate::file;
use crate::parser;
use crate::value_structure::Value;

pub enum Event {
    EventEnter,
    EventLeave,
}



#[repr(C)]
#[allow(dead_code)]
pub enum Type {
    TypeNull = 0,
    TypeFalse,
    TypeTrue,
    TypeSint,
    TypeUint,
    TypeFloat,
    TypeDouble,
    TypeString,
    TypeBlob,
    TypeEnum,
    TypeBitmask,
    TypeArray,
    TypeStruct,
    TypeOpaque,
    TypeRepr,
    TypeWstring,
}

