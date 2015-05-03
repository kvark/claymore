#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rustc_serialize;
extern crate rand;
extern crate cgmath;
extern crate glutin;
extern crate gfx;
extern crate gfx_pipeline;
extern crate gfx_window_glutin;
extern crate claymore_load;
extern crate claymore_scene;

mod reflect;

struct TileInfo<R: gfx::Resources> {
    mesh: gfx::Mesh<R>,
    slice: gfx::Slice<R>,
    material: gfx_pipeline::Material<R>,
    river_mask: u8,
}

impl<R: gfx::Resources> TileInfo<R> {
    pub fn fit_orientation(&self, neighbors: u8, rivers: u8) -> Option<u8> {
        let mut mask = self.river_mask;
        for i in 0u8.. 4 {
            if (mask & neighbors) == rivers {
                return Some(i)
            }
            mask = (mask >> 1) | ((mask & 1) << 3);
        }
        None
    }
}


fn main() {
    use std::env;
    use gfx::traits::*;
    use gfx_pipeline::Pipeline;

    env_logger::init().unwrap();
    let root = env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string());

    println!("Reading configuration...");
    let config: reflect::Demo = {
        use std::fs::File;
        use std::io::Read;
        use rustc_serialize::json;
        let mut file = File::open(&format!("{}/examples/forest/config.json", root))
                            .unwrap();
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();
        json::decode(&s).unwrap()
    };

    println!("Creating the window...");
    let window = glutin::WindowBuilder::new()
        .with_title(config.name)
        .with_vsync()
        .with_gl(glutin::GL_CORE)
        .build().unwrap();
    let mut canvas = gfx_window_glutin::init(window).into_canvas();

    println!("Loading asset palette...");
    let mut scene = claymore_load::Context::new(&mut canvas.factory, root)
                                           .load_scene(&config.palette.scene)
                                           .unwrap();
    for ent in scene.entities.iter_mut() {
        ent.visible = false;
    }

    println!("Processing data...");
    let tile_info: Vec<_> = config.palette.tiles.iter().map(|t| {
        let entity = scene.entities.iter().find(|ent| ent.name == t.name)
                          .expect(&format!("Unable to find tile {:?}", t.name));
        let rmask = t.river.chars().fold(0, |m, c| match c {
            'n' => m | 1,
            'e' => m | 2,
            's' => m | 4,
            'w' => m | 8,
            _   => panic!("Unknown river direction: {}", c),
        });
        TileInfo {
            mesh: entity.mesh.clone(),
            slice: entity.slice.clone(),
            material: entity.material.clone(),
            river_mask: rmask,
        }
    }).collect();

    println!("Generating content...");
    {
        use std::collections::HashMap;
        type Position = (i32, i32);
        struct Tile {
            info_id: usize,
            orientation: u8,
        }
        let mut tile_map: HashMap<Position, Tile> = HashMap::new();
        for y in -10i32 ..10 {
            for x in -10i32 ..10 {
                if tile_map.contains_key(&(x,y)) {
                    continue
                }
                // figure out what neighbour edges are rivers
                let mut river_mask = 0;
                let mut neighbour_mask = 0;
                let offsets = [[0, 1], [1, 0], [0,-1], [-1, 0]];
                for (bit, off) in offsets.iter().enumerate() {
                    let pos = (x + off[0], y + off[1]);
                    if let Some(tile) = tile_map.get(&pos) {
                        neighbour_mask |= 1 << bit;
                        let river_bit = (tile.orientation + 2) & 3;
                        let info = &tile_info[tile.info_id];
                        if info.river_mask & river_bit != 0 {
                            river_mask |= 1 << bit;
                        }
                    }
                }
                // find a matching prototype
                let mut matched = 0;
                for info in tile_info.iter() {
                    if info.fit_orientation(neighbour_mask, river_mask).is_some() {
                        matched += 1;
                    }
                }
                if matched == 0 {
                    error!("Couldn't find a tile match for {:?}, where neighbors = {}, rivers = {}",
                        (x, y), neighbour_mask, river_mask);
                    continue
                }
                let chosen = matched / 2; //TODO: random
                matched = 0;
                for (id, info) in tile_info.iter().enumerate() {
                    match info.fit_orientation(neighbour_mask, river_mask) {
                        Some(orientation) if matched == chosen => {
                            use cgmath::ToRad;
                            let tile = Tile {
                                info_id: id,
                                orientation: orientation,
                            };
                            let size = config.palette.size;
                            let node = scene.world.add_node(
                                format!("Tile ({}, {})", x, y),
                                claymore_scene::space::Parent::None,
                                cgmath::Decomposed {
                                    scale: 1.0,
                                    rot: cgmath::Rotation3::from_axis_angle(
                                        &cgmath::Vector3::new(1.0, 0.0, 0.0),
                                        cgmath::deg(orientation as f32 * 90.0).to_rad(),
                                    ),
                                    disp: cgmath::Vector3::new(
                                        x as f32 * size,
                                        y as f32 * size,
                                        0.0,
                                    ),
                                });
                            let entity = claymore_scene::base::Entity {
                                name: String::new(),
                                visible: true,
                                material: info.material.clone(),
                                mesh: info.mesh.clone(),
                                slice: info.slice.clone(),
                                node: node,
                                skeleton: None,
                                bound: cgmath::Aabb3::new(
                                    cgmath::Point3::new(0.0, 0.0, 0.0),
                                    cgmath::Point3::new(size, size, 1.0),
                                ),
                            };
                            scene.entities.push(entity);
                            tile_map.insert((x, y), tile);
                        }
                        Some(_) => {
                            matched += 1;
                        }
                        None => (),
                    }
                }
            }
        }
    }

    println!("Initializing the graphics...");
    let mut pipeline = gfx_pipeline::forward::Pipeline::new(&mut canvas.factory)
                                                       .unwrap();
    pipeline.background = Some([0.2, 0.3, 0.4, 1.0]);

    let mut camera = match scene.cameras.first() {
        Some(cam) => cam.clone(),
        None => {
            println!("No cameras found!");
            return;
        }
    };

    println!("Rendering...");
    'main: loop {
        for event in canvas.output.window.poll_events() {
            use glutin::{Event, VirtualKeyCode};
            match event {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) =>
                    break 'main,
                Event::Closed => break 'main,
                _ => {},
            }
        }

        scene.world.update();

        camera.projection.aspect = canvas.get_aspect_ratio();
        pipeline.render(&scene, &camera, &mut canvas).unwrap();

        canvas.present();
    }
    println!("Done.");
}
