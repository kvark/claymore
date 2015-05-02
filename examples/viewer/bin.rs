extern crate env_logger;
extern crate cgmath;
extern crate glutin;
extern crate gfx;
extern crate gfx_pipeline;
extern crate gfx_window_glutin;
extern crate gfx_debug_draw;
extern crate claymore_load;
extern crate claymore_scene;

mod control;

fn main() {
    use std::env;
    use cgmath::{vec3, FixedArray, Matrix, ToMatrix4, Transform};
    use gfx::traits::*;
    use gfx_pipeline::Pipeline;

    env_logger::init().unwrap();
    println!("Creating the window...");

    let window = glutin::WindowBuilder::new()
        .with_title("Scene viewer".to_string())
        .with_vsync()
        .with_gl(glutin::GL_CORE)
        .build().unwrap();
    let mut canvas = gfx_window_glutin::init(window).into_canvas();

    let mut debug_renderer = gfx_debug_draw::DebugRenderer::from_canvas(
        &mut canvas, 64, None, None).ok().unwrap();

    let mut scene = claymore_load::create_scene();
    {
        let mut context = claymore_load::Context::new(&mut canvas.factory,
            env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string()));
        context.alpha_test = Some(20);
        context.forgive = true;
        for path in env::args().skip(1) {
            println!("Loading scene: {}", path);
            context.extend_scene(&mut scene, &path).unwrap();
        }
    }

    println!("Initializing the graphics...");
    let mut pipeline = gfx_pipeline::forward::Pipeline::new(&mut canvas.factory)
                                                       .unwrap();
    pipeline.background = Some([0.2, 0.3, 0.4, 1.0]);

    let mut camera = match scene.cameras.first() {
        Some(cam) => cam.clone(),
        None => {
            println!("No cameras found in any of the scenes. Usage:");
            println!("viewer <path_to_scene1> <path_to_scene2> ...");
            return;
        }
    };
    let mut control = {
        use claymore_scene::base::World;
        let target_node = scene.entities[0].node;
        control::Control::new(0.005, 0.01, 0.5,
            scene.world.get_transform(&target_node))
    };

    println!("Rendering...");
    'main: loop {
        for event in canvas.output.window.poll_events() {
            use glutin::{Event, ElementState, MouseButton, VirtualKeyCode};
            match event {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) =>
                    break 'main,
                Event::Closed => break 'main,
                Event::MouseInput(ElementState::Pressed, MouseButton::Left) =>
                    control.rot_capture(&scene.world.get_node(camera.node).local),
                Event::MouseInput(ElementState::Released, MouseButton::Left) =>
                    control.rot_release(),
                Event::MouseInput(ElementState::Pressed, MouseButton::Middle) =>
                    control.move_capture(&scene.world.get_node(camera.node).local),
                Event::MouseInput(ElementState::Released, MouseButton::Middle) =>
                    control.move_release(),
                Event::MouseMoved(coords) =>
                    control.position(coords, &mut scene.world.mut_node(camera.node).local),
                Event::MouseWheel(shift) =>
                    control.wheel(shift, &mut scene.world.mut_node(camera.node).local),
                _ => {},
            }
        }

        scene.world.update();
        let len = 0.1f32;

        for node in scene.world.iter_nodes() {
            let r = node.world.transform_as_point(&vec3(0.0, 0.0, 0.0)).into_fixed();
            let x = node.world.transform_as_point(&vec3(len, 0.0, 0.0)).into_fixed();
            let y = node.world.transform_as_point(&vec3(0.0, len, 0.0)).into_fixed();
            let z = node.world.transform_as_point(&vec3(0.0, 0.0, len)).into_fixed();
            debug_renderer.draw_line(r, x, [1.0, 0.0, 0.0, 0.5]);
            debug_renderer.draw_line(r, y, [0.0, 1.0, 0.0, 0.5]);
            debug_renderer.draw_line(r, z, [0.0, 0.0, 1.0, 0.5]);
        }

        camera.projection.aspect = canvas.get_aspect_ratio();
        pipeline.render(&scene, &camera, &mut canvas).unwrap();

        // this causes an ICE: https://github.com/rust-lang/rust/issues/24152
        //debug_renderer.render_canvas(&mut canvas, camera.get_view_projection(&scene.world));
        if true {
            use cgmath::FixedArray;
            use claymore_scene::base::World;
            let cam_inv = scene.world.get_transform(&camera.node).invert().unwrap();
            let proj_mx = camera.projection.to_matrix4().mul_m(&cam_inv.to_matrix4()).into_fixed();
            debug_renderer.render_canvas(&mut canvas, proj_mx);
        }

        canvas.present();
    }
    println!("Done.");
}
