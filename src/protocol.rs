use serde::{Deserialize, Serialize};

pub const PAYLOAD_SIZE: usize = 32;

#[derive(Debug)]
pub enum Command {
    HeloP,
    OlehP,
    Ident,
    Tnedi,
    TrustB,
    TrustT,
    HeloL,
    OlehL,
    Push,
    PushAck,
    ADeny,
    UDeny,
    Add,
    Del,
}

impl From<Command> for u8 {
    fn from(value: Command) -> Self {
        match value {
            Command::HeloP => 0x00,
            Command::OlehP => 0x01,
            Command::Ident => 0x02,
            Command::Tnedi => 0x03,
            Command::TrustB => 0x04,
            Command::TrustT => 0x05,
            Command::HeloL => 0x06,
            Command::OlehL => 0x07,
            Command::Push => 0x08,
            Command::PushAck => 0x09,
            Command::ADeny => 0x0A,
            Command::UDeny => 0x0B,
            Command::Add => 0x0C,
            Command::Del => 0x0D,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PFPRequest {
    pub command_id: u8,
    pub hop_count: u8,
    pub source_addr: u32,
    pub dest_addr: u32,
    pub request_id: u8,
    pub request_part: u8,
    pub request_count: u8,
    pub payload: [u8; PAYLOAD_SIZE],
}

impl PFPRequest {
    pub fn new_helop(source_addr: u32) -> Self {
        Self {
            command_id: Command::HeloP.into(),
            hop_count: 0,
            source_addr,
            dest_addr: 0,
            request_id: 1,
            request_part: 0,
            request_count: 1,
            payload: [0; PAYLOAD_SIZE],
        }
    }

    pub fn new_add(source_addr: u32, new_device_addr: u32) -> Self {
        let mut payload = [0; PAYLOAD_SIZE];
        payload[0..4].copy_from_slice(&new_device_addr.to_le_bytes());

        Self {
            command_id: Command::Add.into(),
            hop_count: 0,
            source_addr,
            dest_addr: 0,
            request_id: 1,
            request_part: 0,
            request_count: 1,
            payload,
        }
    }

    pub fn new_del(source_addr: u32, lost_device_addr: u32) -> Self {
        let mut payload = [0; PAYLOAD_SIZE];
        payload[0..4].copy_from_slice(&lost_device_addr.to_le_bytes());

        Self {
            command_id: Command::Add.into(),
            hop_count: 0,
            source_addr,
            dest_addr: 0,
            request_id: 1,
            request_part: 0,
            request_count: 1,
            payload,
        }
    }
}
