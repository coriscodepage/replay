extern crate regex;
extern crate sdl3;
extern crate snap;

mod call;
mod file;
mod parser;
mod retracer;
mod signatures;
mod test;
mod trace;
#[path = "../helpers/try.rs"]
mod r#try;
mod value_structure;

mod gl_context;
mod region;

use sdl3::event::Event;
use sdl3::video::{SwapInterval, Window};
use sdl3::{EventPump, Sdl};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::c_void;
use std::time::Duration;
use std::rc::Rc;

use crate::call::Call;
use crate::parser::Parser;
use crate::retracer::Retracer;
use crate::test::SdlContext;
use crate::r#try::GlRetracer;
use crate::value_structure::Value;

fn parse_value(value: &dyn Any) -> String {
    let mut current_value_string = String::new();
    match value {
        any if any.is::<value_structure::None>() => current_value_string.push_str("None"),
        any if any.is::<value_structure::Bool>() => current_value_string.push_str(&format!(
            "{}",
            any.downcast_ref::<value_structure::Bool>()
                .unwrap()
                .to_u32()
                .unwrap()
        )),
        any if any.is::<value_structure::Array>() => {
            let array = any.downcast_ref::<value_structure::Array>().unwrap();
            if array.values.len() == 1 {
                current_value_string
                    .push_str(&format!("&{}", parse_value(array.values[0].as_any())));
            } else {
                current_value_string.push_str("{ ");
                let mut separator = "";
                for val in &array.values {
                    current_value_string
                        .push_str(&format!("{separator}{}", parse_value(val.as_any())));
                    separator = ", ";
                }
                current_value_string.push_str(" }");
            }
        }
        any if any.is::<value_structure::Struct>() => {
            let stru = any.downcast_ref::<value_structure::Struct>().unwrap();
            current_value_string.push_str("{ ");
            let mut separator = "";
            for i in 0..stru.sig.member_names.len() {
                current_value_string.push_str(&format!(
                    "{separator}{} = {}",
                    stru.sig.member_names[i],
                    parse_value(stru.members[i].as_any())
                ));
                separator = ", ";
            }
            current_value_string.push_str(" }");
        }
        any if any.is::<value_structure::Bitmask>() => current_value_string.push_str(&format!(
            "Bitmask({})",
            any.downcast_ref::<value_structure::Bitmask>()
                .unwrap()
                .value
        )),
        any if any.is::<value_structure::Blob>() => current_value_string.push_str(&format!(
            "Blob({})",
            any.downcast_ref::<value_structure::Blob>().unwrap().size
        )),
        any if any.is::<value_structure::Double>() => current_value_string.push_str(&format!(
            "{:.2}",
            any.downcast_ref::<value_structure::Double>().unwrap().value
        )),
        any if any.is::<value_structure::Float>() => current_value_string.push_str(&format!(
            "{:.2}",
            any.downcast_ref::<value_structure::Float>().unwrap().value
        )),
        any if any.is::<value_structure::Enum>() => current_value_string.push_str(&format!(
            "Enum({})",
            any.downcast_ref::<value_structure::Enum>().unwrap().value
        )),
        any if any.is::<value_structure::U32>() => current_value_string.push_str(&format!(
            "{}",
            any.downcast_ref::<value_structure::U32>().unwrap().value
        )),
        any if any.is::<value_structure::I32>() => current_value_string.push_str(&format!(
            "{}",
            any.downcast_ref::<value_structure::I32>().unwrap().value
        )),
        any if any.is::<value_structure::Pointer>() => current_value_string.push_str(&format!(
            "0x{:x}",
            any.downcast_ref::<value_structure::Pointer>()
                .unwrap()
                .value as usize
        )),
        any if any.is::<value_structure::VString>() => current_value_string.push_str(
            &any.downcast_ref::<value_structure::VString>()
                .unwrap()
                .value,
        ),
        _ => current_value_string.push_str("UNKNOWN"),
    }
    current_value_string
}

fn dump_call(call: &mut Call) {
    let mut argument_string = String::new();
    for i in 0..call.sig.num_args {
        let mut padding = "";
        if !argument_string.is_empty() {
            padding = ", ";
        }
        argument_string.push_str(&format!(
            "{padding}{} = {}",
            call.sig.arg_names[i].clone(),
            parse_value(call.arg(i).as_any())
        ));
    }
    if let Some(ret) = &call.ret {
        println!(
            "{}: {}({}) = {}",
            call.number,
            call.sig.name,
            argument_string,
            parse_value(ret.as_any())
        );
    } else {
        println!("{}: {}({})", call.number, call.sig.name, argument_string);
    }
}

