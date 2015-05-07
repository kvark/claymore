use std::ops;
use rand;
use cgmath;
use gfx;
use claymore_scene;
use reflect;


#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum Direction {
    North,
    East,
    South,
    West,
}

const ALL_DIRECTIONS: &'static [Direction] = &[
    Direction::North, Direction::East,
    Direction::South, Direction::West
];

#[derive(Clone, Copy, PartialEq)]
struct DirectionSet(u8);

const SET_EMPTY: DirectionSet = DirectionSet(0);

impl Direction {
    pub fn from_char(c: char) -> Direction {
        match c {
            'n' => Direction::North,
            'e' => Direction::East,
            's' => Direction::South,
            'w' => Direction::West,
            _   => panic!("Unknown direction: {}", c),
        }
    }

    pub fn to_vector(&self) -> [i32; 2] {
        [[0, 1], [1, 0], [0,-1], [-1, 0]][*self as usize]
    }
}

impl ops::Shr<Direction> for DirectionSet {
    type Output = DirectionSet;
    fn shr(self, rhs: Direction) -> DirectionSet {
        let d = rhs as u8;
        let m = self.0;
        DirectionSet(((m << d) & 0xF) | (m >> (4 - d)))
    }
}

impl ops::BitOr<Direction> for DirectionSet {
    type Output = DirectionSet;
    fn bitor(self, rhs: Direction) -> DirectionSet {
        DirectionSet(self.0 | (1 << (rhs as u8)))
    }
}

impl ops::BitAnd for DirectionSet {
    type Output = DirectionSet;
    fn bitand(self, rhs: DirectionSet) -> DirectionSet {
        DirectionSet(self.0 & rhs.0)
    }
}


struct Drawable<R: gfx::Resources> {
    node: claymore_scene::NodeId<f32>,
    mesh: gfx::Mesh<R>,
    fragments: Vec<claymore_scene::Fragment<R>>,
}

impl<R: gfx::Resources> Drawable<R> {
    pub fn new(ent: &claymore_scene::Entity<R, f32>) -> Drawable<R> {
        Drawable {
            node: ent.node.clone(),
            mesh: ent.mesh.clone(),
            fragments: ent.fragments.clone(),
        }
    }
}


