use cgmath;
use gfx;
use gfx_scene;
use claymore_scene as cs;
use super::reflect as json;

pub type Scalar = f32;

#[derive(Debug)]
pub enum Error {
    NoCamera,
    MissingNode(String),
    Mesh(String, super::mesh::Error),
    Material(String, super::mat::Error),
}

pub fn load<'a, R: 'a + gfx::Resources, F: 'a + gfx::Factory<R>>(
            raw: json::Scene, context: &mut super::Context<R, F>)
            -> Result<cs::Scene<R, Scalar>, Error> {
    use std::collections::hash_map::{HashMap, Entry};
    fn read_space<S: cgmath::BaseFloat>(space: &json::Space<S>)
                  -> cs::Transform<S> {
        cgmath::Decomposed {
            scale: space.scale,
            rot: {
                let (x, y, z, w) = space.rot;
                cgmath::Quaternion::new(w, x, y, z)
            },
            disp: {
                let (x, y, z) = space.pos;
                cgmath::Vector3::new(x, y, z)
            },
        }
    }
    fn populate_world(world: &mut cs::World<Scalar>,
                      raw_nodes: &[json::Node],
                      parent: cs::space::Parent<cs::Transform<Scalar>>) {
        for n in raw_nodes.iter() {
            let space = read_space(&n.space);
            let nid = world.add_node(n.name.clone(), parent, space);
            populate_world(world, &n.children, cs::space::Parent::Domestic(nid));
        }
    }
    // create world
    let mut world = cs::space::World::new();
    populate_world(&mut world, &raw.nodes, cs::space::Parent::None);
    // read camera
    let camera = {
        use std::num::Float;
        let cam = match raw.cameras.first() {
            Some(c) => c,
            None => return Err(Error::NoCamera),
        };
        let node = match world.find_node(&cam.node) {
            Some(n) => n,
            None => return Err(Error::MissingNode(cam.node.to_string())),
        };
        let (fovx, fovy) = cam.angle;
        let (near, far) = cam.range;
        let proj = cgmath::PerspectiveFov {
            fovy: cgmath::rad(fovy),
            aspect: fovx.tan() / fovy.tan(),
            near: near,
            far: far,
        };
        gfx_scene::Camera {
            name: cam.name.clone(),
            node: node,
            projection: proj,
        }
    };
    // read materials
    let mut material_map: HashMap<String, cs::Material<R>> = HashMap::new();
    // read entities
    let mut scene = gfx_scene::Scene::new(world);
    scene.cameras.push(camera);
    for ent in raw.entities.iter() {
        let node = match scene.world.find_node(&ent.node) {
            Some(n) => n,
            None => return Err(Error::MissingNode(ent.node.clone())),
        };
        let (mesh, mut slice) = match context.request_mesh(&ent.mesh) {
            Ok(success) => success,
            Err(e) => return Err(Error::Mesh(ent.mesh.clone(), e)),
        };
        let (ra, rb) = ent.range;
        slice.start = ra as gfx::VertexCount;
        slice.end = rb as gfx::VertexCount;
        let material = match material_map.entry(ent.material.clone()) {
            Entry::Occupied(m) => m.get().clone(),
            Entry::Vacant(v) => match raw.materials.iter().find(|r| r.name == ent.material) {
                Some(raw_mat) => match super::mat::load(&raw_mat, context) {
                    Ok(m) => v.insert(m).clone(),
                    Err(e) => return Err(Error::Material(ent.material.clone(), e)),
                },
                None => return Err(Error::Material(
                    ent.material.clone(), super::mat::Error::NotFound)),
            },
        };
        let bound_min = cgmath::Point3::new(-100.0, -100.0, -100.0);
        let bound_max = cgmath::Point3::new(1000.0, 1000.0, 1000.0);
        scene.entities.push(gfx_scene::Entity {
            name: ent.mesh.clone(),
            material: material,
            mesh: mesh,
            slice: slice,
            node: node,
            skeleton: None, //TODO
            bound: cgmath::Aabb3::new(bound_min, bound_max), //TODO
        });
    }
    // done
    Ok(scene)
}
