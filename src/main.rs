#![crate_name = "claymore"]
#![crate_type = "bin"]

extern crate blade;

fn main() {
    let _scene = blade::scene::load_json("data/test");
}
