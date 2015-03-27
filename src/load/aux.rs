use std::io;

pub trait ReadExt {
    fn read_u8(&mut self) -> Result<u8, io::Error>;
    fn read_u32(&mut self) -> Result<u32, io::Error>;
}

impl<R: io::Read> ReadExt for R {
    fn read_u8(&mut self) -> Result<u8, io::Error> {
        let mut res = [0u8; 1];
        self.read(&mut res).map(|_| res[0])
    }

    fn read_u32(&mut self) -> Result<u32, io::Error> {
        let mut buf = [0u8; 4];
        self.read(&mut buf).map(|_| {
            ((buf[0] as u32) << 24) | ((buf[1] as u32) << 16) |
            ((buf[2] as u32) << 8) | ((buf[3] as u32) << 0)
        })
    }
}