struct xsdl {
    windows: HashMap<u32, Rc<RefCell<SdlContext>>>,
    contexts: HashMap<u32, Rc<RefCell<gl_context::Context>>>,
    event_pump: Option<Rc<RefCell<EventPump>>>,
}
impl xsdl {
    fn create_dud(&mut self) -> u32 {
        let mut window = test::SdlContext::new("hidden", 1, 1, sdl3::init().unwrap()).unwrap();
        window.window.hide();
        self.event_pump = Some(window.get_event_pump().unwrap());
        self.contexts.insert(1, Rc::new(RefCell::new(gl_context::Context::new(window.gl_context.clone()))));
        self.windows.insert(1, Rc::new(RefCell::new(window)));
        
        1
    }
    fn glXCreateWindow(&mut self, call: &mut Call) -> u32{
        let mut window = test::SdlContext::new("Window", 800, 600, sdl3::init().unwrap()).unwrap();
        window.window.show();
        window.event_pump = self.event_pump.clone();
        let key = call.arg(0).to_u32().unwrap();
        self.contexts.insert(key, Rc::new(RefCell::new(gl_context::Context::new(window.gl_context.clone()))));
        self.windows.insert(key, Rc::new(RefCell::new(window)));
        key
    }
    fn glXCreateContext(&mut self, call: &mut Call) {
        if self.windows.get(&call.arg(0).to_u32().unwrap()).is_none() {
        self.glXCreateWindow(call);
        }
    }
}

pub fn main() {

    let mut calls = Vec::<Call>::new();
    //test::test();
    let mut sdl = Rc::new(sdl3::init().unwrap());
    //unsafe { gl::Viewport(0, 0, 800, 600) };
    //sdl_ctx.window.gl_make_current(&sdl_ctx.gl_context).unwrap();
    //sdl_ctx.window.show();
    let mut xw = xsdl{ windows: HashMap::with_capacity(2), contexts: HashMap::with_capacity(2), event_pump: None };
    /*'running: loop {
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
    }*/
    let cw = xw.create_dud();
    
    let mut parser = Parser::new("../apitrace/hl2.trace").unwrap();
;
    let mut retracer = Retracer::init(xw.contexts.get(&cw).unwrap().clone());
    for (n, e) in r#try::gl_callbacks {
        retracer.add_callback(retracer::Entry {
            name: n.to_owned(),
            callback: e,
        });
    }
    parser.parse_properties().unwrap();
    //println!("{:?}", parser.properties);
    //panic!();
    let mut sdl_ctx = xw.windows.get(&cw).unwrap().clone();
    sdl_ctx.borrow().window.gl_make_current(&sdl_ctx.borrow().gl_context).unwrap();
    'running: for _ in 0..100000000 {
        match parser.parse_call() {
            Ok(mut call) => {
                dump_call(&mut call);

                /*if call.sig.name == "glClear" {
                    sdl_ctx.window.gl_swap_window();
                    std::thread::sleep(Duration::from_millis(16));
                }*/
                if call.sig.name == "glXCreateWindow" {
                    xw.glXCreateWindow(&mut call);
                } else if call.sig.name == "glXCreateContext" || call.sig.name == "glXCreateNewContext" {
                    xw.glXCreateContext(&mut call);
                }
                
                else if call.sig.name == "glXMakeCurrent" {
                    let mut vao: u32 = 0;
                    //panic!();
                    /*unsafe {
                        gl::GenVertexArrays(1, &mut vao);
                        gl::BindVertexArray(vao);
                    }*/
                    sdl_ctx = xw.windows.get_mut(&call.arg(0).to_u32().unwrap()).unwrap().clone();
                    sdl_ctx.borrow().window.gl_make_current(&sdl_ctx.borrow().gl_context).unwrap();
                    std::thread::sleep(Duration::from_millis(16 / 2));
                } else if call.sig.name == "glXSwapBuffers" {
                    sdl_ctx.borrow().window.gl_swap_window();
                    std::thread::sleep(Duration::from_millis(16 / 2));
                } else if call.sig.name == "memcpy" {
                    region::retrace_memcpy(&mut call);
                } else {
                    match retracer.retrace(&mut call) {
                        Ok(_) => {
                           // println!("{}", call.sig.name);
                        }
                        Err(err) => {
                            //println!("nope: {}", call.sig.name)
                            //eprintln!("error: {}", err)
                        }
                    }
                    //calls.push(call);
                }
            }
            Err(_) => {}
        }
        
        for event in sdl_ctx.borrow_mut().event_pump.as_ref().unwrap().borrow_mut().poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        /*match parser.parse_call() {

            Ok(mut val) => {println!("Call tracing: {}.", val.sig.name); match retracer.retrace(&mut val) {
                Ok(_) => println!("Call: {} retraced.", val.sig.name),
                Err(err) => {
                    eprintln!("error: {}", err)
                }
            }},
            Err(err) => {} //eprintln!("{}", err); panic!()}
        }; */
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
