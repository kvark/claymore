#![crate_name = "blade"]
#![crate_type = "lib"]
#![feature(plugin)]

extern crate cgmath;
extern crate gfx;
#[macro_use]
#[plugin]
extern crate gfx_macros;

pub mod draw;
pub mod scene;
pub mod space;

pub struct Id<T>(usize);
