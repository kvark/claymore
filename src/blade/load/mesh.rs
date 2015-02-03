use gfx;

#[derive(Debug)]
pub enum Error {
    Path,
}

pub fn load<D: gfx::Device>(path: &str, device: &mut D)
            -> Result<gfx::Mesh, Error> {
    Err(Error::Path)
}
