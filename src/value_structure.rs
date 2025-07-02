use std::{fmt::Debug, os::raw::c_void, rc::Rc};

use crate::signatures;

pub trait Value: Debug {
    fn to_bool(&self) -> Option<bool>;
    fn to_usize(&self) -> Option<usize>;
    fn to_i64(&self) -> Option<i64>;
    fn to_f32(&self) -> Option<f32>;
    fn to_f64(&self) -> Option<f64>;
}

#[derive(Debug)]
pub struct None {}

impl Value for None {
    fn to_bool(&self) -> Option<bool> {
        None
    }
    fn to_usize(&self) -> Option<usize> {
        None
    }
    fn to_f32(&self) -> Option<f32> {
        None
    }
    fn to_f64(&self) -> Option<f64> {
        None
    }
    fn to_i64(&self) -> Option<i64> {
        None
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
    fn to_usize(&self) -> Option<usize> {
        match self.value {
            true => Some(1),
            false => Some(0),
        }
    }
    fn to_i64(&self) -> Option<i64> {
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
}

#[derive(Debug)]
pub struct Usize {
    pub value: usize,
}

impl Value for Usize {
    fn to_bool(&self) -> Option<bool> {
        Some(self.value != 0)
    }
    fn to_usize(&self) -> Option<usize> {
        Some(self.value)
    }
    fn to_i64(&self) -> Option<i64> {
        return Some(self.value as i64);
    }
    fn to_f32(&self) -> Option<f32> {
        Some(self.value as f32)
    }
    fn to_f64(&self) -> Option<f64> {
        Some(self.value as f64)
    }
}

#[derive(Debug)]
pub struct I64 {
    pub value: i64,
}

impl Value for I64 {
    fn to_bool(&self) -> Option<bool> {
        Some(self.value != 0)
    }
    fn to_usize(&self) -> Option<usize> {
        if self.value >= 0 {
            return Some(self.value as usize);
        }
        None
    }
    fn to_i64(&self) -> Option<i64> {
        Some(self.value)
    }
    fn to_f32(&self) -> Option<f32> {
        Some(self.value as f32)
    }
    fn to_f64(&self) -> Option<f64> {
        Some(self.value as f64)
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
    fn to_usize(&self) -> Option<usize> {
        Some(self.value as usize)
    }
    fn to_i64(&self) -> Option<i64> {
        Some(self.value as i64)
    }
    fn to_f32(&self) -> Option<f32> {
        Some(self.value)
    }
    fn to_f64(&self) -> Option<f64> {
        Some(self.value as f64)
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
    fn to_usize(&self) -> Option<usize> {
        Some(self.value as usize)
    }
    fn to_i64(&self) -> Option<i64> {
        Some(self.value as i64)
    }
    fn to_f32(&self) -> Option<f32> {
        Some(self.value as f32)
    }
    fn to_f64(&self) -> Option<f64> {
        Some(self.value)
    }
}

#[derive(Debug)]
pub struct VString {
    pub value: String
}

impl Value for VString {
    fn to_bool(&self) -> Option<bool> {
        Some(true)
    }
    fn to_usize(&self) -> Option<usize> {
        None
    }
    fn to_f32(&self) -> Option<f32> {
        None
    }
    fn to_f64(&self) -> Option<f64> {
        None
    }
    fn to_i64(&self) -> Option<i64> {
        None
    }
}

#[derive(Debug)]
pub struct Pointer {
    pub value: *mut c_void,
}

impl Value for Pointer {
    fn to_bool(&self) -> Option<bool> {
        todo!()
    }
    fn to_usize(&self) -> Option<usize> {
        todo!()
    }
    fn to_f32(&self) -> Option<f32> {
        todo!()
    }
    fn to_f64(&self) -> Option<f64> {
        todo!()
    }
    fn to_i64(&self) -> Option<i64> {
        todo!()
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
    fn to_usize(&self) -> Option<usize> {
        todo!()
    }
    fn to_f32(&self) -> Option<f32> {
        todo!()
    }
    fn to_f64(&self) -> Option<f64> {
        todo!()
    }
    fn to_i64(&self) -> Option<i64> {
        todo!()
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
    fn to_usize(&self) -> Option<usize> {
        todo!()
    }
    fn to_f32(&self) -> Option<f32> {
        todo!()
    }
    fn to_f64(&self) -> Option<f64> {
        todo!()
    }
    fn to_i64(&self) -> Option<i64> {
        todo!()
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
    fn to_usize(&self) -> Option<usize> {
        todo!()
    }
    fn to_f32(&self) -> Option<f32> {
        todo!()
    }
    fn to_f64(&self) -> Option<f64> {
        todo!()
    }
    fn to_i64(&self) -> Option<i64> {
        todo!()
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
    fn to_usize(&self) -> Option<usize> {
        todo!()
    }
    fn to_f32(&self) -> Option<f32> {
        todo!()
    }
    fn to_f64(&self) -> Option<f64> {
        todo!()
    }
    fn to_i64(&self) -> Option<i64> {
        todo!()
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
    fn to_usize(&self) -> Option<usize> {
        todo!()
    }
    fn to_f32(&self) -> Option<f32> {
        todo!()
    }
    fn to_f64(&self) -> Option<f64> {
        todo!()
    }
    fn to_i64(&self) -> Option<i64> {
        todo!()
    }
}