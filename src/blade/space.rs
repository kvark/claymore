use cgmath::{BaseNum, Transform, Transform3};
use Id;

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
    bones: Vec<Bone<T>>,
}

pub struct World<S, T> {
    nodes: Vec<Node<T>>,
    skeletons: Vec<Skeleton<T>>,
}

impl<S: BaseNum, T: Transform3<S> + Clone> World<S, T> {
    pub fn new() -> World<S, T> {
        World {
            nodes: Vec::new(),
            skeletons: Vec::new(),
        }
    }

    pub fn find_node(&self, name: &str) -> Option<Id<Node<T>>> {
        self.nodes.iter().position(|n| n.name == name)
                         .map(|i| Id(i))
    }

    pub fn add_node(&mut self, name: String, parent: Parent<T>, local: T)
            -> Id<Node<T>> {
        //TODO: check that parent is valid
        let nid = Id(self.nodes.len());
        self.nodes.push(Node {
            name: name,
            parent: parent,
            local: local,
            world: Transform::identity(),
        });
        nid
    }

    pub fn update(&mut self) {
        for i in 0.. self.nodes.len() {
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
        }
    }
}
