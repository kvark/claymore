extern crate env_logger;
extern crate glutin;
extern crate gfx;
extern crate gfx_phase;
extern crate gfx_scene;
extern crate gfx_device_gl;
extern crate "claymore-load" as load;
extern crate "claymore-scene" as scene;

fn main() {
    use gfx::traits::*;

    env_logger::init().unwrap();
    println!("Initializing the window...");

    let window = glutin::WindowBuilder::new().with_vsync().build().unwrap();
    window.set_title("Claymore");
    unsafe { window.make_current() };
    let mut device = gfx_device_gl::GlDevice::new(|s| window.get_proc_address(s));

    let (w, h) = window.get_inner_size().unwrap();
    let frame = gfx::Frame::new(w as u16, h as u16);
    let mut renderer = device.create_renderer();

    println!("Loading the test scene...");
    let (mut phase, mut scene) = {
        let mut context = load::Context::new(&mut device).unwrap();
        let program = context.request_program("phong").unwrap();
        let texture = (context.texture_black.clone(), None);
        let phase = gfx_phase::Phase::new_cached(
           "Main",
            scene::tech::Technique::new(program, texture)
        );
        let scene = load::scene("data/vika", &mut context).unwrap();
        (phase, scene)
    };
    let mut camera = scene.cameras[0].clone();
    camera.projection.aspect = w as f32 / h as f32;

    println!("Rendering...");
    'main: loop {
        use gfx_scene::AbstractScene;
        // quit when Esc is pressed.
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) => break 'main,
                glutin::Event::Closed => break 'main,
                _ => {},
            }
        }

        scene.world.update();

        let clear_data = gfx::ClearData {
            color: [0.2, 0.3, 0.4, 1.0],
            depth: 1.0,
            stencil: 0,
        };
        renderer.reset();
        renderer.clear(clear_data, gfx::COLOR | gfx::DEPTH, &frame);
        scene.draw(&mut phase, &camera, &frame, &mut renderer).unwrap();

        device.submit(renderer.as_buffer());
        window.swap_buffers();
        device.after_frame();
    }
    println!("Done.");
}
