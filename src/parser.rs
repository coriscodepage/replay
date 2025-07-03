use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    panic::Location, rc::Rc,
};

use crate::{
    call::{Call, CallDetail, CallError},
    file::{SnappyError, SnappyFile},
    signatures::{
        BitmaskFlag, BitmaskSignature, EnumSignature, EnumValue, FunctionSignature, StructSignature,
    },
    trace::{self, Event},
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
    VersionMismatch(&'static Location<'static>),
    SnappyError(&'static Location<'static>, SnappyError),
    CallFormingError(&'static Location<'static>, CallError),
}

impl ParserError {
    #[track_caller]
    pub fn version_mismatch() -> Self {
        Self::VersionMismatch(Location::caller())
    }

    #[track_caller]
    pub fn snappy_error(error: SnappyError) -> Self {
        Self::SnappyError(Location::caller(), error)
    }

    #[track_caller]
    pub fn call_forming_error(error: CallError) -> Self {
        Self::CallFormingError(Location::caller(), error)
    }
}

impl From<SnappyError> for ParserError {
    #[track_caller]
    fn from(value: SnappyError) -> Self {
        Self::snappy_error(value)
    }
}

impl From<CallError> for ParserError {
    #[track_caller]
    fn from(value: CallError) -> Self {
        Self::call_forming_error(value)
    }
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParserError::VersionMismatch(location) => {
                write!(f, "Version mismatch at {}:{}", location.file(), location.line())
            }
            ParserError::SnappyError(location, err) => {
                write!(f, "Snappy error: {} at {}:{}", err, location.file(), location.line())
            }
            ParserError::CallFormingError(location, err) => {
                write!(
                    f,
                    "Call forming error: {} at {}:{}",
                    err,
                    location.file(),
                    location.line()
                )
            }
        }
    }
}

impl Error for ParserError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParserError::SnappyError(_, err) => Some(err),
            ParserError::CallFormingError(_, err) => Some(err),
            _ => None,
        }
    }
}

pub struct Parser {
    pub snappy: SnappyFile,
    pub version: usize,
    pub api: API,
    pub properties: HashMap<String, String>,
    call_number: usize,
    call_list: VecDeque<Call>,
    // SigState like Vec
    functions: Vec<Option<FunctionSignature>>,
    enums: Vec<Option<Rc<EnumSignature>>>,
    structs: Vec<Option<Rc<StructSignature>>>,
    bitmasks: Vec<Option<Rc<BitmaskSignature>>>,
}

impl Parser {
    const TRACE_VERSION: usize = 6;
    pub fn new(path: &str) -> Result<Self, ParserError> {
        let mut snappy = SnappyFile::new(path)?;
        let version = snappy.read_varint()?;
        if version > Self::TRACE_VERSION {
            return Err(ParserError::version_mismatch());
        }
        if version < snappy.read_varint()? {
            return Err(ParserError::version_mismatch());
        }
        Ok(Self {
            snappy,
            version,
            api: API::ApiUnknown,
            properties: HashMap::new(),
            call_number: 0,
            call_list: VecDeque::new(),
            functions: Vec::new(),
            enums: Vec::new(),
            structs: Vec::new(),
            bitmasks: Vec::new(),
        })
    }

