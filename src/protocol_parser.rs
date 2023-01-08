use crate::protocol::{PFPRequest, PAYLOAD_SIZE};
use nom::{
    bytes,
    number::complete::{le_u32, le_u8},
    sequence::tuple,
    IResult,
};

fn parse_payload(input: &[u8]) -> IResult<&[u8], [u8; PAYLOAD_SIZE]> {
    let (input, payload) = bytes::complete::take(PAYLOAD_SIZE)(input)?;
    Ok((input, payload.try_into().unwrap()))
}

pub fn parse(input: &[u8]) -> IResult<&[u8], PFPRequest> {
    let (input, result) = tuple((
        le_u8,
        le_u8,
        le_u32,
        le_u32,
        le_u8,
        le_u8,
        le_u8,
        parse_payload,
    ))(input)?;

    Ok((
        input,
        PFPRequest {
            command_id: result.0,
            hop_count: result.1,
            source_addr: result.2,
            dest_addr: result.3,
            request_id: result.4,
            request_part: result.5,
            request_count: result.6,
            payload: result.7,
        },
    ))
}
