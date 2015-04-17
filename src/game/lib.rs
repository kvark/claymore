extern crate gfx;
extern crate gfx_scene;
extern crate gfx_pipeline;
extern crate claymore_scene as scene;
extern crate claymore_load as load;

use gfx_pipeline::forward::Pipeline;


pub struct App<D: gfx::Device> {
    scene: scene::Scene<D::Resources, load::Scalar>,
    pipeline: Pipeline<D>,
}

impl<D: gfx::Device> App<D> {
    pub fn new<F: gfx::Factory<D::Resources>>(device: &D, factory: &mut F) -> App<D> {
        use std::env;
        // load the scene
        let (scene, texture) = {
            let mut context = load::Context::new(factory,
                env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string())
                ).unwrap();
            let scene = context.load_scene("data/vika").unwrap();
            (scene, (context.texture_black.clone(), None))
        };
        // create the pipeline
        let mut pipeline = Pipeline::new(device, factory, texture).unwrap();
        pipeline.background = Some([0.2, 0.3, 0.4, 1.0]);
        // done
        App {
            scene: scene,
            pipeline: pipeline,
        }
    }

    pub fn render<O: gfx::Output<D::Resources>>(&mut self, output: &O)
        -> Result<gfx::SubmitInfo<D>, gfx_pipeline::Error> {
        use gfx_pipeline::Pipeline;
        self.scene.world.update();
        let mut camera = self.scene.cameras[0].clone();
        camera.projection.aspect = {
            let (w, h) = output.get_size();
            w as f32 / h as f32
        };
        self.pipeline.render(&self.scene, &camera, output)
    }
}
