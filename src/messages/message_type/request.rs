use std::io::Write;
use std::net::TcpStream;

pub struct Request {
    len: u32,
    id: u8,
    index: u32,
    begin: u32,
    length: u32,
}

impl Request {
    pub fn new(index: u32, begin: u32, length: u32) -> Request {
        Request {
            len: 13,
            id: 6,
            index,
            begin,
            length,
        }
    }

    pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let size = self.length as i32 - self.begin as i32;
        if size <= 16384 {
            stream.write_all(&self.len.to_be_bytes())?;
            stream.write_all(&[self.id])?;
            stream.write_all(&self.index.to_be_bytes())?;
            stream.write_all(&self.begin.to_be_bytes())?;
            stream.write_all(&self.length.to_be_bytes())?;
        }
        Ok(())
    }
}
