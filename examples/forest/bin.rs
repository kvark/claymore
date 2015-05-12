extern crate clock_ticks;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rustc_serialize;
extern crate rand;
extern crate cgmath;
extern crate glutin;
extern crate gfx;
extern crate gfx_phase;
extern crate gfx_pipeline;
extern crate gfx_window_glutin;
extern crate gfx_debug_draw;
extern crate claymore_load;
extern crate claymore_scene;

mod generate;
mod reflect;


#[derive(Debug)]
enum Order {
    Default,
    FrontToBack,
    BackToFront,
    Material,
    Mesh,
    Unordered,
}

impl Order {
    pub fn next<R: gfx::Resources>(&mut self) -> Option<gfx_pipeline::forward::OrderFun<R>> {
        let (fun, order): (Option<gfx_pipeline::forward::OrderFun<R>>, Order) = match *self {
            Order::Default => (Some(gfx_phase::sort::front_to_back), Order::FrontToBack),
            Order::FrontToBack => (Some(gfx_phase::sort::back_to_front), Order::BackToFront),
            Order::BackToFront => (Some(gfx_phase::sort::program), Order::Material),
            Order::Material => (Some(gfx_phase::sort::mesh), Order::Mesh),
            Order::Mesh => (None, Order::Unordered),
            Order::Unordered => (Some(gfx_pipeline::forward::order), Order::Default),
        };
        *self = order;
        fun
    }
}

fn move_camera<S: cgmath::BaseFloat>(
    camera: &claymore_scene::Camera<S>,
    vec: &cgmath::Vector3<S>,
    world: &mut claymore_scene::World<S>
){
    use cgmath::{EuclideanVector, Transform, Vector};
    let node = world.mut_node(camera.node);
    let mut cam_offset = node.local.transform_vector(vec);
    let len = cam_offset.length();
    if vec.z != cgmath::zero() {
        cam_offset.x = cgmath::zero();
        cam_offset.y = cgmath::zero();
    }else {
        cam_offset.z = cgmath::zero();
    };
    let rescale = len / cam_offset.length();
    node.local.disp.add_self_v(&cam_offset.mul_s(rescale));
}

fn rotate_camera<S: cgmath::BaseFloat>(
    camera: &claymore_scene::Camera<S>,
    amount: S,
    world: &mut claymore_scene::World<S>
){
    use cgmath::{Transform, Vector};
    let node = world.mut_node(camera.node);
    let zvec = node.local.transform_vector(&cgmath::Vector3::unit_z());
    let t = -node.local.disp.z / zvec.z;
    let anchor = node.local.disp.add_v(&zvec.mul_s(t));
    let t_rotation = cgmath::Decomposed {
        scale: cgmath::one(),
        rot: cgmath::Rotation3::from_axis_angle(
            &cgmath::Vector3::unit_z(), cgmath::rad(amount)),
        disp: cgmath::zero(),
    };
    let t_offset_inv = cgmath::Decomposed {
        scale: cgmath::one(),
        rot: cgmath::Rotation::identity(),
        disp: -anchor,
    };
    let t_offset = cgmath::Decomposed {
        scale: cgmath::one(),
        rot: cgmath::Rotation::identity(),
        disp: anchor,
    };
    let relative = t_offset.concat(&t_rotation.concat(&t_offset_inv));
    node.local = relative.concat(&node.local);
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

    let mut last_moment = clock_ticks::precise_time_ns();
    let mut avg_time = 0;
    let mut order = Order::Default;

    println!("Rendering...");
    'main: loop {
        let delta = clock_ticks::precise_time_ns() - last_moment;
        avg_time = (avg_time * config.debug.time_factor + delta) /
            (config.debug.time_factor + 1);
        last_moment += delta;

        let seconds = (delta/1000000) as f32 / 1000.0;
        let move_delta = config.control.move_speed * seconds;
        let rotate_delta = config.control.rotate_speed * seconds;

        for event in canvas.output.window.poll_events() {
            // TODO: use the scroll
            use glutin::{Event, VirtualKeyCode};
            use glutin::ElementState::Pressed;
            match event {
                Event::Closed => break 'main,
                Event::KeyboardInput(Pressed, _, Some(VirtualKeyCode::Escape)) =>
                    break 'main,
                Event::KeyboardInput(Pressed, _, Some(VirtualKeyCode::A)) =>
                    move_camera(&camera, &cgmath::vec3(-move_delta, 0.0, 0.0), &mut scene.world),
                Event::KeyboardInput(Pressed, _, Some(VirtualKeyCode::D)) =>
                    move_camera(&camera, &cgmath::vec3(move_delta, 0.0, 0.0),  &mut scene.world),
                Event::KeyboardInput(Pressed, _, Some(VirtualKeyCode::S)) =>
                    move_camera(&camera, &cgmath::vec3(0.0, -move_delta, 0.0), &mut scene.world),
                Event::KeyboardInput(Pressed, _, Some(VirtualKeyCode::W)) =>
                    move_camera(&camera, &cgmath::vec3(0.0, move_delta, 0.0),  &mut scene.world),
                Event::KeyboardInput(Pressed, _, Some(VirtualKeyCode::X)) =>
                    move_camera(&camera, &cgmath::vec3(0.0, 0.0, -move_delta), &mut scene.world),
                Event::KeyboardInput(Pressed, _, Some(VirtualKeyCode::Z)) =>
                    move_camera(&camera, &cgmath::vec3(0.0, 0.0, move_delta),  &mut scene.world),
                Event::KeyboardInput(Pressed, _, Some(VirtualKeyCode::Q)) =>
                    rotate_camera(&camera, -rotate_delta, &mut scene.world),
                Event::KeyboardInput(Pressed, _, Some(VirtualKeyCode::E)) =>
                    rotate_camera(&camera, rotate_delta, &mut scene.world),
                Event::KeyboardInput(Pressed, _, Some(VirtualKeyCode::Tab)) =>
                    pipeline.phase.sort = order.next(),
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
                format!("frame time = {} ms", avg_time / 1000000),
                format!("primitives = {}", report.primitives_rendered),
                format!("order = {:?}", order),
                //format!("ratio = {}",     report.get_ratio()),
                //format!("invisible = {}", report.calls_invisible),
                format!("calls culled = {}", report.calls_culled),
                //format!("rejected = {}",  report.calls_rejected),
                //format!("failed = {}",    report.calls_failed),
                format!("calls passed = {}", report.calls_passed),
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
