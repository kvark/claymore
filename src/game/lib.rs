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


pub struct App<R: gfx::Resources> {
    scene: scene::Scene<R, load::Scalar>,
    pipeline: Pipeline<R>,
    grid: grid::Grid<R>,
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
        App {
            scene: scene,
            pipeline: pipeline,
            grid: grid,
        }
    }

    pub fn render<C: gfx::CommandBuffer<R>, O: gfx::Output<R>>(
                  &mut self, renderer: &mut gfx::Renderer<R, C>, output: &O) {
        use gfx_pipeline::Pipeline;
        self.scene.world.update();
        let mut camera = self.scene.cameras[0].clone();
        camera.projection.aspect = {
            let (w, h) = output.get_size();
            w as f32 / h as f32
        };
        self.pipeline.render(&self.scene, renderer, &camera, output).unwrap();
        self.grid.update_params(&camera, &self.scene.world);
        self.grid.draw(renderer, output);
    }
}
