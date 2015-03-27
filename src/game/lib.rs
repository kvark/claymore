extern crate gfx;
extern crate gfx_phase;
extern crate gfx_scene;
extern crate claymore_scene as scene;
extern crate claymore_load as load;

pub struct App<D: gfx::Device> {
    renderer: gfx::Renderer<D::Resources, D::CommandBuffer>,
    frame: gfx::Frame<D::Resources>,
    scene: scene::Scene<D::Resources, load::Scalar>,
    phase: scene::tech::Phase<D::Resources>,
}

impl<
    R: gfx::Resources,
    C: gfx::CommandBuffer<R>,
    D: gfx::Device<Resources = R, CommandBuffer = C> + gfx::Factory<R>
> App<D> {
    pub fn new(device: &mut D, width: u16, height: u16) -> App<D>
    {
        use gfx::traits::*;
        let renderer = device.create_renderer();
        let mut context = load::Context::new(device).unwrap();
        let program = context.request_program("phong").unwrap();
        let texture = (context.texture_black.clone(), None);
        let phase = gfx_phase::Phase::new_cached(
           "Main",
            scene::tech::Technique::new(program, texture)
        );
        let mut scene = load::scene("data/vika", &mut context).unwrap();
        scene.cameras[0].projection.aspect = width as f32 / height as f32;
        // done
        App {
            renderer: renderer,
            frame: gfx::Frame::new(width, height),
            scene: scene,
            phase: phase,
        }
    }

    pub fn render(&mut self) -> Result<gfx::SubmitInfo<D>, gfx_scene::Error> {
        use gfx_scene::AbstractScene;
        self.scene.world.update();

        let clear_data = gfx::ClearData {
            color: [0.2, 0.3, 0.4, 1.0],
            depth: 1.0,
            stencil: 0,
        };
        self.renderer.reset();
        self.renderer.clear(clear_data, gfx::COLOR | gfx::DEPTH, &self.frame);
        let camera = self.scene.cameras[0].clone();

        match self.scene.draw(&mut self.phase, &camera, &self.frame, &mut self.renderer) {
            Ok(_) => Ok(self.renderer.as_buffer()),
            Err(e) => Err(e),
        }
    }
}
