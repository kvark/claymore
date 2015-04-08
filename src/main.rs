extern crate env_logger;
extern crate glutin;
extern crate gfx;
extern crate gfx_device_gl;
extern crate claymore_game as game;

pub fn main() {
    use gfx::traits::*;

    env_logger::init().unwrap();
    println!("Initializing the window...");

    let window = glutin::WindowBuilder::new()
        .with_title("Claymore".to_string())
        .with_vsync()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
        .build().unwrap();
    unsafe { window.make_current() };
    let (mut device, mut factory) = gfx_device_gl::create(|s| window.get_proc_address(s));
    let (w, h) = window.get_inner_size().unwrap();

    println!("Loading the game...");
    let mut app: game::App<gfx_device_gl::Device> =
        game::App::new(&mut factory, w as u16, h as u16);

    println!("Rendering...");
    'main: loop {
        // quit when Esc is pressed.
        for event in window.poll_events() {
            match event {
                glutin::Event::Resized(w, h) => app.set_size(w as u16, h as u16),
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) => break 'main,
                glutin::Event::Closed => break 'main,
                _ => {},
            }
        }

        let buf = app.render().unwrap();

        device.submit(buf);
        window.swap_buffers();
        device.after_frame();
        factory.cleanup();
    }
    println!("Done.");
}