struct TileProto<R: gfx::Resources> {
    drawable: Drawable<R>,
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

struct Tile {
    proto_id: usize,
    orientation: u8,
    node: claymore_scene::NodeId<f32>,
}


pub struct Gen<R: gfx::Resources> {
    proto_tiles: Vec<TileProto<R>>,
    water_plants: Vec<Drawable<R>>,
    plants: Vec<Drawable<R>>,
    tents: Vec<Drawable<R>>,
    tile_size: f32,
}

impl<R: gfx::Resources> Gen<R> {
    pub fn new(config: &reflect::Palette, scene: &claymore_scene::Scene<R, f32>) -> Gen<R> {
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
                drawable: Drawable::new(ent),
                river_mask: mask,
            }
        }).collect();
        let water_plants = config.water_plants.iter().map(|name| Drawable::new(
            scene.entities.iter()
                          .find(|ent| &ent.name == name)
                          .expect(&format!("Unable to find water plant {}", name))
        )).collect();
        let plants = config.plants.iter().map(|name| Drawable::new(
            scene.entities.iter()
                          .find(|ent| &ent.name == name)
                          .expect(&format!("Unable to find plant {}", name))
        )).collect();
        let tents = config.tents.iter().map(|name| Drawable::new(
            scene.entities.iter()
                          .find(|ent| &ent.name == name)
                          .expect(&format!("Unable to find tent {}", name))
        )).collect();
        Gen {
            proto_tiles: protos,
            water_plants: water_plants,
            plants: plants,
            tents: tents,
            tile_size: config.size,
        }
    }

    fn get_water_spots(&self, river_mask: u8) -> Vec<(f32, f32)> {
        let mut spots = Vec::new();
        if river_mask == 5 || river_mask == 12 {
            spots.push((0.5, 0.5))
        }
        if river_mask & 1 != 0 {
            spots.push((0.5, 0.8));
        }
        if river_mask & 2 != 0 {
            spots.push((0.8, 0.5));
        }
        if river_mask & 4 != 0 {
            spots.push((0.5, 0.2));
        }
        if river_mask & 8 != 0 {
            spots.push((0.2, 0.5));
        }
        spots
    }

    fn get_grass_spots(&self, river_mask: u8, has_tent: bool)
                       -> Vec<(f32, f32)> {
        let mut spots = vec![
            (0.1, 0.1),
            (0.9, 0.1),
            (0.1, 0.9),
            (0.9, 0.9),
        ];
        if river_mask == 0 && !has_tent {
            spots.push((0.5, 0.5));
        }
        if river_mask & 1 == 0 {
            spots.push((0.5, 0.9));
        }
        if river_mask & 2 == 0 {
            spots.push((0.9, 0.5));
        }
        if river_mask & 4 == 0 {
            spots.push((0.5, 0.1));
        }
        if river_mask & 8 == 0 {
            spots.push((0.1, 0.5));
        }
        spots
    }

    fn make_tile(&self, x: i32, y: i32, proto_id: usize, orientation: u8,
                 world: &mut claymore_scene::World<f32>)
                 -> claymore_scene::Entity<R, f32> {
        use cgmath::ToRad;
        let drawable = &self.proto_tiles[proto_id].drawable;
        debug!("\tUsing orientation {} and proto id {}", orientation, proto_id);
        let rotation = {
            use cgmath::Rotation;
            use claymore_scene::base::World;
            let relative: cgmath::Quaternion<_> = cgmath::Rotation3::from_axis_angle(
                &cgmath::Vector3::new(0.0, 0.0, -1.0),
                cgmath::deg(orientation as f32 * 90.0).to_rad(),
            );
            relative.concat(&world.get_transform(&drawable.node).rot)
        };
        let (rot_x, rot_y) = [(0, 0), (0, 1), (1, 1), (1, 0)][orientation as usize];
        let node = world.add_node(
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
        claymore_scene::base::Entity {
            name: String::new(),
            visible: true,
            mesh: drawable.mesh.clone(),
            node: node,
            skeleton: None,
            bound: cgmath::Aabb3::new(
                cgmath::Point3::new(0.0, 0.0, 0.0),
                cgmath::Point3::new(self.tile_size, 0.5, -self.tile_size),
            ),
            fragments: drawable.fragments.clone(),
        }
    }

    fn make_prop(&self, base_node: claymore_scene::NodeId<f32>,
                 drawable: &Drawable<R>, position: (f32, f32), z: f32,
                 size: cgmath::Point3<f32>,
                 world: &mut claymore_scene::World<f32>)
                 -> claymore_scene::Entity<R, f32> {
        use cgmath::Transform;
        let rotation = world.get_node(drawable.node)
                            .local.rot.clone();
        let local = cgmath::Vector3::new(
            position.0 * self.tile_size,    //TODO: offset center?
            z,
            -position.1 * self.tile_size,
        );
        let translation = world.get_node(base_node)
                               .local.transform_as_point(&local);
        debug!("Found spot {:?}, ended up at pos {:?}", position, translation);
        let node = world.add_node(
            String::new(),
            claymore_scene::space::Parent::None,
            cgmath::Decomposed {
                scale: 1.0,
                rot: rotation,
                disp: translation,
            });
        claymore_scene::base::Entity {
            name: String::new(),
            visible: true,
            mesh: drawable.mesh.clone(),
            node: node,
            skeleton: None,
            bound: cgmath::Aabb3::new(
                cgmath::Point3::new(0.0, 0.0, 0.0),
                size,
            ),
            fragments: drawable.fragments.clone(),
        }
    }

    pub fn populate(&self, model: &reflect::Model,
                    scene: &mut claymore_scene::Scene<R, f32>) {
        use std::collections::HashMap;
        type Position = (i32, i32);
        scene.entities.clear();
        println!("Generating content...");
        let mut rng = rand::thread_rng();
        let mut tile_map: HashMap<Position, Tile> = HashMap::new();
        for y in -model.grid_size.0 ..model.grid_size.0 {
            for x in -model.grid_size.1 ..model.grid_size.1 {
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
                            let entity = self.make_tile(x, y, id, orientation, &mut scene.world);
                            tile_map.insert((x, y), Tile {
                                proto_id: id,
                                orientation: orientation,
                                node: entity.node.clone(),
                            });
                            scene.entities.push(entity);
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
        // place props
        for (&(x, y), tile) in tile_map.iter() {
            use rand::Rng;
            let river_mask = self.proto_tiles[tile.proto_id].river_mask;
            // water plants
            if river_mask != 0 && rng.next_f32() < model.water_plant_chance {
                let plant_type = rng.gen_range(0, self.water_plants.len());
                debug!("Generating water plant type {} on tile ({}, {}) with mask {}",
                    plant_type, x, y, river_mask);
                let spots = self.get_water_spots(river_mask);
                let position = spots[rng.gen_range(0, spots.len())];
                let entity = self.make_prop(tile.node, &self.water_plants[plant_type],
                    position, 0.15, cgmath::Point3::new(1.0, 0.2, -1.0),
                    &mut scene.world);
                scene.entities.push(entity);
            }
            // tents
            let mut has_tent = false;
            if river_mask == 0 && rng.next_f32() < model.tent_chance {
                let tent_type = rng.gen_range(0, self.tents.len());
                debug!("Generating tent type {} on tile ({}, {})", tent_type, x, y);
                let entity = self.make_prop(tile.node, &self.tents[tent_type],
                    (0.5, 0.5), 0.2, cgmath::Point3::new(3.0, 3.0, -3.0),
                    &mut scene.world);
                scene.entities.push(entity);
                has_tent = true;
            }
            // plants
            let mut spots = self.get_grass_spots(river_mask, has_tent);
            let max_plants = if river_mask != 0 || has_tent {
                model.max_river_plants
            } else {
                model.max_grass_plants
            };
            for _ in 0.. max_plants {
                if spots.is_empty() || rng.next_f32() >= model.plant_chance {
                    continue
                }
                let plant_type = rng.gen_range(0, self.plants.len());
                debug!("Generating plant type {} on tile ({}, {}) with mask {}",
                    plant_type, x, y, river_mask);
                let spot_id = rng.gen_range(0, spots.len());
                let position = spots.swap_remove(spot_id);
                let entity = self.make_prop(tile.node, &self.plants[plant_type],
                    position, 0.2, cgmath::Point3::new(3.0, 6.0, -3.0),
                    &mut scene.world);
                scene.entities.push(entity);
            }
        }
    }
}
