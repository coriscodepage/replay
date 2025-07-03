extern crate regex;
extern crate sdl3;
extern crate snap;

mod file;
mod parser;
mod trace;
mod value_structure;
mod call;
mod signatures;
mod retracer;
mod test;
#[path ="../helpers/try.rs"]
mod r#try;

mod gl_context;

use sdl3::event::Event;
use sdl3::video::{SwapInterval, Window};
use sdl3::{EventPump, Sdl};
use std::error::Error;
use std::ffi::c_void;

use crate::parser::Parser;
use crate::retracer::Retracer;



pub fn main() {
    //test::test();
    /*let mut parser = Parser::new("../apitrace/hl2.trace").unwrap();
    let mut retracer = Retracer::init();
    parser.parse_properties().unwrap();
    for _ in 0..15000{
        match parser.parse_call() {
            Ok(mut val) => match retracer.retrace(&mut val) {
                Ok(_) => println!("Call: {} retraced.", val.sig.name),
                Err(err) => {}//eprintln!("error: {}", err),
            }
            Err(err) => {}//eprintln!("{}", err); panic!()}
        };

    }*/
    /*parser.parse_properties().unwrap();
        let _ = parser.snappy.read_type::<u8>().unwrap();
        let _ = parser.snappy.read_varint().unwrap();
        println!("{:?} | derived API: {:?}", parser.parse_function_sig().unwrap(), parser.api);
        let _ = parser.snappy.read_type::<u8>().unwrap();
        let _ = parser.snappy.read_varint().unwrap();
        println!("{:?} | derived API: {:?}", parser.parse_function_sig().unwrap(), parser.api);
    */
}
