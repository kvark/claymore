use claymore_scene;
use reflect;


struct TileComponent<R: gfx::Resources> {
    mesh: gfx::Mesh<R>,
    slice: gfx::Slice<R>,
    material: gfx_pipeline::Material<R>,
}

struct TipeProto<R: gfx::Resources> {
    node: claymore_scene::NodeId<f32>,
    components: Vec<TileComponent<R>>,
    river_mask: u8,
}

impl<R: gfx::Resources> TileInfo<R> {
    pub fn fit_orientation(&self, neighbors: u8, rivers: u8) -> Option<u8> {
        let mut mask = self.river_mask;
        for i in 0u8.. 4 {
            if (mask & neighbors) == rivers {
                return Some(i)
            }
            mask = ((mask << 1) & 0xF) | (mask >> 3);
        }
        None
    }
}

pub struct Gen<R: gfx::Resources> {
    proto_tiles: Vec<TileProto<R>>,
}

impl<R: gfx::Resources> Gen<R> {
    pub fn new(scene: &claymore_scene::Scene<R>) -> Gen<R> {
        let config: reflect::Demo = {
            use std::fs::File;
            use std::io::Read;
            use rustc_serialize::json;
            let mut file = File::open(&format!("{}/examples/forest/config.json", root))
                                .unwrap();
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            json::decode(&s).unwrap()
        };

        println!("Processing data...");
        let protos: Vec<_> = config.palette.tiles.iter().map(|t| {
            let mask = t.river.chars().fold(0, |m, c| match c {
                'n' => m | 1,
                'e' => m | 2,
                's' => m | 4,
                'w' => m | 8,
                _   => panic!("Unknown river direction: {}", c),
            });
            let node = scene.entities.iter()    //TODO
                                     .find(|ent| ent.name == t.name)
                                     .map(|ent| ent.node.clone())
                                     .unwrap();
            let components: Vec<_> = scene.entities.iter()
                                                   .filter(|ent| ent.name == t.name)
                                                   .map(|ent|
                TileComponent {
                    mesh: ent.mesh.clone(),
                    slice: ent.slice.clone(),
                    material: ent.material.clone(),
                }
            ).collect();
            info!("Found tile {} with {} components and river mask {}",
                t.name, components.len(), mask);
            TileProto {
                node: node,
                components: components,
                river_mask: mask,
            }
        }).collect();

        Gen {
            proto_tiles: protos,
        }
    }
}