use std::io::{BufRead, ErrorKind};

use tracing::debug;

pub trait ReadUntilPat {
    fn read_until_pat<const L: usize>(
        &mut self,
        delim: &[u8; L],
        buf: &mut Vec<u8>,
    ) -> std::io::Result<usize>;
}

impl<R: BufRead + ?Sized> ReadUntilPat for R {
    fn read_until_pat<const L: usize>(
        &mut self,
        delim: &[u8; L],
        buf: &mut Vec<u8>,
    ) -> std::io::Result<usize> {
        let mut read = 0;
        loop {
            let (done, used) = {
                let available = match self.fill_buf() {
                    Ok(n) => n,
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                };
                match available
                    .windows(delim.len())
                    .position(|window| window == delim)
                {
                    Some(i) => {
                        buf.extend_from_slice(&available[..=i]);
                        (true, i + 1)
                    }
                    None => {
                        debug!("PAT NOT FOUND");
                        buf.extend_from_slice(available);
                        (false, available.len())
                    }
                }
            };
            self.consume(used);
            read += used;
            if done || used == 0 {
                return Ok(read);
            }
        }
    }
}
