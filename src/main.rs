extern crate env_logger;
extern crate glutin;
extern crate gfx;
extern crate gfx_window_glutin;
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
    let mut canvas = gfx_window_glutin::init(window).into_canvas();

    println!("Loading the game...");
    let mut app = game::App::new(&mut canvas.factory);

    println!("Rendering...");
    'main: loop {
        // quit when Esc is pressed.
        for event in canvas.output.window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) => break 'main,
                glutin::Event::Closed => break 'main,
                _ => {},
            }
        }

        app.render(&mut canvas.renderer, &canvas.output);

        canvas.present();
    }
    println!("Done.");
}
