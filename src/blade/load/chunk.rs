use std::old_io as io;

static NAME_LENGTH: u32 = 8;

pub struct Root<R> {
    pub name: String,
    input: R,
    buffer: Vec<u8>,
}

impl<R> Root<R> {
    pub fn new(name: String, input: R) -> Root<R> {
        Root {
            name: name,
            input: input,
            buffer: Vec::new(),
        }
    }
}

pub struct Chunk<'a, R: 'a> {
    name: String,
    size: u32,
    root: &'a mut Root<R>,
}

pub trait Reader<R> {
    fn read_bytes(&mut self, num: u32) -> &[u8];
    fn read_u8(&mut self) -> u8;
    fn read_u32(&mut self) -> u32;
    fn read_bool(&mut self) -> bool;
    fn read_string(&mut self) -> String;
    fn enter<'b>(&'b mut self) -> Chunk<'b, R>;
}

impl<R: io::Reader> Reader<R> for Root<R> {
    fn read_bytes(&mut self, num: u32) -> &[u8] {
        self.buffer.truncate(0);
        for _ in (0.. num) {
            let b = self.input.read_u8().unwrap();
            self.buffer.push(b);
        }
        self.buffer.as_slice()
    }

    fn read_u8(&mut self) -> u8 {
        self.input.read_u8().unwrap()
    }

    fn read_u32(&mut self) -> u32 {
        self.input.read_le_u32().unwrap()
    }

    fn read_bool(&mut self) -> bool {
        self.input.read_u8().unwrap() != 0
    }

    fn read_string(&mut self) -> String {
        let size = self.input.read_u8().unwrap();
        let buf = self.input.read_exact(size as usize).unwrap();
        String::from_utf8(buf).unwrap()
    }

    fn enter<'b>(&'b mut self) -> Chunk<'b, R> {
        let name = {
            let raw = self.read_bytes(NAME_LENGTH);
            let buf = match raw.position_elem(&0) {
                Some(p) => &raw[..p],
                None => raw,
            };
            String::from_utf8_lossy(buf)
                .into_owned()
        };
        let size = self.read_u32();
        Chunk    {
            name: name,
            size: size,
            root: self,
        }
    }
}

impl<'a, R: io::Reader> Chunk<'a, R> {
    pub fn get_name(&self) -> &str {
        self.name.as_slice()
    }

    pub fn has_more(&self)-> bool {
        self.size != 0
    }

    pub fn skip(&mut self) {
        use std::old_io::BytesReader;
        let _ = self.root.input.bytes()
            .skip(self.size as usize);
        self.size = 0;
    }

    fn count(&mut self, num: u32) {
        assert!(self.size >= num);
        self.size -= num;
    }
}

/*impl<'a, R> Drop for Chunk<'a, R> {
    fn drop(&mut self) {
        assert_eq!(self.size, 0)
    }
}*/

impl<'a, R: io::Reader> Reader<R> for Chunk<'a, R> {
    fn read_bytes(&mut self, num: u32) -> &[u8] {
        self.count(num);
        self.root.read_bytes(num)
    }

    fn read_u8(&mut self) -> u8 {
        self.count(1);
        self.root.read_u8()
    }

    fn read_u32(&mut self) -> u32 {
        self.count(4);
        self.root.read_u32()
    }

    fn read_bool(&mut self) -> bool {
        self.count(1);
        self.root.read_bool()
    }

    fn read_string(&mut self) -> String {
        let s = self.root.read_string();
        self.count(s.len() as u32 + 1);
        s
    }

    fn enter<'b>(&'b mut self) -> Chunk<'b, R> {
        self.count(NAME_LENGTH + 4);
        self.root.enter()
    }
}
