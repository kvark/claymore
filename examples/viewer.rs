extern crate env_logger;
extern crate cgmath;
extern crate glutin;
extern crate gfx;
extern crate gfx_pipeline;
extern crate gfx_device_gl;
extern crate gfx_debug_draw;
extern crate claymore_load;

fn main() {
    use std::env;
    use cgmath::{vec3, FixedArray, Matrix, ToMatrix4, Transform};
    use gfx::traits::*;
    use gfx_pipeline::Pipeline;

    let path = match env::args().nth(1) {
        Some(p) => p,
        None => {
            println!("Call as 'viewer <path_to_scene>`");
            return
        }
    };

    env_logger::init().unwrap();
    println!("Initializing the window...");

    let window = glutin::WindowBuilder::new()
        .with_title("Scene viewer".to_string())
        .with_vsync()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
        .build().unwrap();
    unsafe { window.make_current() };
    let (w, h) = window.get_inner_size().unwrap();
    let mut graphics = gfx_device_gl::GlDevice::new(|s| window.get_proc_address(s))
                                    .into_graphics();

    let mut debug_renderer = gfx_debug_draw::DebugRenderer::new(
        *graphics.device.get_capabilities(),
        &mut graphics.device,
        [w, h], 64, None, None
        ).ok().unwrap();

    println!("Loading scene: {}", path);
    let (mut scene, texture) = {
        let mut context = claymore_load::Context::new(&mut graphics.device,
            env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string())
            ).unwrap();
        let scene = context.load_scene(&path).unwrap();
        (scene, (context.texture_black.clone(), None))
    };

    let mut pipeline = gfx_pipeline::forward::Pipeline::new(
        &mut graphics.device, texture
        ).unwrap();
    pipeline.background = Some([0.2, 0.3, 0.4, 1.0]);
    let mut frame = gfx::Frame::new(w as u16, h as u16);

    println!("Rendering...");
    'main: loop {
        for event in window.poll_events() {
            match event {
                glutin::Event::Resized(w, h) => {
                    frame.width = w as u16;
                    frame.height = h as u16;
                    debug_renderer.resize(w, h);
                },
                glutin::Event::KeyboardInput(_, _,
                    Some(glutin::VirtualKeyCode::Escape)) =>
                    break 'main,
                glutin::Event::Closed => break 'main,
                _ => {},
            }
        }

        scene.world.update();
        let mut camera = scene.cameras[0].clone();
        camera.projection.aspect = w as f32 / h as f32;

        for node in scene.world.iter_nodes() {
            let r = node.world.transform_vector(&vec3(0.0, 0.0, 0.0)).into_fixed();
            let x = node.world.transform_vector(&vec3(1.0, 0.0, 0.0)).into_fixed();
            let y = node.world.transform_vector(&vec3(0.0, 1.0, 0.0)).into_fixed();
            let z = node.world.transform_vector(&vec3(0.0, 0.0, 1.0)).into_fixed();
            debug_renderer.draw_line(r, x, [1.0, 0.0, 0.0, 1.0]);
            debug_renderer.draw_line(r, y, [0.0, 1.0, 0.0, 1.0]);
            debug_renderer.draw_line(r, z, [0.0, 0.0, 1.0, 1.0]);
        }

        let buf = pipeline.render(&scene, &camera, &frame).unwrap();
        graphics.device.submit(buf);

        debug_renderer.update(&mut graphics.device);
        let camatrix = camera.projection.to_matrix4().mul_m(
            &scene.world.get_node(camera.node).world.to_matrix4()
            ).into_fixed();
        debug_renderer.render(&mut graphics, &frame, camatrix);

        graphics.end_frame();
        window.swap_buffers();
        graphics.device.after_frame();
    }
    println!("Done.");
}
