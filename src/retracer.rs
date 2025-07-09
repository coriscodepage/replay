use std::{collections::BTreeMap, error::Error, fmt::Display, panic::Location};
use std::cell::RefCell;
use std::rc::Rc;

use crate::{call::Call, gl_context::Context, r#try::GlRetracer};

pub type Callback = fn(&mut GlRetracer, &mut Call);
 
pub(crate) struct Entry {
    pub(crate) name: String,
    pub(crate) callback: Callback,
}

#[derive(Debug)]
pub enum RetracerError {
    NoCallback(&'static Location<'static>),
}

impl RetracerError {
    #[track_caller]
    pub fn no_callback() -> Self {
        Self::NoCallback(Location::caller())
    }
}

impl Display for RetracerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetracerError::NoCallback(location) => write!(f, "RetracerError error: NoCallback at {}:{}", location.file(), location.line()),
        }
    }
}

impl Error for RetracerError {}

pub struct Retracer {
    map: BTreeMap<String, Callback>,
    callbacks: Vec<Option<Callback>>,
    tracer: GlRetracer,
}

fn check_gl_error(operation: &str) {
    unsafe {
        let error = gl::GetError();
        if error != gl::NO_ERROR {
            let error_str = match error {
                gl::INVALID_ENUM => "GL_INVALID_ENUM",
                gl::INVALID_VALUE => "GL_INVALID_VALUE", 
                gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
                gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
                gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
                _ => "Unknown error",
            };
            println!("OpenGL Error after {}: {} (0x{:x})", operation, error_str, error);
        }
    }
}


impl Retracer {
    pub fn init(context: Rc<RefCell<Context>>) -> Self {
        Self { map: BTreeMap::new(), callbacks: Vec::new(), tracer: GlRetracer::init(context)}
    }

    pub fn retrace(&mut self, call: &mut Call) -> Result<(), RetracerError>{
        let mut callback: Option<Callback> = None;
        let id = call.sig.id;
        if id >= self.callbacks.len() {
            self.callbacks.resize(id + 1, None);
        }
        else {
            callback = self.callbacks[id];
        }

        if callback.is_none() {
            callback = self.map.get(&call.sig.name).copied();
            self.callbacks[id] = callback;
        }
        if let Some(callback) = callback {
            callback(&mut self.tracer, call);
            check_gl_error(&call.sig.name);
            Ok(())
        }
        else {
            Err(RetracerError::no_callback())
        }
    }

    pub fn add_callback(&mut self, entry: Entry) {
        self.map.insert(entry.name, entry.callback);
    }

}
