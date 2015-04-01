use std::marker::PhantomData;
use std::slice;
use cgmath::{BaseFloat, Transform, Transform3};
use id::{Array, Id, Storage};
use gfx_scene;

#[derive(Copy, Debug)]
pub enum Parent<T> {
    None,
    Domestic(Id<Node<T>>),
    Foreign(Id<Skeleton<T>>, Id<Bone<T>>),
}

#[derive(Debug)]
pub struct Node<T> {
    pub name : String,
    parent: Parent<T>,
    pub local: T,
    pub world: T,
}

#[derive(Debug)]
pub struct Bone<T> {
    pub name : String,
    parent: Option<Id<Bone<T>>>,
    pub local: T,
    pub world: T,
    bind_pose: T,
    bind_pose_root_inverse: T,
}

#[derive(Debug)]
pub struct Skeleton<T> {
    pub name: String,
    node: Id<Node<T>>,
    bones: Array<Bone<T>>,
}

#[derive(Debug)]
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

    pub fn iter_nodes<'a>(&'a self) -> slice::Iter<'a, Node<T>> {
        self.nodes.iter()
    }

    pub fn update(&mut self) {
        let skeletons = &mut self.skeletons;
        self.nodes.walk_looking_back(|left, n| {
            n.world = match n.parent {
                Parent::None => n.local.clone(),
                Parent::Domestic(pid) =>
                    left.get(pid)
                        .world.concat(&n.local),
                Parent::Foreign(sid, bid) =>
                    skeletons.get(sid)
                        .bones.get(bid)
                        .world.concat(&n.local),
            };
        });

        for s in skeletons.iter_mut() {
            let world = &self.nodes.get(s.node).world;
            s.bones.walk_looking_back(|left, b| {
                let base = match b.parent {
                    Some(bid) => &left.get(bid).world,
                    None => world,
                };
                b.world = base.concat(&b.local);
            })
        }
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
