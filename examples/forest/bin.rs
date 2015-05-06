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

mod generate;
mod reflect;


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

    if config.generate {
        let gen = generate::Gen::new(&config.palette, &scene);
        gen.populate(&config.palette.model, &mut scene);
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
