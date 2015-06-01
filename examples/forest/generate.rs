use std::ops;
use rand;
use cgmath;
use gfx;
use claymore_scene;
use reflect;


#[derive(Clone, Copy, Debug, PartialEq)]
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

impl Direction {
    pub fn from_char(c: char) -> Result<Direction, char> {
        Ok(match c {
            'n' => Direction::North,
            'e' => Direction::East,
            's' => Direction::South,
            'w' => Direction::West,
            _   => return Err(c)
        })
    }

    pub fn to_vector(self) -> [i32; 2] {
        [[0, 1], [1, 0], [0,-1], [-1, 0]][self as usize]
    }

    pub fn to_degrees(self) -> f32 {
        (self as u8) as f32 * 90.0
    }

    pub fn aligned_as(self, das: Direction, dto: Direction) -> Direction {
        match ((self as u8) + 4 + (das as u8) - (dto as u8)) & 3 {
            0 => Direction::North,
            1 => Direction::East,
            2 => Direction::South,
            3 => Direction::West,
            _ => panic!("bad direction")
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DirectionSet(u8);

const SET_EMPTY     : DirectionSet = DirectionSet(0);
const SET_VERTICAL  : DirectionSet = DirectionSet(5);
const SET_HORISONTAL: DirectionSet = DirectionSet(12);

impl DirectionSet {
    pub fn from_str(s: &str) -> Result<DirectionSet, char> {
        let mut set = SET_EMPTY;
        for c in s.chars() {
            set = set | try!(Direction::from_char(c));
        }
        Ok(set)
    }

    pub fn has(&self, d: Direction) -> bool {
        self.0 & (1 << (d as u8)) != 0
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
    bound: cgmath::Aabb3<f32>,
    fragments: Vec<claymore_scene::Fragment<R>>,
}

impl<R: gfx::Resources> Drawable<R> {
    pub fn new(ent: &claymore_scene::Entity<R, f32>) -> Drawable<R> {
        Drawable {
            node: ent.node.clone(),
            mesh: ent.mesh.clone(),
            bound: ent.bound.clone(),
            fragments: ent.fragments.clone(),
        }
    }
}


struct TileProto<R: gfx::Resources> {
    drawable: Drawable<R>,
    river_mask: DirectionSet,
}

impl<R: gfx::Resources> TileProto<R> {
    pub fn fit_orientation(&self, dir: Direction, neighbors: DirectionSet,
                           rivers: DirectionSet) -> bool {
        (self.river_mask >> dir) & neighbors == rivers
    }
}

struct Tile {
    proto_id: usize,
    orientation: Direction,
    node: claymore_scene::NodeId<f32>,
}


pub struct Gen<R: gfx::Resources> {
    proto_tiles: Vec<TileProto<R>>,
    water_plants: Vec<Drawable<R>>,
    plants: Vec<Drawable<R>>,
    tents: Vec<Drawable<R>>,
    camp_fires: Vec<Drawable<R>>,
    tile_size: f32,
}

impl<R: gfx::Resources> Gen<R> {
    pub fn new(config: &reflect::Palette, scene: &claymore_scene::Scene<R, f32>) -> Gen<R> {
        println!("Processing data...");
        let protos: Vec<_> = config.tiles.iter().map(|(name, river)| {
            let ent = scene.entities.iter()
                                    .find(|ent| &ent.name == name)
                                    .expect(&format!("Unable to find entity {}", name));
            info!("Found tile {} with river mask {}", name, river);
            TileProto {
                drawable: Drawable::new(ent),
                river_mask: DirectionSet::from_str(river).unwrap(),
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
        let camp_fires = config.tents.iter().map(|name| Drawable::new(
            scene.entities.iter()
                          .find(|ent| &ent.name == name)
                          .expect(&format!("Unable to find camp fire {}", name))
        )).collect();
        Gen {
            proto_tiles: protos,
            water_plants: water_plants,
            plants: plants,
            tents: tents,
            camp_fires: camp_fires,
            tile_size: config.size,
        }
    }

    fn get_water_spots(&self, river_mask: DirectionSet) -> Vec<(f32, f32)> {
        let mut spots = Vec::new();
        if river_mask == SET_VERTICAL || river_mask == SET_HORISONTAL {
            spots.push((0.5, 0.5))
        }
        if river_mask.has(Direction::North) {
            spots.push((0.5, 0.8));
        }
        if river_mask.has(Direction::East) {
            spots.push((0.8, 0.5));
        }
        if river_mask.has(Direction::South) {
            spots.push((0.5, 0.2));
        }
        if river_mask.has(Direction::West) {
            spots.push((0.2, 0.5));
        }
        spots
    }

    fn get_grass_spots(&self, river_mask: DirectionSet, has_tent: bool)
                       -> Vec<(f32, f32)> {
        if has_tent {
            return Vec::new()
        }
        let low = 0.15;
        let mid = 0.5;
        let hai = 0.85;
        let mut spots = vec![
            (low, low),
            (hai, low),
            (low, hai),
            (hai, hai),
        ];
        if river_mask == SET_EMPTY {
            spots.push((mid, mid));
        }
        if !river_mask.has(Direction::North) {
            spots.push((mid, hai));
        }
        if !river_mask.has(Direction::East) {
            spots.push((hai, mid));
        }
        if !river_mask.has(Direction::South) {
            spots.push((mid, low));
        }
        if !river_mask.has(Direction::West) {
            spots.push((low, mid));
        }
        spots
    }

    fn make_tile(&self, x: i32, y: i32, proto_id: usize, orientation: Direction,
                 world: &mut claymore_scene::World<f32>)
                 -> claymore_scene::Entity<R, f32> {
        let drawable = &self.proto_tiles[proto_id].drawable;
        debug!("\tUsing orientation {:?} and proto id {}", orientation, proto_id);
        let rotation = {
            use cgmath::Rotation;
            let relative: cgmath::Quaternion<_> = cgmath::Rotation3::from_axis_angle(
                &cgmath::Vector3::new(0.0, 0.0, -1.0),
                cgmath::deg(orientation.to_degrees()).into(),
            );
            relative.concat(&world.get_node(drawable.node).world.rot)
        };
        let (rot_x, rot_y) = match orientation {
            Direction::North => (0, 0),
            Direction::East => (0, 1),
            Direction::South => (1, 1),
            Direction::West => (1, 0),
        };
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
        claymore_scene::Entity {
            name: String::new(),
            visible: true,
            mesh: drawable.mesh.clone(),
            node: node,
            skeleton: None,
            bound: drawable.bound.clone(),
            fragments: drawable.fragments.clone(),
        }
    }

    fn make_prop(&self, base_node: claymore_scene::NodeId<f32>,
                 drawable: &Drawable<R>, position: (f32, f32), z: f32,
                 world: &mut claymore_scene::World<f32>)
                 -> claymore_scene::Entity<R, f32> {
        use cgmath::{Aabb, Point, Rotation, Transform};
        let rotation = world.get_node(drawable.node)
                            .local.rot.clone();
        let mut bound_center = rotation.rotate_point(&drawable.bound.center());
        bound_center.z = 0.0;
        let offset = cgmath::Point3::new(
            position.0 * self.tile_size,
            z,
            -position.1 * self.tile_size,
        );
        let translation = world.get_node(base_node)
                               .local.transform_point(&offset);
        debug!("Found spot {:?}, ended up at pos {:?}", position, translation);
        let node = world.add_node(
            String::new(),
            claymore_scene::space::Parent::None,
            cgmath::Decomposed {
                scale: 1.0,
                rot: rotation,
                disp: translation.sub_p(&bound_center),
            });
        claymore_scene::Entity {
            name: String::new(),
            visible: true,
            mesh: drawable.mesh.clone(),
            node: node,
            skeleton: None,
            bound: drawable.bound.clone(),
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
                let mut river_mask = SET_EMPTY;
                let mut neighbour_mask = SET_EMPTY;
                for dir in ALL_DIRECTIONS {
                    let offset = dir.to_vector();
                    let pos = (x + offset[0], y + offset[1]);
                    if let Some(tile) = tile_map.get(&pos) {
                        neighbour_mask = neighbour_mask | *dir;
                        let river_bit = dir.aligned_as(Direction::South, tile.orientation);
                            debug!("\tChecking for river dir {:?} of neighbor dir {:?}", river_bit, dir);
                        let proto = &self.proto_tiles[tile.proto_id];
                        if proto.river_mask.has(river_bit) {
                            river_mask = river_mask | *dir;
                        }
                    }
                }
                debug!("\tLooking for river mask {:?} of neighbors {:?}", river_mask, neighbour_mask);
                // find a matching prototype
                let mut matched = 0;
                for proto in self.proto_tiles.iter() {
                    for dir in ALL_DIRECTIONS {
                        if proto.fit_orientation(*dir, neighbour_mask, river_mask) {
                            matched += 1;
                        }
                    }
                }
                if matched == 0 {
                    error!("Couldn't find a tile match for {:?}, where neighbors = {:?}, rivers = {:?}",
                        (x, y), neighbour_mask, river_mask);
                    continue
                }
                let chosen = rng.gen_range(0, matched);
                debug!("\tChosen match {} of total {}", chosen, matched);
                matched = 0;
                'proto: for (id, proto) in self.proto_tiles.iter().enumerate() {
                    for dir in ALL_DIRECTIONS {
                        if !proto.fit_orientation(*dir, neighbour_mask, river_mask) {
                            continue;
                        }
                        if matched < chosen {
                            matched += 1;
                            continue
                        }
                        let entity = self.make_tile(x, y, id, *dir, &mut scene.world);
                        tile_map.insert((x, y), Tile {
                            proto_id: id,
                            orientation: *dir,
                            node: entity.node.clone(),
                        });
                        scene.entities.push(entity);
                        break 'proto;
                    }
                }
            }
        }
        // place props
        for (&(x, y), tile) in tile_map.iter() {
            use rand::Rng;
            let river_mask = self.proto_tiles[tile.proto_id].river_mask;
            // water plants
            if river_mask != SET_EMPTY && rng.next_f32() < model.water_plant_chance {
                let plant_type = rng.gen_range(0, self.water_plants.len());
                debug!("Generating water plant type {} on tile ({}, {}) with mask {:?}",
                    plant_type, x, y, river_mask);
                let spots = self.get_water_spots(river_mask);
                let position = spots[rng.gen_range(0, spots.len())];
                let entity = self.make_prop(tile.node, &self.water_plants[plant_type],
                    position, model.water_height, &mut scene.world);
                scene.entities.push(entity);
            }
            // tents
            let mut has_tent = false;
            if river_mask == SET_EMPTY && rng.next_f32() < model.tent_chance {
                let tent_type = rng.gen_range(0, self.tents.len());
                let tent_entity = self.make_prop(tile.node, &self.tents[tent_type],
                    (0.5, 0.5), model.ground_height, &mut scene.world);
                scene.entities.push(tent_entity);
                let fire_type = rng.gen_range(0, self.camp_fires.len());
                let fire_entity = self.make_prop(tile.node, &self.camp_fires[fire_type],
                    (0.5, 1.1), model.ground_height, &mut scene.world);
                scene.entities.push(fire_entity);
                debug!("Generated tent type {} with fire type {} on tile ({}, {})",
                    tent_type, fire_type, x, y);
                has_tent = true;
            }
            // plants
            let mut spots = self.get_grass_spots(river_mask, has_tent);
            let max_plants = if river_mask != SET_EMPTY || has_tent {
                model.max_river_plants
            } else {
                model.max_grass_plants
            };
            for _ in 0.. max_plants {
                if spots.is_empty() || rng.next_f32() >= model.plant_chance {
                    continue
                }
                let plant_type = rng.gen_range(0, self.plants.len());
                debug!("Generating plant type {} on tile ({}, {}) with mask {:?}",
                    plant_type, x, y, river_mask);
                let spot_id = rng.gen_range(0, spots.len());
                let position = spots.swap_remove(spot_id);
                let entity = self.make_prop(tile.node, &self.plants[plant_type],
                    position, model.ground_height, &mut scene.world);
                scene.entities.push(entity);
            }
        }
    }
}
