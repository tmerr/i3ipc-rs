use std::io;
use std::io::{Read, Result};

pub fn read_exact<R: Read + ?Sized>(rdr: &mut R, sz: usize) -> io::Result<Vec<u8>> {
    let mut vec = vec![0; sz];
    let mut nread = 0usize;
    while nread < sz {
        match rdr.read(&mut vec[nread..]) {
            Ok(0) => return Err(io::Error::new(io::ErrorKind::Other, "Unexpected end of file.")),
            Ok(n) => nread += n,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {},
            Err(e) => return Err(From::from(e))
        }
    }
    Ok(vec)
}
