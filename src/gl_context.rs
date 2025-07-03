use std::assert;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Context {
    pub gl_ctx: Rc<sdl3::video::GLContext>,

    //drawable: Option<*mut glws::Drawable>,
    //readable: Option<*mut glws::Drawable>,

    pub current_user_program: u32,
    pub current_program: u32,
    pub current_pipeline: u32,

    pub inside_begin_end: bool,
    pub inside_list: bool,
    pub needs_flush: bool,

    pub used: bool,

    pub khr_debug: bool,
    pub max_debug_message_length: i32,
}

impl Context {
    pub fn new(gl_ctx: Rc<sdl3::video::GLContext>) -> Self {
        Self {
            gl_ctx,
            //drawable: None, !TODO
            //readable: None, !TODO
            current_user_program: 0,
            current_program: 0,
            current_pipeline: 0,
            inside_begin_end: false,
            inside_list: false,
            needs_flush: false,
            used: false,
            khr_debug: false,
            max_debug_message_length: 0,
        }
    }

    pub fn acquire(&mut self) -> Rc<sdl3::video::GLContext> {
        Rc::clone(&self.gl_ctx)
    }

    pub fn features(&mut self) {
        unsafe {
            let extensions = gl::GetString(gl::EXTENSIONS);
            if !extensions.is_null() {
                let ext_str = std::ffi::CStr::from_ptr(extensions as *const i8).to_string_lossy();
                println!("OpenGL Extensions: {}", ext_str);
            }
        }
    }
}
