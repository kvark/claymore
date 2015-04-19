use std::collections::HashMap;

pub type WorldVector = (f32, f32, f32);
pub type CellPosition = (i8, i8);

#[derive(RustcDecodable)]
pub struct Game {
    pub name: String,
    pub characters: HashMap<String, GameChar>,
    pub level: Level,
}

#[derive(RustcDecodable)]
pub struct GameChar {
    pub scene: String,
    pub health: u32,
}

#[derive(RustcDecodable)]
pub struct Level {
    pub scene: String,
    pub grid: Grid,
    pub characters: HashMap<String, LevelChar>,
}

#[derive(RustcDecodable)]
pub struct Grid {
    pub center: WorldVector,
    pub size: f32,
}

#[derive(RustcDecodable)]
pub struct LevelChar {
    pub team: u8,
    pub cell: CellPosition,
    pub scale: f32,
}
