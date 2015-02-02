#![crate_name = "claymore"]
#![crate_type = "bin"]

extern crate blade;

fn main() {
    println!("Reading test scene...");
    let _scene = blade::scene::load_json("data/test").unwrap();
    println!("Done.");
}
