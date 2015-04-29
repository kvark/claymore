#![feature(custom_attribute, plugin)]
#![plugin(gfx_macros)]

extern crate rustc_serialize;
#[macro_use]
extern crate log;
extern crate grid;
extern crate cgmath;
extern crate gfx;
extern crate gfx_scene;
extern crate gfx_pipeline;
extern crate claymore_scene as scene;
extern crate claymore_load as load;

use std::fs::File;
use rustc_serialize::json;
use gfx_pipeline::forward::Pipeline;

mod field;
mod reflect;


fn convert_dir(d: reflect::Direction) -> field::Direction {
    use reflect::Direction as A;
    use field::Direction as B;
    match d {
        A::North => B::North,
        A::East => B::East,
        A::South => B::South,
        A::West => B::West,
    }
}

struct Character {
    _name: String,
    team: u8,
    cell: field::Coordinate,
    node: scene::NodeId<load::Scalar>,
}

pub struct App<R: gfx::Resources> {
    scene: scene::Scene<R, load::Scalar>,
    camera: scene::Camera<load::Scalar>,
    pipeline: Pipeline<R>,
    field: field::Field<R>,
    characters: Vec<Character>,
}

impl<R: gfx::Resources> App<R> {
    pub fn new<F: gfx::Factory<R>>(factory: &mut F) -> App<R> {
        use std::env;
        use std::io::Read;
        let root = env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string());
        let mut scene = load::create_scene();
        // load the config
        let config: reflect::Game = {
            let mut file = File::open(&format!("{}/config/game.json", root)).unwrap();
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            json::decode(&s).unwrap()
        };
        // create the grid
        let field_node = scene.world.add_node(
            "Field".to_string(),
            scene::space::Parent::None,
            cgmath::Transform::identity()
        );
        let field = field::Field::new(factory, field_node,
            config.level.grid.size,
            config.level.grid.area,
            config.level.grid.color);
        // load the scene
        let mut characters = Vec::new();
        {
            let mut context = load::Context::new(factory, root);
            context.extend_scene(&mut scene, &config.level.scene).unwrap();
            for (name, ch) in config.level.characters.iter() {
                let coord = [ch.cell.0 as i32, ch.cell.1 as i32];
                let cur_dir = convert_dir(ch.cell.2);
                match config.characters.get(name) {
                    Some(desc) => {
                        use grid::Grid2;
                        use cgmath::{Point, ToRad};
                        context.alpha_test = Some(desc.alpha_test);
                        let nid = context.extend_scene(&mut scene, &desc.scene).unwrap();
                        let node = scene.world.mut_node(nid);
                        let angle = field.grid.get_angle(convert_dir(desc.direction), cur_dir);
                        node.parent = scene::space::Parent::Domestic(field.node);
                        node.local = cgmath::Decomposed {
                            rot: cgmath::Rotation3::from_axis_angle(
                                &cgmath::Vector3::new(0.0, 0.0, -1.0),
                                cgmath::deg(angle * 180.0).to_rad()
                            ),
                            scale: ch.scale,
                            disp: field.get_center(coord).to_vec(),
                        };
                        characters.push(Character {
                            _name: name.clone(),
                            team: ch.team,
                            cell: coord,
                            node: nid,
                        });
                    },
                    None => {
                        error!("Unable to find character: {}", name);
                    },
                }
            }
        };
        // create the pipeline
        let mut pipeline = Pipeline::new(factory).unwrap();
        pipeline.background = Some([0.2, 0.3, 0.4, 1.0]);
        // done
        let camera = scene.cameras[0].clone();
        App {
            scene: scene,
            camera: camera,
            pipeline: pipeline,
            field: field,
            characters: characters,
        }
    }

    fn mouse_cast(&self, x: f32, y: f32) -> field::Coordinate {
        use cgmath::{EuclideanVector, Matrix, Point, ToMatrix4, Transform};
        use scene::base::World;
        let end_proj = cgmath::Point3::new(x*2.0 - 1.0, 1.0 - y*2.0, 0.0);
        let mx_proj = self.camera.projection.to_matrix4().invert().unwrap();
        let end_cam = cgmath::Point3::from_homogeneous(
            &mx_proj.mul_v(&end_proj.to_homogeneous())
        );
        let ray = cgmath::Ray3::new(cgmath::Point3::new(0.0, 0.0, 0.0),
                                    end_cam.to_vec().normalize());
        let transform = self.scene.world.get_transform(&self.field.node)
                            .invert().unwrap()
                            .concat(&self.scene.world.get_transform(&self.camera.node));
        let ray_grid = transform.transform_ray(&ray);
        self.field.cast_ray(&ray_grid)
    }

    pub fn mouse_click(&mut self, x: f32, y: f32) {
        use std::collections::HashMap;
        let cell = self.mouse_cast(x, y);
        info!("[click] on {:?}", cell);
        let mut cell_map = HashMap::new();
        for ch in self.characters.iter() {
            cell_map.insert(ch.cell, ch.team);
        }
        let player = match self.characters.iter_mut().find(|c| c.team == 0) {
            Some(p) => p,
            None => {
                info!("[click] no playable character");
                return
            },
        };
        match cell_map.get(&cell) {
            Some(team) if *team == player.team => {
                info!("[click] aid ally");
            },
            Some(_team) => {
                info!("[click] attack");
            },
            None => { //move
                use cgmath::Point;
                info!("[click] move");
                player.cell = cell;
                let node = self.scene.world.mut_node(player.node);
                node.local.disp = self.field.get_center(cell).to_vec();
            },
        }
    }

    pub fn rotate_camera(&mut self, degrees: f32) {
        use cgmath::{ToRad, Transform};
        let rotation = cgmath::Decomposed {
            scale: 1.0,
            rot: cgmath::Rotation3::from_axis_angle(
                &cgmath::vec3(0.0, 0.0, 1.0),
                cgmath::deg(degrees).to_rad()
            ),
            disp: cgmath::zero(),
        };
        let transform = &mut self.scene.world.mut_node(self.camera.node).local;
        *transform = rotation.concat(transform);
    }

    pub fn render<S: gfx::Stream<R>>(&mut self, stream: &mut S) {
        use gfx_pipeline::Pipeline;
        self.scene.world.update();
        self.camera.projection.aspect = stream.get_aspect_ratio();
        self.pipeline.render(&self.scene, &self.camera, stream).unwrap();
        self.field.update_params(&self.camera, &self.scene.world);
        self.field.draw(stream);
    }
}
