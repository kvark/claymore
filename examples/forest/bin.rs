extern crate env_logger;
extern crate rustc_serialize;
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
    //TODO

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
