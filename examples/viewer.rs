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
    move_speed: f32,
    zoom_speed: f32,
    rotate_base: Option<(MousePos, claymore_scene::Transform<f32>)>,
    move_base: Option<(MousePos, cgmath::Vector3<f32>)>,
    last_pos: MousePos,
    space: claymore_scene::Transform<f32>,
}

impl Control {
    pub fn new(rot_speed: f32, move_speed: f32, zoom_speed: f32,
               space: claymore_scene::Transform<f32>) -> Control {
        Control {
            rotate_speed: rot_speed,
            move_speed: move_speed,
            zoom_speed: zoom_speed,
            rotate_base: None,
            move_base: None,
            last_pos: (0, 0),
            space: space,
        }
    }

    pub fn rot_capture(&mut self, transform: &claymore_scene::Transform<f32>) {
        self.rotate_base = Some((self.last_pos, transform.clone()));
    }

    pub fn rot_release(&mut self) {
        self.rotate_base = None;
    }

    pub fn move_capture(&mut self, transform: &claymore_scene::Transform<f32>) {
        self.move_base = Some((self.last_pos, transform.disp));
    }

    pub fn move_release(&mut self) {
        self.move_base = None;
    }

    pub fn position(&mut self, coords: MousePos,
                    transform: &mut claymore_scene::Transform<f32>) {
        self.last_pos = coords;
        match self.rotate_base {
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
        match self.move_base {
            Some((base_pos, ref base_disp)) => {
                use cgmath::{Vector, Rotation};
                let local_vector = cgmath::vec3(
                    -(coords.0 - base_pos.0) as f32,
                    (coords.1 - base_pos.1) as f32,
                    0.0).mul_s(self.move_speed);
                let cam_vector = transform.rot.rotate_vector(&local_vector);
                transform.disp = base_disp.add_v(&cam_vector);
            },
            None => (),
        }
    }

    pub fn wheel(&mut self, shift: i32, transform: &mut claymore_scene::Transform<f32>) {
        use cgmath::{Vector, Transform};
        let vector = transform.transform_vector(&cgmath::vec3(0.0, 0.0, 1.0));
        transform.disp.add_self_v(&vector.mul_s(shift as f32 * -self.zoom_speed));
    }
}


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
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
        .build().unwrap();
    unsafe { window.make_current() };
    let (w, h) = window.get_inner_size().unwrap();
    let mut graphics = gfx_device_gl::create(|s| window.get_proc_address(s))
                                    .into_graphics();

    let mut debug_renderer = gfx_debug_draw::DebugRenderer::new(
        &mut graphics, [w, h], 64, None, None).ok().unwrap();

    let (mut scene, texture) = {
        let mut context = claymore_load::Context::new(&mut graphics.factory,
            env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string())
            ).unwrap();
        let mut scene = context.create_scene();
        for path in env::args().skip(1) {
            println!("Loading scene: {}", path);
            context.extend_scene(&mut scene, &path).unwrap();
        }
        (scene, (context.texture_black.clone(), None))
    };

    println!("Initializing the graphics...");
    let mut pipeline = gfx_pipeline::forward::Pipeline::<gfx_device_gl::Device>::new(
        &mut graphics.factory, texture).unwrap();
    pipeline.background = Some([0.2, 0.3, 0.4, 1.0]);
    let mut frame = gfx::Frame::new(w as u16, h as u16);

    let mut camera = scene.cameras[0].clone();
    camera.projection.aspect = w as f32 / h as f32;
    let mut control = {
        use claymore_scene::base::World;
        let target_node = scene.entities[0].node;
        Control::new(0.005, 0.01, 0.5, scene.world.get_transform(&target_node))
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
                    control.rot_capture(&scene.world.get_node(camera.node).local),
                glutin::Event::MouseInput(glutin::ElementState::Released, glutin::MouseButton::Left) =>
                    control.rot_release(),
                glutin::Event::MouseInput(glutin::ElementState::Pressed, glutin::MouseButton::Middle) =>
                    control.move_capture(&scene.world.get_node(camera.node).local),
                glutin::Event::MouseInput(glutin::ElementState::Released, glutin::MouseButton::Middle) =>
                    control.move_release(),
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
            let r = node.world.transform_as_point(&vec3(0.0, 0.0, 0.0)).into_fixed();
            let x = node.world.transform_as_point(&vec3(len, 0.0, 0.0)).into_fixed();
            let y = node.world.transform_as_point(&vec3(0.0, len, 0.0)).into_fixed();
            let z = node.world.transform_as_point(&vec3(0.0, 0.0, len)).into_fixed();
            debug_renderer.draw_line(r, x, [1.0, 0.0, 0.0, 0.5]);
            debug_renderer.draw_line(r, y, [0.0, 1.0, 0.0, 0.5]);
            debug_renderer.draw_line(r, z, [0.0, 0.0, 1.0, 0.5]);
        }

        let buf = pipeline.render(&scene, &camera, &frame).unwrap();
        graphics.device.submit(buf);

        // this causes an ICE: https://github.com/rust-lang/rust/issues/24152
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
