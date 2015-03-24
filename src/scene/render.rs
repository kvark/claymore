use std::{num, slice};
use std::cmp::Ordering;
use cgmath;
use gfx;
use gfx::shade::ShaderParam;

pub type Depth = u32;

pub struct Object<P: ShaderParam> {
    batch: gfx::batch::RefBatch<P>,
    parameters: P,
    depth: Depth,
}

pub fn order_opaque<P: ShaderParam>(a: &Object<P>, b: &Object<P>) -> Ordering {
    (&a.batch, a.depth).cmp(&(&b.batch, b.depth))
}

type Index = usize; //TODO: u32

pub struct ObjectIter<'a, P: 'a + ShaderParam> {
    objects: &'a [Object<P>],
    id_iter: slice::Iter<'a, Index>,
}

impl<'a, P: ShaderParam> Iterator for ObjectIter<'a, P> {
    type Item = &'a Object<P>;

    fn next(&mut self) -> Option<&'a Object<P>> {
        self.id_iter.next().map(|&i| &self.objects[i as usize])
    }
}

pub struct Queue<P: ShaderParam> {
    pub objects: Vec<Object<P>>,
    indices: Vec<Index>,
}

impl<P: ShaderParam> Queue<P> {
    fn is_updated(&self) -> bool {
        self.objects.len() == self.indices.len()
    }

    /// Synchronize indices to have the same length as objects
    pub fn update(&mut self) {
        let ni = self.indices.len();
        if self.objects.len() > ni {
            for i in (ni .. self.objects.len()) {
                self.indices.push(i as Index);
            }
        }else
        if self.objects.len() < ni {
            self.indices.retain(|&i| (i as usize) < ni);
        }
        debug_assert!(self.is_updated());
    }

    /// Sort objects with the given order
    pub fn sort<F: Fn(&Object<P>, &Object<P>) -> Ordering>
                (&mut self, order: F) {
        self.update();
        let objects = self.objects.as_slice();
        self.indices.sort_by(|&ia, &ib|
            (order)(&objects[ia], &objects[ib])
        );
    }

    /// Iterate objects in the sorted order
    pub fn iter<'a>(&'a self) -> ObjectIter<'a, P>  {
        debug_assert!(self.is_updated());
        ObjectIter {
            objects: self.objects.as_slice(),
            id_iter: self.indices.iter(),
        }
    }
}

pub type Bound = ();    //FIXME

pub struct View<S, P: ShaderParam, T> {
    cam_inverse: T,
    frustum: cgmath::Frustum<S>,
    queue: Queue<P>,
}

impl<
    S: cgmath::BaseFloat + num::FromPrimitive,
    P: ShaderParam,
    T: cgmath::Transform3<S>
> View<S, P, T> {
    pub fn clear(&mut self) {
        self.queue.objects.clear()
    }

    pub fn add(&mut self, batch: gfx::batch::RefBatch<P>, data: P,
               transform: &T, _bound: Bound) {
        let view = self.cam_inverse.concat(transform);
        let distance = view.transform_vector(&cgmath::zero());
        let depth_max: Depth = num::Int::max_value();   //FIXME
        let depth_max = num::FromPrimitive::from_u32(depth_max).unwrap();
        let depth = (depth_max * (distance.z - self.frustum.near.d) /
            (self.frustum.far.d - self.frustum.near.d))
            .to_u32().unwrap();
        //TODO: cull based on `bound`
        self.queue.objects.push(Object {
            batch: batch,
            parameters: data,
            depth: depth,
        });
    }

    pub fn render<'a, C: gfx::CommandBuffer>(&'a mut self,
                  renderer: &mut gfx::Renderer<C>, frame: &gfx::Frame,
                  context: &'a gfx::batch::Context) {
        self.queue.sort(order_opaque);
        for ob in self.queue.iter() {
            renderer.draw(&(&ob.batch, &ob.parameters, context), frame)
                    .unwrap();   //FIXME
        }
    }
}
