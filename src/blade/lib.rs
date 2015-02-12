#![crate_name = "blade"]
#![crate_type = "lib"]
#![feature(collections, core, io, plugin, path, rustc_private, unsafe_destructor)]
#![plugin(gfx_macros)]

#[macro_use]
extern crate log;
extern crate "rustc-serialize" as rustc_serialize;
extern crate cgmath;
extern crate gfx;

pub mod draw;
pub mod load;
pub mod render;
pub mod scene;
pub mod space;

pub struct Id<T>(usize);

impl<T> Copy for Id<T> {}