    pub fn parse_properties(&mut self) -> Result<(), ParserError> {
        'properties_parser: loop {
            let name = match self.snappy.read_string() {
                Ok(name) => name,
                Err(SnappyError::InsufficientData(_)) => break 'properties_parser,
                Err(err) => return Err(ParserError::snappy_error(err)),
            };
            let value = match self.snappy.read_string() {
                Ok(name) => name,
                Err(SnappyError::InsufficientData(_)) => String::new(),
                Err(err) => return Err(ParserError::snappy_error(err)),
            };
            self.properties.insert(name, value);
        }
        Ok(())
    }

    pub fn parse_call(&mut self) -> Result<Call, ParserError> {
        loop {
            match self.snappy.read_type::<u8>() {
                Ok(val) => match val {
                    n if Event::EventEnter as u8 == n => {
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
                    n if Event::EventLeave as u8 == n => {
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
                    _ => panic!("Unknown Event type"),
                },

                Err(_) => {
                    if !self.call_list.is_empty() {
                        let call = self.call_list.front_mut().unwrap();
                        call.sig.flag = Some(call.sig.flag.unwrap_or(0) | 64);
                        let call = self.call_list.pop_front();
                        //TODO Whole glErrr handling. Works without
                        return Ok(call.unwrap());
                    }
                    return Err(ParserError::call_forming_error(CallError::NoCallAvailable));
                }
            }
        }
    }

    fn lookup<T: Default>(map: &mut Vec<T>, index: usize) -> &mut T {
        if index >= map.len() {
            map.resize_with(index + 1, T::default);
        }
        &mut map[index]
    }

    pub fn parse_call_detail(&mut self, call: &mut Call) -> Result<bool, ParserError> {
        loop {
            match self.snappy.read_type::<u8>() {
                Err(_) => return Ok(false),
                Ok(val) => {
                    match val {
                        n if CallDetail::CallEnd as u8 == n => return Ok(true),
                        n if CallDetail::CallArg as u8 == n => self.parse_arg(call)?,
                        n if CallDetail::CallRet as u8 == n => call.ret = self.parse_value()?,
                        n if CallDetail::CallBacktrace as u8 == n => {} //TODO
                        n if CallDetail::CallFlags as u8 == n => {
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

    fn parse_arg(&mut self, call: &mut Call) -> Result<(), ParserError> {
        let index = self.snappy.read_varint()?;
        if let Some(val) = self.parse_value()? {
            if index >= call.args.len() {
                call.args.resize_with(index + 1, || {
                    Box::new(value_structure::None {})
                });
            }
            call.args[index] = val;
        };
        Ok(())
    }

    fn parse_value(&mut self) -> Result<Option<Box<dyn Value>>, ParserError> {
        match self.snappy.read_type::<u8>() {
            Err(_) => return Err(ParserError::snappy_error(SnappyError::insufficient_data())),
            Ok(val) => match val {
                n if trace::Type::TypeNull as u8 == n => {
                    return Ok(Some(Box::new(value_structure::None {})));
                },
                n if trace::Type::TypeFalse as u8 == n => {
                    return Ok(Some(Box::new(value_structure::Bool { value: false })));
                },
                n if trace::Type::TypeTrue as u8 == n => {
                    return Ok(Some(Box::new(value_structure::Bool { value: true })));
                },
                n if trace::Type::TypeSint as u8 == n => {
                    return Ok(Some(Box::new(value_structure::I32 {
                        value: -(self.snappy.read_varint()? as i32),
                    })));
                },
                n if trace::Type::TypeUint as u8 == n => {
                    return Ok(Some(Box::new(value_structure::U32 {
                        value: self.snappy.read_varint()? as u32,
                    })));
                },
                n if trace::Type::TypeFloat as u8 == n => {
                    return Ok(Some(Box::new(value_structure::Float {
                        value: self.snappy.read_type::<f32>()?,
                    })));
                },
                n if trace::Type::TypeDouble as u8 == n => {
                    return Ok(Some(Box::new(value_structure::Double {
                        value: self.snappy.read_type::<f64>()?,
                    })));
                },
                n if trace::Type::TypeString as u8 == n => {
                    return Ok(Some(Box::new(value_structure::VString {
                        value: match self.snappy.read_string() {
                            Ok(val) => val,
                            Err(SnappyError::InsufficientData(_)) => String::new(),
                            Err(err) => Err(err)?,
                        },
                    })));
                },
                n if trace::Type::TypeEnum as u8 == n => {
                    let enum_signature = self.parse_enum_sig()?;
                    let value = self.snappy.read_signed_varint()?;
                    return Ok(Some(Box::new(value_structure::Enum {
                        sig: enum_signature,
                        value,
                    })));
                },
                n if trace::Type::TypeBitmask as u8 == n => {
                    let bitmask_signature = self.parse_bitmask_sig()?;
                    let value = self.snappy.read_varint()?;
                    return Ok(Some(Box::new(value_structure::Bitmask {
                        sig: bitmask_signature,
                        value,
                    })));
                },
                n if trace::Type::TypeArray as u8 == n => {
                    let len = self.snappy.read_varint()?;
                    let mut arr = value_structure::Array { values: Vec::new() };
                    arr.values
                        .resize_with(len, || Box::new(value_structure::None {}));
                    for i in 0..len {
                        arr.values[i] = (self.parse_value()?).unwrap();
                    }
                    return Ok(Some(Box::new(arr)));
                },
                n if trace::Type::TypeStruct as u8 == n => {
                    let struct_signature = self.pase_struct_sig()?;
                    let mut value_struct = value_structure::Struct {
                        sig: struct_signature,
                        members: Vec::new(),
                    };
                    for _ in 0..value_struct.sig.num_members {
                        value_struct.members.push((self.parse_value()?).unwrap());
                    }
                    return Ok(Some(Box::new(value_struct)));
                },
                n if trace::Type::TypeBlob as u8 == n => {
                    let size = self.snappy.read_varint()?;
                    let mut buffer = vec![0u8; size];
                    self.snappy.read_bytes(&mut buffer)?;
                    return Ok(Some(Box::new(value_structure::Blob {
                        size,
                        buffer,
                        bound: false,
                    })));
                },
                n if trace::Type::TypeOpaque as u8 == n => {
                    return Ok(Some(Box::new(value_structure::Pointer {
                        value: self.snappy.read_varint()? as *mut std::ffi::c_void,
                    })));
                }
                n if trace::Type::TypeRepr as u8 == n => todo!(),
                n if trace::Type::TypeWstring as u8 == n => todo!(),

                _ => panic!("Unknown type"),
            },
        }
    }

    fn parse_function_sig(&mut self) -> Result<FunctionSignature, ParserError> {
        let id = self.snappy.read_varint()?;
        let function_signature_cached = Parser::lookup(&mut self.functions, id);
        match function_signature_cached {
            Some(val) => {
                if self.snappy.get_current_offset() < *val.state.as_ref().unwrap() {
                    let _ = self.snappy.read_string()?;
                    let num_args = self.snappy.read_varint()?;
                    for _ in 0..num_args {
                        let _ = self.snappy.read_string()?;
                    }
                }
                return Ok(val.clone());
            }
            None => {}
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

    fn parse_enum_sig(&mut self) -> Result<Rc<EnumSignature>, ParserError> {
        let id = self.snappy.read_varint()?;
        let enum_signature = Parser::lookup(&mut self.enums, id);
        match enum_signature {
            Some(val) => {
                if self.snappy.get_current_offset() < *val.state.as_ref().unwrap() {
                    let num_args = self.snappy.read_varint()?;
                    for _ in 0..num_args {
                        let _ = self.snappy.read_string()?;
                        let _ = self.snappy.read_signed_varint()?;
                    }
                }
                return Ok(Rc::clone(&val));
            }
            None => {}
        }
        let num_values = self.snappy.read_varint()?;
        let mut enum_values = vec![EnumValue::default(); num_values];
        for n in &mut enum_values {
            n.name = self.snappy.read_string()?;
            n.value = self.snappy.read_signed_varint()?;
        }

        let sig = Rc::new(EnumSignature {
            id,
            num_values,
            values: enum_values,
            state: Some(self.snappy.get_current_offset()),
        });
        *enum_signature = Some(Rc::clone(&sig));
        Ok(sig)
    }

    fn pase_struct_sig(&mut self) -> Result<Rc<StructSignature>, ParserError> {
        let id = self.snappy.read_varint()?;
        let struct_signature = Parser::lookup(&mut self.structs, id);
        match struct_signature {
            Some(val) => {
                if self.snappy.get_current_offset() < *val.state.as_ref().unwrap() {
                    let _ = self.snappy.read_string()?;
                    let num_args = self.snappy.read_varint()?;
                    for _ in 0..num_args {
                        let _ = self.snappy.read_string()?;
                    }
                }
                return Ok(Rc::clone(&val));
            }
            None => {}
        }
        let name = self.snappy.read_string()?;
        let num_members = self.snappy.read_varint()?;
        let mut member_names = Vec::with_capacity(num_members);
        for _ in 0..num_members {
            member_names.push(self.snappy.read_string()?);
        }
        let sig = Rc::new(StructSignature {
            id,
            name,
            num_members,
            member_names,
            state: Some(self.snappy.get_current_offset()),
        });
        self.structs[id] = Some(Rc::clone(&sig));
        Ok(sig)
    }

    fn parse_bitmask_sig(&mut self) -> Result<Rc<BitmaskSignature>, ParserError> {
        let id = self.snappy.read_varint()?;
        let struct_signature_cached = Parser::lookup(&mut self.bitmasks, id);
        match struct_signature_cached {
            Some(val) => {
                if self.snappy.get_current_offset() < *val.state.as_ref().unwrap() {
                    let num_flags = self.snappy.read_varint()?;
                    for _ in 0..num_flags {
                        let _ = self.snappy.read_string()?;
                        let _ = self.snappy.read_varint()?;
                    }
                }
                return Ok(Rc::clone(&val));
            }
            None => {}
        }
        let num_flags = self.snappy.read_varint()?;
        let mut bitmask_flags = Vec::with_capacity(num_flags);
        for _ in 0..num_flags {
            let flag = BitmaskFlag {
                name: self.snappy.read_string()?,
                value: self.snappy.read_varint()?,
            };
            bitmask_flags.push(flag);
        }
        let sig = Rc::new(BitmaskSignature {
            id,
            num_flags,
            bitmask_flags,
            state: Some(self.snappy.get_current_offset()),
        });
        self.bitmasks[id] = Some(Rc::clone(&sig));
        Ok(sig)
    }
}
