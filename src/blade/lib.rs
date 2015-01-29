#![crate_name = "blade"]
#![crate_type = "lib"]

extern crate cgmath;

mod draw;
mod space;

pub struct Id<T>(usize);
