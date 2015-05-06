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
extern crate gfx_debug_draw;
extern crate claymore_load;
extern crate claymore_scene;

mod reflect;


struct TileInfo<R: gfx::Resources> {
    node: claymore_scene::NodeId<f32>,
    mesh: gfx::Mesh<R>,
    fragments: Vec<claymore_scene::Fragment<R>>,
    river_mask: u8,
}

impl<R: gfx::Resources> TileInfo<R> {
    pub fn fit_orientation(&self, neighbors: u8, rivers: u8) -> Option<u8> {
        let mut mask = self.river_mask;
        for i in 0u8.. 4 {
            if (mask & neighbors) == rivers {
                return Some(i)
            }
            mask = ((mask << 1) & 0xF) | (mask >> 3);
        }
        None
    }
}

fn move_camera<S: cgmath::BaseFloat>(
    camera: &claymore_scene::Camera<S>,
    vec: &cgmath::Vector3<S>,
    world: &mut claymore_scene::World<S>
){
    use cgmath::{Transform, Vector};
    let node = world.mut_node(camera.node);
    let cam_offset = node.local.transform_vector(vec);
    node.local.disp.add_self_v(&cam_offset);
}


fn main() {
    use std::env;
    use gfx::traits::*;
    use gfx_pipeline::Pipeline;

    env_logger::init().unwrap();
    let root = env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string());

    println!("Creating the window...");
    let window = glutin::WindowBuilder::new()
        .with_title("Forest generator".to_string())
        .with_vsync()
        .with_gl(glutin::GL_CORE)
        .build().unwrap();
    let mut canvas = gfx_window_glutin::init(window).into_canvas();

    let mut debug = gfx_debug_draw::DebugRenderer::from_canvas(
        &mut canvas, 64, None, None).ok().unwrap();

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

    println!("Loading asset palette...");
    let mut scene = claymore_load::Context::new(&mut canvas.factory, root)
                                           .load_scene(&config.palette.scene)
                                           .unwrap();
    scene.world.update();

    println!("Processing data...");
    let tile_info: Vec<_> = config.palette.tiles.iter().map(|t| {
        let mask = t.river.chars().fold(0, |m, c| match c {
            'n' => m | 1,
            'e' => m | 2,
            's' => m | 4,
            'w' => m | 8,
            _   => panic!("Unknown river direction: {}", c),
        });
        let ent = scene.entities.iter()
                                .find(|ent| ent.name == t.name)
                                .expect(&format!("Unable to find entity {}", t.name));
        info!("Found tile {} with river mask {}",
            t.name, mask);
        TileInfo {
            node: ent.node.clone(),
            mesh: ent.mesh.clone(),
            fragments: ent.fragments.clone(),
            river_mask: mask,
        }
    }).collect();

    println!("Generating content...");
    if config.generate {
        use std::collections::HashMap;
        type Position = (i32, i32);
        struct Tile {
            info_id: usize,
            orientation: u8,
        }
        scene.entities.clear();
        let mut rng = rand::thread_rng();
        let mut tile_map: HashMap<Position, Tile> = HashMap::new();
        for y in -config.size.0 ..config.size.0 {
            for x in -config.size.1 ..config.size.1 {
                use rand::Rng;
                if tile_map.contains_key(&(x,y)) {
                    continue
                }
                debug!("Generating tile {:?}", (x,y));
                // figure out what neighbour edges are rivers
                let mut river_mask = 0;
                let mut neighbour_mask = 0;
                let offsets = [[0, 1], [1, 0], [0,-1], [-1, 0]];
                for (bit, off) in offsets.iter().enumerate() {
                    let pos = (x + off[0], y + off[1]);
                    if let Some(tile) = tile_map.get(&pos) {
                        neighbour_mask |= 1 << bit;
                        let river_bit = ((bit as u8) + 6 - tile.orientation) & 3;
                            debug!("\tChecking for river bit {} of neighbor dir {}", river_bit, bit);
                        let info = &tile_info[tile.info_id];
                        if info.river_mask & (1 << river_bit) != 0 {
                            river_mask |= 1 << bit;
                        }
                    }
                }
                debug!("\tLooking for river mask {} of neighbors {}", river_mask, neighbour_mask);
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
                let chosen = rng.gen_range(0, matched);
                debug!("\tChosen match {} of total {}", chosen, matched);
                matched = 0;
                for (id, info) in tile_info.iter().enumerate() {
                    match info.fit_orientation(neighbour_mask, river_mask) {
                        Some(orientation) if matched == chosen => {
                            use cgmath::ToRad;
                            debug!("\tUsing orientation {} and info id {}", orientation, id);
                            let size = config.palette.size;
                            let rotation = {
                                use cgmath::Rotation;
                                use claymore_scene::base::World;
                                let relative: cgmath::Quaternion<_> = cgmath::Rotation3::from_axis_angle(
                                    &cgmath::Vector3::new(0.0, 0.0, -1.0),
                                    cgmath::deg(orientation as f32 * 90.0).to_rad(),
                                );
                                let node = tile_info[id].node;
                                relative.concat(&scene.world.get_transform(&node).rot)
                            };
                            let (rot_x, rot_y) = [(0, 0), (0, 1), (1, 1), (1, 0)][orientation as usize];
                            let node = scene.world.add_node(
                                format!("Tile ({}, {})", x, y),
                                claymore_scene::space::Parent::None,
                                cgmath::Decomposed {
                                    scale: 1.0,
                                    rot: rotation,
                                    disp: cgmath::Vector3::new(
                                        (x + rot_x) as f32 * size,
                                        (y + rot_y) as f32 * size,
                                        0.0,
                                    ),
                                });
                            // add the new entity to the scene
                            scene.entities.push(claymore_scene::base::Entity {
                                name: String::new(),
                                visible: true,
                                mesh: tile_info[id].mesh.clone(),
                                node: node,
                                skeleton: None,
                                bound: cgmath::Aabb3::new(
                                    cgmath::Point3::new(0.0, 0.0, 0.0),
                                    cgmath::Point3::new(size, 0.5, -size),
                                ),
                                fragments: tile_info[id].fragments.clone(),
                            });
                            // register the new tile
                            tile_map.insert((x, y), Tile {
                                info_id: id,
                                orientation: orientation,
                            });
                            break;
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
            // TODO: use the scroll
            use glutin::{Event, VirtualKeyCode};
            match event {
                Event::Closed => break 'main,
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) =>
                    break 'main,
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::A)) =>
                    move_camera(&camera, &cgmath::vec3(-1.0, 0.0, 0.0), &mut scene.world),
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::D)) =>
                    move_camera(&camera, &cgmath::vec3(1.0, 0.0, 0.0),  &mut scene.world),
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::S)) =>
                    move_camera(&camera, &cgmath::vec3(0.0, -1.0, 0.0), &mut scene.world),
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::W)) =>
                    move_camera(&camera, &cgmath::vec3(0.0, 1.0, 0.0),  &mut scene.world),
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::E)) =>
                    move_camera(&camera, &cgmath::vec3(0.0, 0.0, -1.0), &mut scene.world),
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Q)) =>
                    move_camera(&camera, &cgmath::vec3(0.0, 0.0, 1.0),  &mut scene.world),
                _ => {},
            }
        }

        scene.world.update();

        camera.projection.aspect = canvas.get_aspect_ratio();
        let report = pipeline.render(&scene, &camera, &mut canvas).unwrap();

        {
            let win_size = canvas.output.get_size();
            let color = config.debug.color;
            let offset = config.debug.offset;
            let mut offset = [
                if offset.0 < 0 {win_size.0 as i32 + offset.0} else {offset.0},
                if offset.1 < 0 {win_size.1 as i32 + offset.1} else {offset.1},
            ];
            let color = [color.0, color.1, color.2, color.3];
            let voff = 10;
            let strings = [
                format!("ratio = {}",     report.get_ratio()),
                format!("invisible = {}", report.calls_invisible),
                format!("culled = {}",    report.calls_culled),
                format!("rejected = {}",  report.calls_rejected),
                format!("failed = {}",    report.calls_failed),
                format!("passed = {}",    report.calls_passed),
            ];
            for s in strings.iter() {
                debug.draw_text_on_screen(s, offset, color);
                offset[1] += config.debug.line_jump;
            }
        }
        debug.render_canvas(&mut canvas, [[0.0; 4]; 4]);

        canvas.present();
    }
    println!("Done.");
}
