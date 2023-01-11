use std::{io::BufReader, time::Duration};

use tokio_serial::FlowControl;

use crate::read_until::ReadUntilPat;

const SERIAL_TIMEOUT: Duration = Duration::from_secs(5);

pub struct SimpleSerial {
    reader: BufReader<Box<dyn tokio_serial::SerialPort>>,
}

impl SimpleSerial {
    pub fn new(path: &str, baud_rate: u32) -> crate::Result<Self> {
        let port = tokio_serial::new(path, baud_rate)
            .baud_rate(baud_rate)
            .flow_control(FlowControl::Software)
            .timeout(SERIAL_TIMEOUT)
            .open()?;
        let reader = BufReader::new(port);

        Ok(Self { reader })
    }

    pub fn read_line(&mut self) -> crate::Result<Vec<u8>> {
        let mut line = Vec::new();
        self.reader.read_until_pat(b"\r\n", &mut line)?;
        Ok(line)
    }
}
