use std::{cell::{RefCell, UnsafeCell}, collections::LinkedList, ffi::CString, fmt::Debug, ops::Index, os::raw::c_void, ptr::NonNull, rc::Rc, sync::{Mutex, OnceLock, RwLock}};
use std::thread_local;
use libc::c_char;

use crate::signatures;

use std::any::Any;

thread_local! {
static LEAKED_BLOBS:   RefCell<LinkedList<NonNull<Blob>>> = RefCell::new(LinkedList::new());
}

pub trait Value: Debug + Any {
    fn to_bool(&self) -> Option<bool>;
    fn to_u32(&self) -> Option<u32>;
    fn to_i32(&self) -> Option<i32>;
    fn to_f32(&self) -> Option<f32>;
    fn to_f64(&self) -> Option<f64>;
    fn to_array(&self) -> Option<&Array>;
    fn to_pointer(&self) -> Option<*mut c_void>;
    fn as_any(&self) -> &dyn Any;
    fn to_string(&self) -> *mut c_char;
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
       None
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        Some(std::ptr::null_mut())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn to_string(&self) -> *mut c_char {
        todo!()
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
    
    fn to_string(&self) -> *mut c_char {
        todo!()
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
    
    fn to_string(&self) -> *mut c_char {
        todo!()
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
    
    fn to_string(&self) -> *mut c_char {
        todo!()
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
    
    fn to_string(&self) -> *mut c_char {
        todo!()
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
    
    fn to_string(&self) -> *mut c_char {
        todo!()
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
        None
    }

    fn to_pointer(&self) -> Option<*mut c_void> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn to_string(&self) -> *mut c_char {
        let mut val = self.value.clone();
        //val.push('\0');
        CString::new(val).unwrap().into_raw()
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
    
    fn to_string(&self) -> *mut c_char {
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
    
    fn to_string(&self) -> *mut c_char {
        todo!()
    }
}

impl Array {
    fn size(&self) -> usize {
        self.values.len()
    }
}

impl Index<usize> for Array {
    type Output = Box<dyn Value>;

    fn index(&self, index: usize) -> &Self::Output {
        self.values.get(index).unwrap()
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
        Some(self.value as u32)
    }
    fn to_f32(&self) -> Option<f32> {
        todo!()
    }
    fn to_f64(&self) -> Option<f64> {
        todo!()
    }
    fn to_i32(&self) -> Option<i32> {
        Some(self.value as i32)
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
    
    fn to_string(&self) -> *mut c_char {
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
    
    fn to_string(&self) -> *mut c_char {
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
    fn to_u32(&self) -> Option<u32> {
        Some(self.value as u32)
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
    
    fn to_string(&self) -> *mut c_char {
        todo!()
    }
}

#[derive(Debug, Clone)]
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
        if self.bound {
            let boxed_self = Box::new(self.clone());
            let raw_ptr = Box::into_raw(boxed_self);
            let nonnull = unsafe { NonNull::new_unchecked(raw_ptr) };
            LEAKED_BLOBS.with(|blobs| {
                blobs.borrow_mut().push_back(nonnull);
            });
            //leaked.write().unwrap().push_front(raw_ptr);
            Some(nonnull.as_ptr() as *mut c_void)
        }
        else {
            Some(self.buffer.as_ptr() as *mut c_void)
        }
        
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn to_string(&self) -> *mut c_char {
        todo!()
    }
}
