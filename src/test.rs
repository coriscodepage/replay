use std::{error::Error, ffi::c_void, ptr::{self, null_mut}, rc::Rc, time::Duration};

use gl::types::{GLenum, GLint};
use sdl3::{
    EventPump, Sdl,
    event::Event,
    video::{SwapInterval, Window},
};

use crate::{call::Call, gl_context, retracer::Callback, value_structure::{Array, Value}};
use bumpalo::Bump;

pub struct ScopedAllocator {
    pub bump: Bump,
}

impl ScopedAllocator {
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    pub fn alloc_array<T: Default>(&mut self, value: &Box<dyn Value>) -> &mut [T] {
        match value.to_array() {
            Some(array) => {
                let num_elems = array.values.len();
                self.bump.alloc_slice_fill_default::<T>(num_elems)
            }
           None => panic!(""),//TODO: make this better,
            _ => {
                panic!("alloc_array: unexpected value type");
            }
        }
    }
}

pub struct SdlContext {
    pub sdl: Sdl,
    pub window: Window,
    pub gl_context: Rc<sdl3::video::GLContext>,
    pub gl: (),
    pub event_pump: EventPump,
}

impl SdlContext {
    pub fn new(title: &str, width: u32, height: u32) -> Result<Self, Box<dyn Error>> {
        let sdl = sdl3::init()?;
        let video_sb = sdl.video()?;
        let gl_attrs = video_sb.gl_attr();
        gl_attrs.set_context_profile(sdl3::video::GLProfile::Core);
        gl_attrs.set_context_version(3, 3);

        let window = video_sb
            .window(title, width, height)
            .resizable()
            .opengl()
            .build()?;
        let gl_context = window.gl_create_context()?;
        let gl = gl::load_with(|s| {
            video_sb
                .gl_get_proc_address(s)
                .map_or(std::ptr::null(), |p| p as *const c_void)
        });

        window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::VSync)?;
        let event_pump = sdl.event_pump()?;
        Ok(Self {
            sdl,
            window,
            gl_context: Rc::new(gl_context),
            gl,
            event_pump,
        })
    }
}

pub fn test() {
    let mut sdl_ctx = SdlContext::new("Okienko :3", 800, 600).unwrap();
    unsafe { gl::Viewport(0, 0, 800, 600) };
    let mut glon = gl_context::Context::new(Rc::clone(&sdl_ctx.gl_context));
    sdl_ctx.window.gl_make_current(&sdl_ctx.gl_context).unwrap();
    sdl_ctx.window.show();
    'running: loop {
        for event in sdl_ctx.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }
        unsafe {
            gl::ClearColor(0.2, 0.1, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        sdl_ctx.window.gl_swap_window();
        std::thread::sleep(Duration::from_millis(16));
    }
}

/*
struct GlRetrace_ {
    context: SdlContext,
}

impl GlRetrace_ {
    pub fn retrace_gl_front_face(&mut self, call: &mut Call) {
        let current_context = &self.context; // your tracking logic

        let mode: GLenum = if let Some(call) = call.arg(0).to_u32() {
            call as GLenum
        } else {
            0
        };
        let error_position: GLint = -1;
        unsafe { gl::GetIntegerv(gl::GL_PROGRAM_ERROR_POSITION_ARB, &mut error_position) };
        unsafe {
            gl::FrontFace(mode);
        }

        //if debug_enabled(current_context) {
        //    check_gl_error(call);
        //}
    }
}

pub fn retrace_gl_get_query_objectiv(&mut self, call: &trace::Call) {
    let current_context = glretrace::get_current_context();

    let mut query_buffer: GLint = 0;
    if let Some(ctx) = &current_context {
        if ctx.features().query_buffer_object {
            unsafe {
                gl::GetIntegerv(gl::QUERY_BUFFER_BINDING, &mut query_buffer);
            }
        }
    }

    if query_buffer == 0 && retrace::QUERY_HANDLING == retrace::QueryHandling::Skip {
        return;
    }

    // Simulate a scope-bound allocator (noop here)
    let _allocator = retrace::ScopedAllocator;
    let edew: [(&'static str, Callback); 1] = [("dawdawd", self.retrace_gl_front_face)];
    let mut id = call.arg(0).to_uint();

    id = retrace::_query_map()[&id];
    let pname = call.arg(1).to_sint() as GLenum;

    let mut retval: GLint = 0;
    let params_ptr: *mut GLint = if query_buffer != 0 {
        call.arg(2).to_pointer() as *mut GLint
    } else {
        &mut retval
    };

    unsafe {
        gl::GetQueryObjectiv(id, pname, params_ptr);
    }

    // Extract expected result from trace
    let result_array = call.arg(2).to_array();
    if let Some(array) = result_array {
        for value in &array.values {
            // Here you could log or use the value
        }

        if query_buffer == 0 && retrace::QUERY_HANDLING != retrace::QueryHandling::Skip {
            assert_eq!(array.values.len(), 1);
            let expect = array.values[0].to_uint() as GLint;

            if call.arg(1).to_uint() == gl::QUERY_RESULT_AVAILABLE as u64 {
                if expect == 1 && retval == 0 {
                    // recurse to retry
                    return retrace_gl_get_query_objectiv(call);
                }
            } else if retrace::QUERY_HANDLING == retrace::QueryHandling::RunAndCheckResult
                && (expect - retval).abs() > retrace::QUERY_TOLERANCE
            {
                retrace::warning(
                    call,
                    &format!(
                        "Warning: query returned {} but trace contained {} (tol = {})",
                        retval,
                        expect,
                        retrace::QUERY_TOLERANCE
                    ),
                );
            }
        }
    }
}
*/