use std::{fmt::Debug, os::raw::c_void, rc::Rc};

use crate::signatures;

use std::any::Any;

pub trait Value: Debug + Any {
    fn to_bool(&self) -> Option<bool>;
    fn to_u32(&self) -> Option<u32>;
    fn to_i32(&self) -> Option<i32>;
    fn to_f32(&self) -> Option<f32>;
    fn to_f64(&self) -> Option<f64>;
    fn to_array(&self) -> Option<&Array>;
    fn to_pointer(&self) -> Option<*mut c_void>;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct None {}

impl Value for None {
    fn to_bool(&self) -> Option<bool> {
        None
    }
    fn to_u32(&self) -> Option<u32> {
        Some(4)
    }
    fn to_f32(&self) -> Option<f32> {
        None
    }
    fn to_f64(&self) -> Option<f64> {
        None
    }
    fn to_i32(&self) -> Option<i32> {
        Some(0)
    }

    fn to_array(&self) -> Option<&Array> {
        todo!()
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Bool {
    pub value: bool,
}

impl Value for Bool {
    fn to_bool(&self) -> Option<bool> {
        Some(self.value)
    }
    fn to_u32(&self) -> Option<u32> {
        match self.value {
            true => Some(1),
            false => Some(0),
        }
    }
    fn to_i32(&self) -> Option<i32> {
        match self.value {
            true => Some(1),
            false => Some(0),
        }
    }
    fn to_f32(&self) -> Option<f32> {
        match self.value {
            true => Some(1.0),
            false => Some(0.0),
        }
    }
    fn to_f64(&self) -> Option<f64> {
        match self.value {
            true => Some(1.0),
            false => Some(0.0),
        }
    }
    fn to_array(&self) -> Option<&Array> {
        todo!()
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct U32 {
    pub value: u32,
}

impl Value for U32 {
    fn to_bool(&self) -> Option<bool> {
        Some(self.value != 0)
    }
    fn to_u32(&self) -> Option<u32> {
        Some(self.value)
    }
    fn to_i32(&self) -> Option<i32> {
        return Some(self.value as i32);
    }
    fn to_f32(&self) -> Option<f32> {
        Some(self.value as f32)
    }
    fn to_f64(&self) -> Option<f64> {
        Some(self.value as f64)
    }
    fn to_array(&self) -> Option<&Array> {
        todo!()
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct I32 {
    pub value: i32,
}

impl Value for I32 {
    fn to_bool(&self) -> Option<bool> {
        Some(self.value != 0)
    }
    fn to_u32(&self) -> Option<u32> {
        if self.value >= 0 {
            return Some(self.value as u32);
        }
        None
    }
    fn to_i32(&self) -> Option<i32> {
        Some(self.value)
    }
    fn to_f32(&self) -> Option<f32> {
        Some(self.value as f32)
    }
    fn to_f64(&self) -> Option<f64> {
        Some(self.value as f64)
    }
    fn to_array(&self) -> Option<&Array> {
        todo!()
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Float {
    pub value: f32,
}

impl Value for Float {
    fn to_bool(&self) -> Option<bool> {
        Some(self.value != 0.0)
    }
    fn to_u32(&self) -> Option<u32> {
        Some(self.value as u32)
    }
    fn to_i32(&self) -> Option<i32> {
        Some(self.value as i32)
    }
    fn to_f32(&self) -> Option<f32> {
        Some(self.value)
    }
    fn to_f64(&self) -> Option<f64> {
        Some(self.value as f64)
    }
    fn to_array(&self) -> Option<&Array> {
        todo!()
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Double {
    pub value: f64,
}

impl Value for Double {
    fn to_bool(&self) -> Option<bool> {
        Some(self.value != 0.0)
    }
    fn to_u32(&self) -> Option<u32> {
        Some(self.value as u32)
    }
    fn to_i32(&self) -> Option<i32> {
        Some(self.value as i32)
    }
    fn to_f32(&self) -> Option<f32> {
        Some(self.value as f32)
    }
    fn to_f64(&self) -> Option<f64> {
        Some(self.value)
    }
    fn to_array(&self) -> Option<&Array> {
        todo!()
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct VString {
    pub value: String,
}

impl Value for VString {
    fn to_bool(&self) -> Option<bool> {
        Some(true)
    }
    fn to_u32(&self) -> Option<u32> {
        None
    }
    fn to_f32(&self) -> Option<f32> {
        None
    }
    fn to_f64(&self) -> Option<f64> {
        None
    }
    fn to_i32(&self) -> Option<i32> {
        None
    }
    fn to_array(&self) -> Option<&Array> {
        todo!()
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Pointer {
    pub value: *mut c_void,
}

impl Value for Pointer {
    fn to_bool(&self) -> Option<bool> {
        Some(!self.value.is_null())
    }
    fn to_u32(&self) -> Option<u32> {
        Some(5)
    }
    fn to_f32(&self) -> Option<f32> {
        todo!()
    }
    fn to_f64(&self) -> Option<f64> {
        todo!()
    }
    fn to_i32(&self) -> Option<i32> {
        todo!()
    }
    fn to_array(&self) -> Option<&Array> {
        todo!()
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        Some(self.value)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Array {
    pub values: Vec<Box<dyn Value>>,
}

impl Value for Array {
    fn to_bool(&self) -> Option<bool> {
        todo!()
    }
    fn to_u32(&self) -> Option<u32> {
        todo!()
    }
    fn to_f32(&self) -> Option<f32> {
        todo!()
    }
    fn to_f64(&self) -> Option<f64> {
        todo!()
    }
    fn to_i32(&self) -> Option<i32> {
        todo!()
    }
    fn to_array(&self) -> Option<&Array> {
        Some(self)
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Array {
    fn size(&self) -> usize {
        self.values.len()
    }
}

#[derive(Debug)]
pub struct Enum {
    pub sig: Rc<signatures::EnumSignature>,
    pub value: i64,
}

impl Value for Enum {
    fn to_bool(&self) -> Option<bool> {
        todo!()
    }
    fn to_u32(&self) -> Option<u32> {
        todo!()
    }
    fn to_f32(&self) -> Option<f32> {
        todo!()
    }
    fn to_f64(&self) -> Option<f64> {
        todo!()
    }
    fn to_i32(&self) -> Option<i32> {
        todo!()
    }
    fn to_array(&self) -> Option<&Array> {
        todo!()
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Struct {
    pub sig: Rc<signatures::StructSignature>,
    pub members: Vec<Box<dyn Value>>,
}

impl Value for Struct {
    fn to_bool(&self) -> Option<bool> {
        todo!()
    }
    fn to_u32(&self) -> Option<u32> {
        todo!()
    }
    fn to_f32(&self) -> Option<f32> {
        todo!()
    }
    fn to_f64(&self) -> Option<f64> {
        todo!()
    }
    fn to_i32(&self) -> Option<i32> {
        todo!()
    }
    fn to_array(&self) -> Option<&Array> {
        todo!()
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Bitmask {
    pub sig: Rc<signatures::BitmaskSignature>,
    pub value: usize,
}

impl Value for Bitmask {
    fn to_bool(&self) -> Option<bool> {
        todo!()
    }
    fn to_u32(&self) -> Option<u32> {
        todo!()
    }
    fn to_f32(&self) -> Option<f32> {
        todo!()
    }
    fn to_f64(&self) -> Option<f64> {
        todo!()
    }
    fn to_i32(&self) -> Option<i32> {
        todo!()
    }
    fn to_array(&self) -> Option<&Array> {
        todo!()
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Blob {
    pub size: usize,
    pub buffer: Vec<u8>,
    pub bound: bool,
}

impl Value for Blob {
    fn to_bool(&self) -> Option<bool> {
        todo!()
    }
    fn to_u32(&self) -> Option<u32> {
        Some(6)
    }
    fn to_f32(&self) -> Option<f32> {
        todo!()
    }
    fn to_f64(&self) -> Option<f64> {
        todo!()
    }
    fn to_i32(&self) -> Option<i32> {
        todo!()
    }
    fn to_array(&self) -> Option<&Array> {
        todo!()
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        Some(self.buffer.as_ptr() as *mut c_void)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
