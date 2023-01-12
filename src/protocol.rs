use tracing::warn;

pub const PAYLOAD_SIZE: usize = 32;

#[derive(Debug, Clone, Copy)]
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
    Alive,
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
            Command::Alive => 0x0E,
        }
    }
}

impl From<u8> for Command {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Command::HeloP,
            0x01 => Command::OlehP,
            0x02 => Command::Ident,
            0x03 => Command::Tnedi,
            0x04 => Command::TrustB,
            0x05 => Command::TrustT,
            0x06 => Command::HeloL,
            0x07 => Command::OlehL,
            0x08 => Command::Push,
            0x09 => Command::PushAck,
            0x0A => Command::ADeny,
            0x0B => Command::UDeny,
            0x0C => Command::Add,
            0x0D => Command::Del,
            0x0E => Command::Alive,
            _ => {
                warn!("Unknown command: {}", value);
                panic!("Unknown command: {value}");
            }
        }
    }
}

impl PartialEq<Command> for u8 {
    fn eq(&self, other: &Command) -> bool {
        *self == (u8::from(*other))
    }
}

#[derive(Debug)]
pub struct PFPRequest {
    pub command_id: u8,
    pub hop_count: u8,
    pub source_addr: u32,
    pub dest_addr: u32,
    pub forwarded_by_addr: u32,
    pub request_id: u8,
    pub request_part: u8,
    pub request_count: u8,
    pub payload: [u8; PAYLOAD_SIZE],
}

impl From<PFPRequest> for Vec<u8> {
    fn from(value: PFPRequest) -> Self {
        let mut result = Vec::new();
        result.push(value.command_id);
        result.push(value.hop_count);
        result.extend_from_slice(&value.source_addr.to_be_bytes());
        result.extend_from_slice(&value.dest_addr.to_be_bytes());
        result.extend_from_slice(&value.forwarded_by_addr.to_be_bytes());
        result.push(value.request_id);
        result.push(value.request_part);
        result.push(value.request_count);
        result.extend_from_slice(&value.payload);
        result
    }
}

impl PFPRequest {
    pub fn new_helop(source_addr: u32) -> Self {
        Self {
            command_id: Command::HeloP.into(),
            hop_count: 0,
            source_addr,
            dest_addr: 0,
            forwarded_by_addr: 0,
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
            forwarded_by_addr: 0,
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
            forwarded_by_addr: 0,
            request_id: 1,
            request_part: 0,
            request_count: 1,
            payload,
        }
    }
}
