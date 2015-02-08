use std::old_io as io;
use gfx;
use super::chunk::Reader;

pub type Success = (String, gfx::Mesh, gfx::Slice);

fn parse_type(type_: char, normalized: u8) -> Result<gfx::attrib::Type, ()> {
    use gfx::attrib::Type::*;
    use gfx::attrib::IntSubType::*;
    use gfx::attrib::IntSize::*;
    use gfx::attrib::SignFlag::*;
    use gfx::attrib::FloatSubType::Default;
    use gfx::attrib::FloatSize::*;
    Ok(match (type_, normalized) {
        ('b', 0) => Int(Raw, U8, Signed),
        ('B', 0) => Int(Raw, U8, Unsigned),
        ('b', 1) => Int(Normalized, U8, Signed),
        ('B', 1) => Int(Normalized, U8, Unsigned),
        ('s', 0) => Int(Raw, U16, Signed),
        ('S', 0) => Int(Raw, U16, Unsigned),
        ('s', 1) => Int(Normalized, U16, Signed),
        ('S', 1) => Int(Normalized, U16, Unsigned),
        ('l', 0) => Int(Raw, U32, Signed),
        ('L', 0) => Int(Raw, U32, Unsigned),
        ('l', 1) => Int(Normalized, U32, Signed),
        ('L', 1) => Int(Normalized, U32, Unsigned),
        ('h', 0) => Float(Default, F16),
        ('f', 0) => Float(Default, F32),
        _ => return Err(()),
    })
}

#[derive(Debug)]
pub enum Error {
    Path(io::IoError),
    Chunk(String),
    Signature(String),
    Topology(String),
    DoubleIndex,
    AttribType(char, u8),
    IndexType(char),
    Stride(u8),
    Other,
}

pub fn load<R: io::Reader, D: gfx::Device>(
            reader: &mut Reader<R>, device: &mut D)
            -> Result<Success, Error>    {
    use gfx::PrimitiveType;
    let mut cmesh = reader.enter();
    if cmesh.get_name() != "k3mesh"    {
        return Err(Error::Signature(cmesh.get_name().to_string()))
    }
    let mesh_name = cmesh.read_string();
    let n_vert = cmesh.read_u32();
    let topology = cmesh.read_string();
    info!("\tname: {}, vertices: {}", mesh_name, n_vert);
    let mut slice = gfx::Slice {
        start: 0,
        end: n_vert,
        prim_type: match topology.as_slice() {
            "1" => PrimitiveType::Point,
            "2" => PrimitiveType::Line,
            "2s"=> PrimitiveType::LineStrip,
            "3" => PrimitiveType::TriangleList,
            "3s"=> PrimitiveType::TriangleStrip,
            "3f"=> PrimitiveType::TriangleFan,
            _ => return Err(Error::Topology(topology)),
        },
        kind: gfx::SliceKind::Vertex,
    };
    let mut mesh = gfx::Mesh::new(n_vert);
    while cmesh.has_more() {
        let mut cbuf = cmesh.enter();
        let buf_name = cbuf.get_name().to_string();
        match (buf_name.as_slice(), slice.kind) {
            ("buffer", _) => {
                let stride = cbuf.read_u8();
                let format_str = cbuf.read_string();
                debug!("\tBuffer stride: {}, format: {}", stride, format_str);
                let buffer = {
                    let data = cbuf.read_bytes(n_vert * (stride as u32));
                    device.create_buffer_static_raw(data)
                };
                let mut offset = 0;
                let mut ft = format_str.bytes();
                loop {
                    let el_count = match ft.next() {
                        Some(s) => s,
                        None => break,
                    };
                    let type_ = match ft.next() {
                        Some(t) => t as char,
                        None => return Err(Error::Other),
                    };
                    let name = cbuf.read_string();
                    let flags = cbuf.read_u8();
                    debug!("\t\tname: {}, count: {}, type: {}, flags: {}",
                        name, el_count, type_, flags);
                    let normalized = flags & 1;
                    let el_type = match parse_type(type_, normalized) {
                        Ok(t) => t,
                        Err(_) => return Err(Error::AttribType(type_, flags)),
                    };
                    mesh.attributes.push(gfx::Attribute {
                        name: name,
                        buffer: buffer.raw(),
                        format: gfx::attrib::Format {
                            elem_count: el_count,
                            elem_type: el_type,
                            offset: offset as gfx::attrib::Offset,
                            stride: stride as gfx::attrib::Stride,
                            instance_rate: 0,
                        },
                    });
                    offset += el_count * el_type.get_size();
                }
                if offset != stride {
                    return Err(Error::Stride(offset));
                }
            },
            ("index", gfx::SliceKind::Vertex)=> {
                let n_ind = cbuf.read_u32();
                let format = cbuf.read_u8() as char;
                debug!("\tIndex format: {}, count: {}", format, n_ind);
                slice.kind = match format {
                    'B' => {
                        let data = cbuf.read_bytes(n_ind * 1);
                        let buf = device.create_buffer_static_raw(data);
                        gfx::SliceKind::Index8(buf.cast(), 0)
                    },
                    'S' => {
                        let data = cbuf.read_bytes(n_ind * 2);
                        let buf = device.create_buffer_static_raw(data);
                        gfx::SliceKind::Index16(buf.cast(), 0)
                    },
                    'L' => {
                        let data = cbuf.read_bytes(n_ind * 4);
                        let buf = device.create_buffer_static_raw(data);
                        gfx::SliceKind::Index32(buf.cast(), 0)
                    },
                    _ => return Err(Error::IndexType(format)),
                };
            },
            ("index", _) => return Err(Error::DoubleIndex),
            _ => return Err(Error::Chunk(buf_name)),
        }
    }
    Ok((mesh_name, mesh, slice))
}
