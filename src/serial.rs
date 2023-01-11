use std::{
    io::{BufRead, BufReader},
    time::Duration,
};

use tokio_serial::FlowControl;

use crate::read_until::ReadUntilPat;

const SERIAL_TIMEOUT: Duration = Duration::from_secs(5);

pub struct SimpleSerial {
    reader: BufReader<Box<dyn tokio_serial::SerialPort>>,
}

impl SimpleSerial {
    pub fn new(path: &str, baud_rate: u32) -> crate::Result<Self> {
        let port = tokio_serial::new(path, baud_rate)
            .timeout(SERIAL_TIMEOUT)
            .open()?;
        let reader = BufReader::new(port);

        Ok(Self { reader })
    }

    pub fn read_line(&mut self) -> crate::Result<Vec<u8>> {
        let mut line = Vec::new();
        self.reader.read_until(b'\n', &mut line)?;
        Ok(line)
    }
}
