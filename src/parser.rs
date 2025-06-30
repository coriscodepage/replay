use std::{collections::HashMap, convert::TryFrom, error::Error};

use crate::{
    file::{SnappyError, SnappyFile},
    trace::{self, Call, CallDetail, CallError, Event, FunctionSignature, Type},
    value_structure::{self, Value},
};

#[repr(C)]
#[derive(PartialEq, Debug)]
pub enum API {
    ApiUnknown = 0,
    ApiGl,  // GL + GLX/WGL/CGL
    ApiEgl, // GL/GLES1/GLES2/VG + EGL
    ApiDx,  // All DirectX
    ApiD3d7,
    ApiD3d8,
    ApiD3d9,
    ApiDxgi, // D3D10.x, D3D11.x
    ApiD2d1, // Direct2D
    ApiMax,
}

#[derive(Debug)]
pub enum ParserError {
    VersionMismatch,
    SnappyError(SnappyError),
    CallFormingError(CallError),
}

impl From<SnappyError> for ParserError {
    fn from(value: SnappyError) -> Self {
        Self::SnappyError(value)
    }
}

impl From<CallError> for ParserError {
    fn from(value: CallError) -> Self {
        Self::CallFormingError(value)
    }
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error for ParserError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParserError::SnappyError(err) => Some(err),
            ParserError::CallFormingError(err) => Some(err),
            _ => None,
        }
    }
}

pub struct Parser {
    pub snappy: SnappyFile,
    pub version: usize,
    pub api: API,
    pub properties: HashMap<String, String>,
    functions: Vec<Option<FunctionSignature>>,
    call_index: usize,
}

impl Parser {
    const TRACE_VERSION: usize = 6;
    pub fn new(path: &str) -> Result<Self, ParserError> {
        let mut snappy = SnappyFile::new(path)?;
        let version = snappy.read_varint()?;
        if version > Self::TRACE_VERSION {
            return Err(ParserError::VersionMismatch);
        }
        if version < snappy.read_varint()? {
            return Err(ParserError::VersionMismatch);
        }
        Ok(Self {
            snappy,
            version,
            api: API::ApiUnknown,
            properties: HashMap::new(),
            functions: Vec::new(),
            call_index: 0,
        })
    }

    pub fn parse_properties(&mut self) -> Result<(), ParserError> {
        'properties_parser: loop {
            let name = match self.snappy.read_string() {
                Ok(name) => name,
                Err(SnappyError::InsufficientData) => break 'properties_parser,
                Err(err) => return Err(ParserError::SnappyError(err)),
            };
            let value = match self.snappy.read_string() {
                Ok(name) => name,
                Err(SnappyError::InsufficientData) => String::new(),
                Err(err) => return Err(ParserError::SnappyError(err)),
            };
            self.properties.insert(name, value);
        }
        Ok(())
    }

    fn parse_call(&mut self) -> Result<Call, ParserError> {
        match Event::try_from(self.snappy.read_type::<u8>()?).unwrap() {
            Event::EventEnter => {
                let thread_id = self.snappy.read_varint()?;
                let signature = self.parse_function_sig()?;
                let sig = self.parse_function_sig()?;
                self.call_index = self.call_index + 1;
                return Ok(Call {
                    flag: sig.flag,
                    sig: sig,
                    index: self.call_index, //NOTE: this is a guess
                    ret: None,
                    args: Vec::new(),
                });
            }
            Event::EventLeave => todo!(),
        };
    }

    pub fn parse_call_detail(&mut self) -> Result<Option<()>, ParserError> {
        loop {
            match self.snappy.read_type::<u8>() {
                Err(_) => return Ok(None),
                Ok(val) => {
                    match val {
                        n if trace::CallDetail::CallEnd as u8 == n => todo!(), //TODO return CALL
                        n if trace::CallDetail::CallArg as u8 == n => todo!(), //TODO
                        n if trace::CallDetail::CallRet as u8 == n => todo!(), //TODO
                        n if trace::CallDetail::CallBacktrace as u8 == n => todo!(), //TODO
                        n if trace::CallDetail::CallFlags as u8 == n => todo!(), //TODO
                        _ => panic!("Unknown call detail"),
                    }
                }
            };
        }
    }

    fn parse_value(&mut self) -> Result<Option<Box<dyn Value>>, ParserError> {
        let mut return_value: Option<Box<dyn Value>> = None;
        match self.snappy.read_type::<u8>() {
            Err(_) => return Err(ParserError::SnappyError(SnappyError::InsufficientData)),
            Ok(val) => match val {
                n if trace::Type::TypeNull as u8 == n => {
                    return Ok(Some(Box::new(value_structure::None {})))
                }
                n if trace::Type::TypeFalse as u8 == n => {
                    return Ok(Some(Box::new(value_structure::Bool{value: false})))
                }
                n if trace::Type::TypeTrue as u8 == n => {
                    return Ok(Some(Box::new(value_structure::Bool{value: true})))
                }
                _ => panic!("Unknown type"),
            },
        }
        Err(ParserError::VersionMismatch)
    }

    pub fn parse_function_sig(&mut self) -> Result<FunctionSignature, ParserError> {
        let id = self.snappy.read_varint()?;
        if id > self.functions.len() {
            self.functions.resize(id + 1, None);
        } else if let Some(r) = self.functions[id].as_ref() {
            return Ok(r.clone());
        }
        let name = self.snappy.read_string()?;
        let num_args = self.snappy.read_varint()?;
        let mut arg_names = Vec::with_capacity(num_args);

        for _ in 0..num_args {
            arg_names.push(self.snappy.read_string()?);
        }
        let flag = Call::lookup_call_flag(&name)?;

        if self.api == API::ApiUnknown {
            self.api = match &name {
                n if n.starts_with("glX") => API::ApiGl,
                n if n.starts_with("wgl")
                    && n.chars().nth(3).map(|c| c.is_ascii_uppercase()) == Some(true) =>
                {
                    API::ApiGl
                }
                n if n.starts_with("CGL") => API::ApiGl,
                n if n.starts_with("egl")
                    && n.chars().nth(3).map(|c| c.is_ascii_uppercase()) == Some(true) =>
                {
                    API::ApiEgl
                }
                n if n.starts_with("Direct") || n.starts_with("D3D") || n.starts_with("Create") => {
                    API::ApiDx
                }
                _ => API::ApiUnknown,
            }
        }
        let sig = FunctionSignature {
            id,
            name,
            num_args,
            arg_names,
            flag,
            state: Some(self.snappy.get_current_offset()),
        };
        self.functions[id] = Some(sig.clone());
        // TODO: gl error sig
        Ok(sig)
    }
}
