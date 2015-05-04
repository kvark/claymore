#[derive(RustcDecodable)]
pub struct Demo {
    pub name: String,
    pub generate: bool,
    pub size: (i32, i32),
    pub palette: Palette,
}

#[derive(RustcDecodable)]
pub struct Palette {
    pub scene: String,
    pub size: f32,
    pub tiles: Vec<Tile>,
}

#[derive(RustcDecodable)]
pub struct Tile {
    pub name: String,
    pub river: String,
}
