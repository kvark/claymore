use rustc_serialize::{json, Decodable};
use std::collections::HashMap;
use std::old_io as io;

type Scalar = f32;

#[derive(RustcDecodable)]
struct Scene {
    global: Global,
    nodes: Vec<Node>,
    materials: Vec<Material>,
    entities: Vec<Entity>,
    cameras: Vec<Camera>,
    lights: Vec<Light>,
}

#[derive(RustcDecodable)]
struct Global {
    gravity: (f32, f32, f32),
}

#[derive(RustcDecodable)]
struct Node {
    name: String,
    space: Space<Scalar>,
    children: Vec<Node>,
    actions: Vec<Action>,
}

#[derive(RustcDecodable)]
struct Space<S> {
    pos: (S, S, S),
    rot: (S, S, S, S),
    scale: S,
}

#[derive(RustcDecodable)]
struct Entity {
    mesh: String,
    range: (u32, u32),
    armature: String,
    material: String,
    actions: Vec<Action>,
}

#[derive(RustcDecodable)]
struct Light {
    name: String,
    kind: String,
    color: (f32, f32, f32),
    energy: f32,
    distance: f32,
    attenuation: (f32, f32),
    spherical: bool,
    parameters: Vec<f32>,
    actions: Vec<Action>,
}

#[derive(RustcDecodable)]
struct Camera {
    name: String,
    fov_y: f32,
    range: (f32, f32),
    actions: Vec<Action>,
}

#[derive(RustcDecodable)]
struct Material {
    name: String,
    shader: String,
    data: HashMap<String, Data>,
    textures: Vec<Texture>,
}

type Data = (String, Vec<f32>);
type Texture = ();  //TODO
type Action = ();   //TODO

#[derive(Debug)]
pub enum Error {
    Read(io::IoError),
    Decode(json::DecoderError),
}

pub fn json(path: &str) -> Result<Scene, Error> {
    match io::File::open(&Path::new(path)).read_to_string() {
        Ok(data) => json::decode(data.as_slice()).map_err(|e|
            Error::Decode(e)
        ),
        Err(e) => Err(Error::Read(e)),
    }
}
