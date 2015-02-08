#![crate_name = "blade"]
#![crate_type = "lib"]
#![feature(collections, core, plugin, io, path)]

#[macro_use]
extern crate log;
extern crate "rustc-serialize" as rustc_serialize;
extern crate cgmath;
extern crate gfx;
#[macro_use]
#[plugin]
extern crate gfx_macros;

pub mod draw;
pub mod load;
pub mod scene;
pub mod space;

pub struct Id<T>(usize);

impl<T> Copy for Id<T> {}
