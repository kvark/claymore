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
    pub gravity: (Scalar, Scalar, Scalar),
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
    pub color: (Scalar, Scalar, Scalar),
    pub energy: Scalar,
    pub distance: Scalar,
    pub attenuation: (Scalar, Scalar),
    pub spherical: bool,
    pub parameters: Vec<Scalar>,
    pub actions: Vec<Action>,
}

#[derive(RustcDecodable)]
pub struct Camera {
    pub name: String,
    pub node: String,
    pub angle: (Scalar, Scalar),
    pub range: (Scalar, Scalar),
    pub actions: Vec<Action>,
}

#[derive(RustcDecodable)]
pub struct Material {
    pub name: String,
    pub shader: String,
    pub transparent: bool,
    pub data: HashMap<String, Data>,
    pub textures: Vec<Texture>,
}

pub type Data = (String, Vec<f32>);

#[derive(RustcDecodable)]
pub struct Texture {
    pub name: String,
    pub image: Image,
    pub filter: u8,
    pub wrap: (i8, i8, i8),
    pub offset: (Scalar, Scalar, Scalar),
    pub scale: (Scalar, Scalar, Scalar),
}

#[derive(RustcDecodable)]
pub struct Image {
    pub path: String,
    pub space: String,
    pub mapping: String,
}

pub type Action = ();   //TODO
