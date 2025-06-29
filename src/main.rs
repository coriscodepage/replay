extern crate sdl3;
extern crate snap;
extern crate regex;

mod file;
mod parser;
mod trace;
mod value_structure;

use sdl3::event::Event;
use sdl3::video::{SwapInterval, Window};
use sdl3::{EventPump, Sdl};
use std::error::Error;
use std::ffi::c_void;
use std::fs::File;
use std::io::{Read, Write};
use std::time::Duration;
use regex::Regex;

use crate::parser::Parser;
use crate::trace::FunctionSignature;

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
    let mut parser = Parser::new("/mnt/glorbus/prog/glReplay/apitrace/hl2.trace").unwrap();
    parser.parse_properties().unwrap();
    let _ = parser.snappy.read_type::<u8>().unwrap();
    let _ = parser.snappy.read_varint().unwrap();
    println!("{:?} | derived API: {:?}", parser.parse_function_sig().unwrap(), parser.api);
    let _ = parser.snappy.read_type::<u8>().unwrap();
    let _ = parser.snappy.read_varint().unwrap();
    println!("{:?} | derived API: {:?}", parser.parse_function_sig().unwrap(), parser.api);

    /*let mut snappy = SnappyFile::new("/mnt/glorbus/prog/glReplay/apitrace/hl2.trace").unwrap();s
    for _ in 0..100 {
        snappy.read_type::<u32>().unwrap();
        println!("oby: 0x{:x}", snappy.read_type::<u32>().unwrap());
    }*/
    //println!("waaawd: {}", snappy.read_varint().unwrap());
    /*let mut snappy_file = File::open("/mnt/glorbus/prog/glReplay/apitrace/hl2.trace").unwrap();
    let mut buf: [u8; 2] = [0; 2];
    snappy_file.read_exact(&mut buf).unwrap();
    match str::from_utf8(&buf) {
        Ok(header) => {
            println!("header: {:0}", header)
        }
        Err(_) => {}
    }
    let mut chunk_len = 0;
    let mut buf2: [u8; 4] = [0; 4];

    println!("length: {:0}", chunk_len);*/
    //let stdout = io::stdout();

    // Wrap the stdin reader in a Snappy reader.
    //let mut rdr = snap::read::FrameDecoder::new();
    //let mut wtr = stdout.lock();
    //io::copy(&mut rdr, &mut wtr).expect("I/O operation failed");
}

pub fn mainer() {
    let mut sdl_ctx = SdlContext::new("Okienko :3", 800, 600).unwrap();
    unsafe { gl::Viewport(0, 0, 800, 600) };
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

pub fn mainerr() {
    let mut meow = File::open("/mnt/glorbus/prog/glReplay/apitrace/szit.data").unwrap();
    let mut mrrp: Vec<u8> = vec![];
    meow.read_to_end(&mut mrrp).unwrap();

    let langosz = snap::raw::decompress_len(&mrrp).unwrap();
    let mut dekoderito_bombardito = snap::raw::Decoder::new();
    let aaaa = dekoderito_bombardito.decompress_vec(&mrrp).unwrap();
    let mut penis = File::create_new("decomp2.data").unwrap();
    penis.write_all(&aaaa).unwrap();
    penis.sync_all().unwrap();
    println!("Ted Kaczyński miał kutrwa rację: {0:x}", langosz);
}
