#[derive(RustcDecodable)]
pub struct Demo {
    pub name: String,
    pub generate: bool,
    pub size: (i32, i32),
    pub debug: Debug,
    pub palette: Palette,
}

#[derive(RustcDecodable)]
pub struct Debug {
    pub offset: (i32, i32),
    pub line_jump: i32,
    pub color: (f32, f32, f32, f32),
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
