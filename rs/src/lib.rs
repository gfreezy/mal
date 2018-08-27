#![feature(nll)]
#![feature(slice_concat_ext)]
extern crate rustyline;
#[macro_use]
extern crate failure;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate indextree;
#[macro_use]
extern crate debug_stub_derive;

pub mod core;
pub mod env;
pub mod error;
pub mod printer;
pub mod reader;
pub mod types;
