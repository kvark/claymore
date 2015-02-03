use std::collections::HashMap;

pub type Scalar = f32;

#[derive(RustcDecodable)]
pub struct Scene {
    pub global: Global,
    pub nodes: Vec<Node>,
    pub materials: Vec<Material>,
    pub entities: Vec<Entity>,
    pub cameras: Vec<Camera>,
    pub lights: Vec<Light>,
}

#[derive(RustcDecodable)]
pub struct Global {
    pub gravity: (f32, f32, f32),
}

#[derive(RustcDecodable)]
pub struct Node {
    pub name: String,
    pub space: Space<Scalar>,
    pub children: Vec<Node>,
    pub actions: Vec<Action>,
}

#[derive(RustcDecodable)]
pub struct Space<S> {
    pub pos: (S, S, S),
    pub rot: (S, S, S, S),
    pub scale: S,
}

#[derive(RustcDecodable)]
pub struct Entity {
    pub node: String,
    pub mesh: String,
    pub range: (u32, u32),
    pub armature: String,
    pub material: String,
    pub actions: Vec<Action>,
}

#[derive(RustcDecodable)]
pub struct Light {
    pub name: String,
    pub node: String,
    pub kind: String,
    pub color: (f32, f32, f32),
    pub energy: f32,
    pub distance: f32,
    pub attenuation: (f32, f32),
    pub spherical: bool,
    pub parameters: Vec<f32>,
    pub actions: Vec<Action>,
}

#[derive(RustcDecodable)]
pub struct Camera {
    pub name: String,
    pub node: String,
    pub angle: (f32, f32),
    pub range: (f32, f32),
    pub actions: Vec<Action>,
}

#[derive(RustcDecodable)]
pub struct Material {
    pub name: String,
    pub shader: String,
    pub data: HashMap<String, Data>,
    pub textures: Vec<Texture>,
}

pub type Data = (String, Vec<f32>);
pub type Texture = ();  //TODO
pub type Action = ();   //TODO
