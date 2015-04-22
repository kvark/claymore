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
    let (mut mouse_x, mut mouse_y) = (0, 0);
    'main: loop {
        // quit when Esc is pressed.
        for event in canvas.output.window.poll_events() {
            use glutin::{ElementState, Event, MouseButton, VirtualKeyCode};
            match event {
                Event::Closed => break 'main,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Escape)) => break 'main,
                Event::MouseMoved((x, y)) => { mouse_x = x; mouse_y = y; },
                Event::MouseInput(ElementState::Pressed, MouseButton::Left) => {
                    let (sx, sy) = canvas.output.get_size();
                    app.mouse_click(mouse_x as f32 / sx as f32, mouse_y as f32 / sy as f32);
                },
                _ => (),
            }
        }

        app.render(&mut canvas.renderer, &canvas.output);
        canvas.present();
    }
    println!("Done.");
}
