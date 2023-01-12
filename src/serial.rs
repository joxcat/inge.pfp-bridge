use std::{
    io::{BufReader, BufWriter, Write},
    time::Duration,
};

use tracing::debug;

use crate::read_until::ReadUntilPat;

const SERIAL_TIMEOUT: Duration = Duration::from_secs(5);

pub struct SimpleSerial {
    serial: Box<dyn tokio_serial::SerialPort>,
}

impl SimpleSerial {
    pub fn new(path: &str, baud_rate: u32) -> crate::Result<Self> {
        let serial = tokio_serial::new(path, baud_rate)
            .baud_rate(baud_rate)
            .timeout(SERIAL_TIMEOUT)
            .open()?;
        Ok(Self { serial })
    }

    pub fn read_line(&mut self) -> crate::Result<Vec<u8>> {
        let mut line = Vec::new();
        let mut reader = BufReader::new(&mut self.serial);
        reader.read_until_pat(b"\r\n", &mut line)?;
        Ok(line)
    }

    pub fn write_buf(&mut self, buf: &[u8]) -> crate::Result<()> {
        let buf = [buf, b"\r\n"].concat();
        let mut writer = BufWriter::new(&mut self.serial);
        debug!(packet = crate::logger::slice_to_hex(&buf), "buf");
        writer.write_all(&buf)?;
        writer.flush()?;
        Ok(())
    }
}
