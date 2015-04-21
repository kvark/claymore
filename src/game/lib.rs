#![feature(custom_attribute, plugin)]
#![plugin(gfx_macros)]

extern crate rustc_serialize;
#[macro_use]
extern crate log;
extern crate hex2d;
extern crate cgmath;
extern crate gfx;
extern crate gfx_scene;
extern crate gfx_pipeline;
extern crate claymore_scene as scene;
extern crate claymore_load as load;

use std::fs::File;
use rustc_serialize::json;
use gfx_pipeline::forward::Pipeline;

mod grid;
mod reflect;


struct Character {
    name: String,
    team: u8,
    cell: grid::Coordinate,
    node: scene::NodeId<load::Scalar>,
}

pub struct App<R: gfx::Resources> {
    scene: scene::Scene<R, load::Scalar>,
    camera: scene::Camera<load::Scalar>,
    pipeline: Pipeline<R>,
    grid: grid::Grid<R>,
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
        let grid_node = scene.world.add_node(
            "Grid".to_string(),
            scene::space::Parent::None,
            cgmath::Transform::identity()
        );
        let grid = grid::Grid::new(factory, grid_node,
            config.level.grid.size, config.level.grid.color);
        // load the scene
        let mut characters = Vec::new();
        let texture = {
            let mut context = load::Context::new(factory, root).unwrap();
            context.extend_scene(&mut scene, &config.level.scene).unwrap();
            for (name, ch) in config.level.characters.iter() {
                let coord = hex2d::Coordinate::new(ch.cell.0, ch.cell.1);
                match config.characters.get(name) {
                    Some(desc) => {
                        use cgmath::Point;
                        let nid = context.extend_scene(&mut scene, &desc.scene).unwrap();
                        let node = scene.world.mut_node(nid);
                        node.parent = scene::space::Parent::Domestic(grid.node);
                        node.local.disp = grid.get_center(coord).to_vec();
                        node.local.scale = ch.scale;
                        characters.push(Character {
                            name: name.clone(),
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
            (context.texture_white.clone(), None)
        };
        // create the pipeline
        let mut pipeline = Pipeline::new(factory, texture).unwrap();
        pipeline.background = Some([0.2, 0.3, 0.4, 1.0]);
        // done
        let camera = scene.cameras[0].clone();
        App {
            scene: scene,
            camera: camera,
            pipeline: pipeline,
            grid: grid,
            characters: characters,
        }
    }

    pub fn mouse_click(&mut self, x: f32, y: f32) {
        use std::collections::HashMap;
        use cgmath::{Matrix, Point, ToMatrix4, Transform};
        use scene::base::World;
        let mut cell_map = HashMap::new();
        for ch in self.characters.iter() {
            cell_map.insert(ch.cell, ch.team);
        }
        let player = match self.characters.iter_mut().find(|c| c.team == 0) {
            Some(p) => p,
            None => {
                println!("click: no playable character");
                return
            },
        };
        let end_proj = cgmath::Point3::new(x*2.0 - 0.5, y*2.0 - 0.5, 0.0);
        let mx_proj = self.camera.projection.to_matrix4().invert().unwrap();
        let end_cam = cgmath::Point3::from_homogeneous(
            &mx_proj.mul_v(&end_proj.to_homogeneous())
        );
        let ray = cgmath::Ray3::new(cgmath::Point3::new(0.0, 0.0, 0.0), end_cam.to_vec());
        let transform = self.scene.world.get_transform(&self.grid.node).invert().unwrap()
                            .concat(&self.scene.world.get_transform(&self.camera.node));
        let ray_grid = transform.transform_ray(&ray);
        let cell = match self.grid.cast_ray(&ray_grid) {
            Some(c) => c,
            None => {
                println!("click: invalid cell");
                return
            },
        };
        match cell_map.get(&cell) {
            Some(team) if *team == player.team => {
                println!("click: aid ally");
            },
            Some(_team) => {
                println!("click: attack");
            },
            None => { //move
                println!("click: move");
                player.cell = cell;
                let node = self.scene.world.mut_node(player.node);
                node.local.disp = self.grid.get_center(cell).to_vec();
            },
        }
    }

    pub fn render<C: gfx::CommandBuffer<R>, O: gfx::Output<R>>(
                  &mut self, renderer: &mut gfx::Renderer<R, C>, output: &O) {
        use gfx_pipeline::Pipeline;
        self.scene.world.update();
        self.camera.projection.aspect = {
            let (w, h) = output.get_size();
            w as f32 / h as f32
        };
        self.pipeline.render(&self.scene, renderer, &self.camera, output).unwrap();
        self.grid.update_params(&self.camera, &self.scene.world);
        self.grid.draw(renderer, output);
    }
}
