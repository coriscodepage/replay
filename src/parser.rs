use std::{
    collections::{HashMap, LinkedList, VecDeque},
    convert::TryFrom,
    error::Error,
};

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
    call_number: usize,
    call_list: VecDeque<Call>,
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
            call_number: 0,
            call_list: VecDeque::new(),
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

    pub fn parse_call(&mut self) -> Result<Call, ParserError> {
        loop {
            match self.snappy.read_type::<u8>() {
                Ok(val) => match Event::try_from(val).unwrap() {
                    Event::EventEnter => {
                        let thread_id = self.snappy.read_varint()?;
                        let sig = self.parse_function_sig()?;
                        let mut call = Call {
                            sig,
                            number: self.call_number, //NOTE: this is a guess
                            ret: None,
                            args: Vec::new(),
                            thread_id: thread_id as u16,
                        };
                        self.call_number = self.call_number + 1;
                        if self.parse_call_detail(&mut call)? {
                            self.call_list.push_back(call);
                        }
                    }
                    Event::EventLeave => {
                        let call_number = self.snappy.read_varint()?;
                        let mut call: Option<Call> = None;
                        for el in 0..self.call_list.len() {
                            if self.call_list[el].number == call_number {
                                call = self.call_list.remove(el);
                            }
                        }
                        if call.is_none() {
                            call = Some(Call::default());
                            let _ = self.parse_call_detail(&mut call.as_mut().unwrap())?;
                        } else if self.parse_call_detail(&mut call.as_mut().unwrap())? {
                            //TODO Whole glErrr handling. Works without
                            return Ok(call.unwrap());
                        }
                    }
                },

                Err(_) => {
                    if !self.call_list.is_empty() {
                        let call = self.call_list.front_mut().unwrap();
                        call.sig.flag = Some(call.sig.flag.unwrap_or(0) | 64);
                        let call = self.call_list.pop_front();
                        //TODO Whole glErrr handling. Works without
                        return Ok(call.unwrap());
                    }
                    return Err(ParserError::CallFormingError(CallError::NoCallAvailable));
                }
            }
        }
    }

    pub fn parse_call_detail(&mut self, call: &mut Call) -> Result<bool, ParserError> {
        loop {
            match self.snappy.read_type::<u8>() {
                Err(_) => return Ok(false),
                Ok(val) => {
                    match val {
                        n if trace::CallDetail::CallEnd as u8 == n => return Ok(true),
                        n if trace::CallDetail::CallArg as u8 == n => {
                            call.args = self.parse_arg()?
                        }
                        n if trace::CallDetail::CallRet as u8 == n => {
                            call.ret = self.parse_value()?
                        }
                        n if trace::CallDetail::CallBacktrace as u8 == n => {} //TODO
                        n if trace::CallDetail::CallFlags as u8 == n => {
                            let flag = self.snappy.read_varint()?;
                            if flag & 1 == 1 {
                                call.sig.flag = Some(call.sig.flag.unwrap_or(0) | 1);
                            }
                        }
                        _ => panic!("Unknown call detail"),
                    }
                }
            };
        }
    }

    fn parse_arg(&mut self) -> Result<Vec<Box<dyn Value>>, ParserError> {
        let index = self.snappy.read_varint()?;
        let mut v_args = Vec::<Box<dyn Value>>::new();
        if let Some(val) = self.parse_value()? {
            if index >= v_args.len() {
                v_args.resize_with(v_args.len() + index + 1, || {Box::new(value_structure::None{})});
            }
            v_args[index] = val;
        };
        Ok(v_args)
    }

    fn parse_value(&mut self) -> Result<Option<Box<dyn Value>>, ParserError> {
        match self.snappy.read_type::<u8>() {
            Err(_) => return Err(ParserError::SnappyError(SnappyError::InsufficientData)),
            Ok(val) => match val {
                n if trace::Type::TypeNull as u8 == n => {
                    return Ok(Some(Box::new(value_structure::None {})))
                }
                n if trace::Type::TypeFalse as u8 == n => {
                    return Ok(Some(Box::new(value_structure::Bool { value: false })))
                }
                n if trace::Type::TypeTrue as u8 == n => {
                    return Ok(Some(Box::new(value_structure::Bool { value: true })))
                }
                n if trace::Type::TypeSint as u8 == n => {
                    return Ok(Some(Box::new(value_structure::I64 {
                        value: -(self.snappy.read_varint()? as i64),
                    })))
                }
                n if trace::Type::TypeUint as u8 == n => {
                    return Ok(Some(Box::new(value_structure::Usize {
                        value: self.snappy.read_varint()?,
                    })))
                }
                n if trace::Type::TypeFloat as u8 == n => {
                    return Ok(Some(Box::new(value_structure::Float {
                        value: self.snappy.read_type::<f32>()?,
                    })))
                }
                n if trace::Type::TypeDouble as u8 == n => {
                    return Ok(Some(Box::new(value_structure::Double {
                        value: self.snappy.read_type::<f64>()?,
                    })))
                }
                n if trace::Type::TypeString as u8 == n => {
                    return Ok(Some(Box::new(value_structure::VString {
                        value: self.snappy.read_string()?,
                    })))
                }
                n if trace::Type::TypeEnum as u8 == n => {
                    let enum_signature = todo!();
                    let value = self.snappy.read_signed_varint()?;
                },
                n if trace::Type::TypeBitmask as u8 == n => todo!(),
                n if trace::Type::TypeArray as u8 == n => {
                    let len = self.snappy.read_varint()?;
                    let mut arr = value_structure::Array{ values: Vec::new() };
                    arr.values.resize_with(len, || {Box::new(value_structure::None{})});
                    for i in 0..len {
                        arr.values[i] = self.parse_value().unwrap().unwrap();
                    }
                    return Ok(Some(Box::new(arr)));
                },
                n if trace::Type::TypeStruct as u8 == n => todo!(),
                n if trace::Type::TypeBlob as u8 == n => todo!(),
                n if trace::Type::TypeOpaque as u8 == n => {
                    return Ok(Some(Box::new(value_structure::Pointer {
                        value: self.snappy.read_varint()? as *mut std::ffi::c_void,
                    })))
                },
                n if trace::Type::TypeRepr as u8 == n => todo!(),
                n if trace::Type::TypeWstring as u8 == n => todo!(),

                _ => panic!("Unknown type"),
            },
        }
    }

    fn parse_function_sig(&mut self) -> Result<FunctionSignature, ParserError> {
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

    fn parse_enum_sig(&mut self) {
        
    }
}
