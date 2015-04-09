extern crate env_logger;
extern crate cgmath;
extern crate glutin;
extern crate gfx;
extern crate gfx_pipeline;
extern crate gfx_device_gl;
extern crate gfx_debug_draw;
extern crate claymore_load;
extern crate claymore_scene; //temp

type MousePos = (i32, i32);

struct Control {
    rotate_speed: f32,
    zoom_speed: f32,
    base: Option<(MousePos, claymore_scene::Transform<f32>)>,
    last_pos: MousePos,
    space: claymore_scene::Transform<f32>,
}

impl Control {
    pub fn new(rot_speed: f32, zoom_speed: f32,
               space: claymore_scene::Transform<f32>) -> Control {
        Control {
            rotate_speed: rot_speed,
            zoom_speed: zoom_speed,
            base: None,
            last_pos: (0, 0),
            space: space,
        }
    }

    pub fn capture(&mut self, transform: &claymore_scene::Transform<f32>) {
        self.base = Some((self.last_pos, transform.clone()));
    }

    pub fn release(&mut self) {
        self.base = None;
    }

    pub fn position(&mut self, coords: MousePos,
                    transform: &mut claymore_scene::Transform<f32>) {
        self.last_pos = coords;
        match self.base {
            Some((ref base_pos, ref base_transform)) => {
                use cgmath::Transform;
                // p' = Mp * Tc^ * (Tr * Rz * Tr^) * p
                // Tx = (Tr * Rz^ * Tr^) * Tc
                let path = (coords.0 - base_pos.0) as f32 * -self.rotate_speed;
                let rotation = cgmath::Decomposed {
                    scale: 1.0,
                    rot: cgmath::Rotation3::from_axis_angle(
                        &cgmath::vec3(0.0, 0.0, 1.0), cgmath::rad(path)),
                    disp: cgmath::zero(),
                };
                let space_inv = self.space.invert().unwrap();
                let relative = self.space.concat(&rotation.concat(&space_inv));
                *transform = relative.concat(base_transform);
            },
            None => (),
        }
    }

    pub fn wheel(&mut self, shift: i32, transform: &mut claymore_scene::Transform<f32>) {
        use cgmath::{Vector, Transform};
        if self.base.is_some() {
            return
        }
        let vector = transform.transform_vector(&cgmath::vec3(0.0, 0.0, 1.0));
        transform.disp.add_self_v(&vector.mul_s(shift as f32 * -self.zoom_speed));
    }
}


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
    let mut graphics = gfx_device_gl::create(|s| window.get_proc_address(s))
                                    .into_graphics();

    let mut debug_renderer = gfx_debug_draw::DebugRenderer::new(
        &mut graphics, [w, h], 64, None, None).ok().unwrap();

    println!("Loading scene: {}", path);
    let (mut scene, texture) = {
        let mut context = claymore_load::Context::new(&mut graphics.factory,
            env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string())
            ).unwrap();
        let scene = context.load_scene(&path).unwrap();
        (scene, (context.texture_black.clone(), None))
    };

    let mut pipeline = gfx_pipeline::forward::Pipeline::<gfx_device_gl::Device>::new(
        &mut graphics.factory, texture).unwrap();
    pipeline.background = Some([0.2, 0.3, 0.4, 1.0]);
    let mut frame = gfx::Frame::new(w as u16, h as u16);

    let mut camera = scene.cameras[0].clone();
    camera.projection.aspect = w as f32 / h as f32;
    let mut control = {
        use claymore_scene::base::World;
        let target_node = scene.entities[0].node;
        Control::new(0.005, 0.5, scene.world.get_transform(&target_node))
    };

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
                glutin::Event::MouseInput(glutin::ElementState::Pressed, glutin::MouseButton::Left) =>
                    control.capture(&scene.world.get_node(camera.node).local),
                glutin::Event::MouseInput(glutin::ElementState::Released, glutin::MouseButton::Left) =>
                    control.release(),
                glutin::Event::MouseMoved(coords) =>
                    control.position(coords, &mut scene.world.mut_node(camera.node).local),
                glutin::Event::MouseWheel(shift) =>
                    control.wheel(shift, &mut scene.world.mut_node(camera.node).local),
                _ => {},
            }
        }

        scene.world.update();
        let len = 1f32;

        for node in scene.world.iter_nodes() {
            let r = node.world.transform_vector(&vec3(0.0, 0.0, 0.0)).into_fixed();
            let x = node.world.transform_vector(&vec3(len, 0.0, 0.0)).into_fixed();
            let y = node.world.transform_vector(&vec3(0.0, len, 0.0)).into_fixed();
            let z = node.world.transform_vector(&vec3(0.0, 0.0, len)).into_fixed();
            debug_renderer.draw_line(r, x, [1.0, 0.0, 0.0, 1.0]);
            debug_renderer.draw_line(r, y, [0.0, 1.0, 0.0, 1.0]);
            debug_renderer.draw_line(r, z, [0.0, 0.0, 1.0, 1.0]);
        }

        let buf = pipeline.render(&scene, &camera, &frame).unwrap();
        graphics.device.submit(buf);

        //debug_renderer.render(&mut graphics, &frame,
        //    camera.get_view_projection(&scene.world));
        if true {
            use cgmath::FixedArray;
            use claymore_scene::base::World;
            let cam_inv = scene.world.get_transform(&camera.node).invert().unwrap();
            let proj_mx = camera.projection.to_matrix4().mul_m(&cam_inv.to_matrix4()).into_fixed();
            debug_renderer.render(&mut graphics, &frame, proj_mx);
        }

        graphics.end_frame();
        window.swap_buffers();
        graphics.device.after_frame();
    }
    println!("Done.");
}
