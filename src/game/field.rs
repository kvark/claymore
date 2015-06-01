use std::marker::PhantomData;
use cgmath;
use cgmath::FixedArray;
use gfx;
use grid;
use grid::Grid;
use scene;

gfx_parameters!( Param {
    u_Transform@ mvp: [[f32; 4]; 4],
    u_Color@ color: [f32; 4],
});

gfx_vertex!( Vertex {
    a_Position@ position: [f32; 2],
});

impl Vertex {
    pub fn new(x: f32, y: f32) -> Vertex {
        Vertex {
            position: [x, y],
        }
    }
}

static VERTEX_SRC: &'static [u8] = b"
    #version 150 core

    uniform mat4 u_Transform;
    in vec2 a_Position;

    void main() {
        gl_Position = u_Transform * vec4(a_Position, 0.0, 1.0);
    }
";

static FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    uniform vec4 u_Color;
    out vec4 o_Color;

    void main() {
        o_Color = u_Color;
    }
";

pub use grid::quad::{Coordinate, Direction};

pub struct Field<R: gfx::Resources> {
    pub node: scene::NodeId<f32>,
    pub grid: grid::quad::Grid,
    batch: gfx::batch::Full<Param<R>>,
}

impl<R: gfx::Resources> Field<R> {
    pub fn new<F: gfx::Factory<R>>(factory: &mut F,
               node: scene::NodeId<f32>, size: f32, area: f32,
               color: (f32, f32, f32, f32))
               -> Field<R> {
        use gfx::traits::FactoryExt;
        let grid = grid::quad::Grid::new(size);
        let area = [[-area, -area, 0.0], [area, area, 0.0]];
        let vertices = grid.fold_edges_in_area(&area, Vec::new(), |mut u, a, b, _, _| {
            u.push(Vertex::new(a[0], a[1]));
            u.push(Vertex::new(b[0], b[1]));
            u
        });
        let mesh = factory.create_mesh(&vertices);
        let program = factory.link_program(VERTEX_SRC, FRAGMENT_SRC).unwrap();
        let mut batch = gfx::batch::Full::new(mesh, program, Param {
            mvp: [[0.0; 4]; 4],
            color: [color.0, color.1, color.2, color.3],
            _r: PhantomData,
        }).unwrap();
        batch.state = batch.state.depth(gfx::state::Comparison::LessEqual, false);
        batch.slice.prim_type = gfx::PrimitiveType::Line;
        Field {
            node: node,
            grid: grid,
            batch: batch,
        }
    }

    pub fn get_center(&self, coord: Coordinate) -> cgmath::Point3<f32> {
        let fixed = self.grid.get_cell_center(coord);
        cgmath::Point3::new(fixed[0], fixed[1], fixed[2])
    }

    pub fn get_cell(&self, position: &cgmath::Point3<f32>) -> Coordinate {
        self.grid.get_coordinate(position.as_fixed())
    }

    pub fn cast_ray(&self, ray: &cgmath::Ray3<f32>) -> Coordinate {
        use cgmath::{Point, Vector};
        let t = -ray.origin.z / ray.direction.z;
        let p = ray.origin.add_v(&ray.direction.mul_s(t));
        self.get_cell(&p)
    }

    pub fn update_params(&mut self, camera: &scene::Camera<f32>, world: &scene::World<f32>) {
        use cgmath::{Matrix, Transform};
        let mx_proj: cgmath::Matrix4<f32> = camera.projection.clone().into();
        let model_view = world.get_node(camera.node).world.invert().unwrap()
                              .concat(&world.get_node(self.node).world);
        self.batch.params.mvp = mx_proj.mul_m(&model_view.into()).into_fixed();
    }

    pub fn draw<S: gfx::Stream<R>>(&self, stream: &mut S) {
        stream.draw(&self.batch).unwrap();
    }
}
