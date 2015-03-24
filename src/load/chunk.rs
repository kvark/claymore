use std::old_io as io;
use std::fmt;
use std::ops::{Deref, DerefMut};

static NAME_LENGTH: u32 = 8;

pub struct Root<R> {
    pub name: String,
    input: R,
    buffer: Vec<u8>,
    position: u32,
}

impl<R: io::Reader> Root<R> {
    pub fn new(name: String, input: R) -> Root<R> {
        Root {
            name: name,
            input: input,
            buffer: Vec::new(),
            position: 0,
        }
    }

    pub fn get_pos(&self) -> u32 {
        self.position
    }

    fn skip(&mut self, num: u32) {
        use std::old_io::BytesReader;
        self.position += num;
        let _ = self.input.bytes().skip(num as usize);
    }

    pub fn read_bytes(&mut self, num: u32) -> &[u8] {
        self.position += num;
        self.buffer.truncate(0);
        for _ in (0.. num) {
            let b = self.input.read_u8().unwrap();
            self.buffer.push(b);
        }
        self.buffer.as_slice()
    }

    pub fn read_u8(&mut self) -> u8 {
        self.position += 1;
        self.input.read_u8().unwrap()
    }

    pub fn read_u32(&mut self) -> u32 {
        self.position += 4;
        self.input.read_le_u32().unwrap()
    }

    pub fn read_bool(&mut self) -> bool {
        self.position += 1;
        self.input.read_u8().unwrap() != 0
    }

    pub fn read_string(&mut self) -> String {
        let size = self.input.read_u8().unwrap();
        self.position += 1 + (size as u32);
        let buf = self.input.read_exact(size as usize).unwrap();
        String::from_utf8(buf).unwrap()
    }

    pub fn enter<'b>(&'b mut self) -> Chunk<'b, R> {
        self.position += 4 + NAME_LENGTH;
        let name = {
            let raw = self.read_bytes(NAME_LENGTH);
            let buf = match raw.position_elem(&0) {
                Some(p) => &raw[..p],
                None => raw,
            };
            String::from_utf8_lossy(buf)
                .into_owned()
        };
        debug!("Entering chunk {}", name);
        let size = self.read_u32();
        Chunk    {
            name: name,
            size: size,
            end_pos: self.position + size,
            root: self,
        }
    }
}

pub struct Chunk<'a, R: 'a> {
    name: String,
    size: u32,
    end_pos: u32,
    root: &'a mut Root<R>,
}

impl<'a, R> fmt::Display for Chunk<'a, R> {
    fn fmt(&self, fm: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fm, "Chunk({}, {} left)", self.name, self.size)
    }
}

impl<'a, R: io::Reader> Chunk<'a, R> {
    pub fn get_name(&self) -> &str {
        self.name.as_slice()
    }

    pub fn has_more(&self)-> bool {
        self.root.get_pos() < self.end_pos
    }

    pub fn ignore(self) {
        let left = self.end_pos - self.root.get_pos();
        self.root.skip(left)
    }
}

#[unsafe_destructor]
impl<'a, R: io::Reader> Drop for Chunk<'a, R> {
    fn drop(&mut self) {
        debug!("Leaving chunk");
        assert!(!self.has_more())
    }
}

impl<'a, R> Deref for Chunk<'a, R> {
    type Target = Root<R>;
    fn deref(&self) -> &Root<R> {
        self.root
    }
}

impl<'a, R> DerefMut for Chunk<'a, R> {
    fn deref_mut(&mut self) -> &mut Root<R> {
        self.root
    }
}
