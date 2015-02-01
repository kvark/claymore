#![crate_name = "blade"]
#![crate_type = "lib"]
#![feature(plugin)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate cgmath;
extern crate gfx;
#[macro_use]
#[plugin]
extern crate gfx_macros;

pub mod draw;
mod load;
pub mod scene;
pub mod space;

pub struct Id<T>(usize);
