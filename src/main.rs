#![crate_name = "claymore"]
#![crate_type = "bin"]

extern crate blade;
extern crate gfx;
extern crate glfw;

use gfx::{Device, DeviceHelper};
use glfw::Context;

fn main() {
    println!("Initializing the window...");
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
        .ok().expect("Failed to initialize glfs-rs");

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
    glfw.window_hint(glfw::WindowHint::OpenglForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::OpenglProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw
        .create_window(640, 480, "Cube example", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.make_current();
    glfw.set_error_callback(glfw::FAIL_ON_ERRORS);
    window.set_key_polling(true);

    let (w, h) = window.get_framebuffer_size();
    let frame = gfx::Frame::new(w as u16, h as u16);

    let mut device = gfx::GlDevice::new(|s| window.get_proc_address(s));
    let mut renderer = device.create_renderer();

    println!("Loading the test scene...");
    let scene = blade::scene::load_json("data/test", &mut device).unwrap();

    println!("Rendering...");
    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) =>
                window.set_should_close(true),
                _ => {},
            }
        }

        let clear_data = gfx::ClearData {
            color: [0.2, 0.3, 0.4, 1.0],
            depth: 1.0,
            stencil: 0,
        };
        renderer.clear(clear_data, gfx::COLOR | gfx::DEPTH, &frame);
        scene.draw(&mut renderer);

        device.submit(renderer.as_buffer());
        renderer.reset();
        window.swap_buffers();
    }
    println!("Done.");
}
