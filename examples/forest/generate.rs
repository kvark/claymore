use rand;
use cgmath;
use gfx;
use gfx_pipeline::Material;
use claymore_scene;
use reflect;


struct TileComponent<R: gfx::Resources> {
    mesh: gfx::Mesh<R>,
    slice: gfx::Slice<R>,
    material: Material<R>,
}

struct TileProto<R: gfx::Resources> {
    node: claymore_scene::NodeId<f32>,
    mesh: gfx::Mesh<R>,
    fragments: Vec<claymore_scene::Fragment<R>>,
    river_mask: u8,

}

impl<R: gfx::Resources> TileProto<R> {
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
    tile_size: f32,
}

impl<R: gfx::Resources> Gen<R> {
    pub fn new(config: &::reflect::Palette, scene: &claymore_scene::Scene<R, f32>) -> Gen<R> {
        println!("Processing data...");
        let protos: Vec<_> = config.tiles.iter().map(|t| {
            let mask = t.river.chars().fold(0, |m, c| match c {
                'n' => m | 1,
                'e' => m | 2,
                's' => m | 4,
                'w' => m | 8,
                _   => panic!("Unknown river direction: {}", c),
            });
            let ent = scene.entities.iter()
                                    .find(|ent| ent.name == t.name)
                                    .expect(&format!("Unable to find entity {}", t.name));
            info!("Found tile {} with river mask {}",
                t.name, mask);
            TileProto {
                node: ent.node.clone(),
                mesh: ent.mesh.clone(),
                fragments: ent.fragments.clone(),
                river_mask: mask,
            }
        }).collect();

        Gen {
            proto_tiles: protos,
            tile_size: config.size,
        }
    }

    pub fn populate(&self, grid_size: (i32, i32),
                    scene: &mut claymore_scene::Scene<R, f32>) {
        use std::collections::HashMap;
        type Position = (i32, i32);
        struct Tile {
            proto_id: usize,
            orientation: u8,
        }
        scene.entities.clear();
        println!("Generating content...");
        let mut rng = rand::thread_rng();
        let mut tile_map: HashMap<Position, Tile> = HashMap::new();
        for y in -grid_size.0 ..grid_size.0 {
            for x in -grid_size.1 ..grid_size.1 {
                use rand::Rng;
                if tile_map.contains_key(&(x,y)) {
                    continue
                }
                debug!("Generating tile {:?}", (x,y));
                // figure out what neighbour edges are rivers
                let mut river_mask = 0;
                let mut neighbour_mask = 0;
                let offsets = [[0, 1], [1, 0], [0,-1], [-1, 0]];
                for (bit, off) in offsets.iter().enumerate() {
                    let pos = (x + off[0], y + off[1]);
                    if let Some(tile) = tile_map.get(&pos) {
                        neighbour_mask |= 1 << bit;
                        let river_bit = ((bit as u8) + 6 - tile.orientation) & 3;
                            debug!("\tChecking for river bit {} of neighbor dir {}", river_bit, bit);
                        let proto = &self.proto_tiles[tile.proto_id];
                        if proto.river_mask & (1 << river_bit) != 0 {
                            river_mask |= 1 << bit;
                        }
                    }
                }
                debug!("\tLooking for river mask {} of neighbors {}", river_mask, neighbour_mask);
                // find a matching prototype
                let mut matched = 0;
                for proto in self.proto_tiles.iter() {
                    if proto.fit_orientation(neighbour_mask, river_mask).is_some() {
                        matched += 1;
                    }
                }
                if matched == 0 {
                    error!("Couldn't find a tile match for {:?}, where neighbors = {}, rivers = {}",
                        (x, y), neighbour_mask, river_mask);
                    continue
                }
                let chosen = rng.gen_range(0, matched);
                debug!("\tChosen match {} of total {}", chosen, matched);
                matched = 0;
                for (id, proto) in self.proto_tiles.iter().enumerate() {
                    match proto.fit_orientation(neighbour_mask, river_mask) {
                        Some(orientation) if matched == chosen => {
                            use cgmath::ToRad;
                            debug!("\tUsing orientation {} and proto id {}", orientation, id);
                            let rotation = {
                                use cgmath::Rotation;
                                use claymore_scene::base::World;
                                let relative: cgmath::Quaternion<_> = cgmath::Rotation3::from_axis_angle(
                                    &cgmath::Vector3::new(0.0, 0.0, -1.0),
                                    cgmath::deg(orientation as f32 * 90.0).to_rad(),
                                );
                                let node = self.proto_tiles[id].node;
                                relative.concat(&scene.world.get_transform(&node).rot)
                            };
                            let (rot_x, rot_y) = [(0, 0), (0, 1), (1, 1), (1, 0)][orientation as usize];
                            let node = scene.world.add_node(
                                format!("Tile ({}, {})", x, y),
                                claymore_scene::space::Parent::None,
                                cgmath::Decomposed {
                                    scale: 1.0,
                                    rot: rotation,
                                    disp: cgmath::Vector3::new(
                                        (x + rot_x) as f32 * self.tile_size,
                                        (y + rot_y) as f32 * self.tile_size,
                                        0.0,
                                    ),
                                });
                            // add the new entity to the scene
                            let proto = &self.proto_tiles[id];
                            scene.entities.push(claymore_scene::base::Entity {
                                name: String::new(),
                                visible: true,
                                mesh: proto.mesh.clone(),
                                node: node,
                                skeleton: None,
                                bound: cgmath::Aabb3::new(
                                    cgmath::Point3::new(0.0, 0.0, 0.0),
                                    cgmath::Point3::new(self.tile_size, 0.5, -self.tile_size),
                                ),
                                fragments: proto.fragments.clone(),
                            });
                            // register the new tile
                            tile_map.insert((x, y), Tile {
                                proto_id: id,
                                orientation: orientation,
                            });
                            break;
                        }
                        Some(_) => {
                            matched += 1;
                        }
                        None => (),
                    }
                }
            }
        }
    }
}
