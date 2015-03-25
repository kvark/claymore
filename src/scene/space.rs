use std::marker::PhantomData;
use cgmath::{BaseFloat, Transform, Transform3};
use id::{Array, Id};
use gfx_scene;

#[derive(Copy)]
pub enum Parent<T> {
    None,
    Domestic(Id<Node<T>>),
    Foreign(Id<Skeleton<T>>, Id<Bone<T>>),
}

pub struct Node<T> {
    pub name : String,
    parent: Parent<T>,
    pub local: T,
    pub world: T,
}

pub struct Bone<T> {
    pub name : String,
    parent: Option<Id<Bone<T>>>,
    pub local: T,
    pub world: T,
    bind_pose: T,
    bind_pose_root_inverse: T,
}

pub struct Skeleton<T> {
    pub name: String,
    node: Id<Node<T>>,
    bones: Array<Bone<T>>,
}

pub struct World<S, T> {
    nodes: Array<Node<T>>,
    skeletons: Array<Skeleton<T>>,
    phantom: PhantomData<S>,
}

impl<S: BaseFloat, T: Transform3<S> + Clone> World<S, T> {
    pub fn new() -> World<S, T> {
        World {
            nodes: Array::new(),
            skeletons: Array::new(),
            phantom: PhantomData,
        }
    }

    pub fn get_node(&self, id: Id<Node<T>>) -> &Node<T> {
        self.nodes.get(id)
    }

    pub fn mut_node(&mut self, id: Id<Node<T>>) -> &mut Node<T> {
        self.nodes.get_mut(id)
    }

    pub fn find_node(&self, name: &str) -> Option<Id<Node<T>>> {
        self.nodes.find_id(|n| n.name == name)
    }

    pub fn add_node(&mut self, name: String, parent: Parent<T>, local: T)
                    -> Id<Node<T>> {
        //TODO: check that parent is valid
        self.nodes.add(Node {
            name: name,
            parent: parent,
            local: local,
            world: Transform::identity(),
        })
    }

    pub fn update(&mut self) {
        //TODO: need direct access to Array
        /*for i in 0.. self.nodes.len() {
            let (left, right) = self.nodes.split_at_mut(i);
            let n = &mut right[0];
            n.world = match n.parent {
                Parent::None => n.local.clone(),
                Parent::Domestic(Id(pid)) => {
                    assert!(pid < i);
                    left[pid].world.concat(&n.local)
                },
                Parent::Foreign(Id(sid), Id(bid)) => {
                    self.skeletons[sid].bones[bid].world.concat(&n.local)
                },
            };
        }

        //TODO: refactor to avoid a possible lag, caused by bone parenting

        for s in self.skeletons.iter_mut() {
            let Id(nid) = s.node;
            let world = &self.nodes[nid].world;
            for i in 0.. s.bones.len() {
                let (left, right) = s.bones.split_at_mut(i);
                let b = &mut right[0];
                let base = match b.parent {
                    Some(Id(bid)) => {
                        assert!(bid < i);
                        &left[bid].world
                    },
                    None => world
                };
                b.world = base.concat(&b.local);
            }
        }*/
    }
}

impl<S: BaseFloat + 'static, T: Transform3<S> + Clone> gfx_scene::World for World<S, T> {
    type Scalar = S;
    type Transform = T;
    type NodePtr = Id<Node<T>>;
    type SkeletonPtr = Id<Skeleton<T>>;

    fn get_transform(&self, id: &Id<Node<T>>) -> T {
        self.get_node(*id).world.clone()
    }
}
