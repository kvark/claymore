use std::marker::PhantomData;
use cgmath;
use gfx;
use hex2d;
use scene;

#[shader_param]
struct Param<R: gfx::Resources> {
    mvp: [[f32; 4]; 4],
    _dummy: PhantomData<R>,
}

#[vertex_format]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}

static VERTEX_SRC: &'static [u8] = b"
    #version 120

    uniform mat4 mvp;
    attribute vec2 position;

    void main() {
        gl_Position = mvp * vec4(position, 0.0, 1.0);
        gl_PointSize = 2.0;
    }
";

static FRAGMENT_SRC: &'static [u8] = b"
    #version 120

    void main() {
        gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    }
";


pub type Coordinate = hex2d::Coordinate<i8>;

pub struct Grid<R: gfx::Resources> {
    pub node: scene::NodeId<f32>,
    spacing: hex2d::Spacing,
    batch: gfx::batch::OwnedBatch<Param<R>>,
}

impl<R: gfx::Resources> Grid<R> {
    pub fn new<F: gfx::Factory<R>>(factory: &mut F,
               node: scene::NodeId<f32>, size: f32) -> Grid<R> {
        use gfx::traits::FactoryExt;
        let spacing = hex2d::Spacing::FlatTop(size);
        let mut vertices = Vec::new();
        for i in -10 .. 10 {
            for j in -10 .. 10 {
                let (x, y) = hex2d::Coordinate::new(i, j).to_pixel_float(spacing);
                vertices.push(Vertex {
                    position: [x, y],
                });
            }
        }
        let mesh = factory.create_mesh(&vertices);
        let program = factory.link_program(VERTEX_SRC, FRAGMENT_SRC).unwrap();
        let mut batch = gfx::batch::OwnedBatch::new(mesh, program, Param {
            mvp: [[0.0; 4]; 4],
            _dummy: PhantomData,
        }).unwrap();
        batch.state.depth(gfx::state::Comparison::LessEqual, false);
        batch.slice.prim_type = gfx::PrimitiveType::Point;
        Grid {
            node: node,
            spacing: spacing,
            batch: batch,
        }
    }

    pub fn get_center(&self, coord: Coordinate) -> cgmath::Point3<f32> {
        let (x, y) = coord.to_pixel_float(self.spacing);
        cgmath::Point3::new(x, y, 0.0)
    }

    pub fn get_cell(&self, position: cgmath::Point3<f32>) -> Coordinate {
        hex2d::Coordinate::new(0, 0)    //TODO
    }

    pub fn draw<C: gfx::CommandBuffer<R>, O: gfx::Output<R>>(&self,
                renderer: &mut gfx::Renderer<R, C>, output: &O) {
        renderer.draw(&self.batch, output).unwrap();
    }
}
