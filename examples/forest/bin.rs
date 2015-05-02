extern crate env_logger;
extern crate cgmath;
extern crate glutin;
extern crate gfx;
extern crate gfx_pipeline;
extern crate gfx_window_glutin;
extern crate claymore_load;
extern crate claymore_scene;


fn main() {
    use std::env;
    use gfx::traits::*;
    use gfx_pipeline::Pipeline;

    env_logger::init().unwrap();
    println!("Creating the window...");

    let window = glutin::WindowBuilder::new()
        .with_title("Forest generator".to_string())
        .with_vsync()
        .with_gl(glutin::GL_CORE)
        .build().unwrap();
    let mut canvas = gfx_window_glutin::init(window).into_canvas();

    println!("Loading asset palette...");
    let mut scene = claymore_load::Context::new(
        &mut canvas.factory,
        env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string())
    ).load_scene("data/valefor").unwrap();

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
