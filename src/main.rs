extern crate regex;
extern crate sdl3;
extern crate snap;

mod file;
mod parser;
mod trace;
mod value_structure;
mod call;
mod signatures;

use sdl3::event::Event;
use sdl3::video::{SwapInterval, Window};
use sdl3::{EventPump, Sdl};
use std::error::Error;
use std::ffi::c_void;

use crate::parser::Parser;

pub struct SdlContext {
    pub sdl: Sdl,
    pub window: Window,
    pub gl_context: sdl3::video::GLContext,
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
            gl_context,
            gl,
            event_pump,
        })
    }
}

pub fn main() {
    let mut parser = Parser::new("../apitrace/hl2.trace").unwrap();
    parser.parse_properties().unwrap();
    for _ in 0..1000{
        let call = match parser.parse_call() {
            Ok(val) => val,
            Err(err) => {eprintln!("{}", err); panic!()}
        };
        println!("Parsed call: {:?}", call);
    }
    /*parser.parse_properties().unwrap();
        let _ = parser.snappy.read_type::<u8>().unwrap();
        let _ = parser.snappy.read_varint().unwrap();
        println!("{:?} | derived API: {:?}", parser.parse_function_sig().unwrap(), parser.api);
        let _ = parser.snappy.read_type::<u8>().unwrap();
        let _ = parser.snappy.read_varint().unwrap();
        println!("{:?} | derived API: {:?}", parser.parse_function_sig().unwrap(), parser.api);
    */
}
