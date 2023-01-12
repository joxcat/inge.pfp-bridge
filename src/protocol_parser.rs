use crate::protocol::{Command, PFPRequest, PAYLOAD_SIZE};
use nom::{
    bytes,
    combinator::verify,
    number::complete::{be_u32, be_u8},
    sequence::tuple,
    IResult,
};

fn parse_payload(input: &[u8]) -> IResult<&[u8], [u8; PAYLOAD_SIZE]> {
    let (input, payload) = bytes::complete::take(PAYLOAD_SIZE)(input)?;
    Ok((input, payload.try_into().unwrap()))
}

pub fn parse(input: &[u8]) -> IResult<&[u8], PFPRequest> {
    let u32_parser = be_u32;
    let u8_parser = be_u8;

    let (input, result) = tuple((
        verify(u8_parser, |byte: &u8| {
            *byte >= u8::from(Command::HeloP) && *byte <= u8::from(Command::Alive)
        }),
        u8_parser,
        u32_parser,
        u32_parser,
        u32_parser,
        u8_parser,
        u8_parser,
        u8_parser,
        parse_payload,
    ))(input)?;

    Ok((
        input,
        PFPRequest {
            command_id: result.0,
            hop_count: result.1,
            source_addr: result.2,
            dest_addr: result.3,
            forwarded_by_addr: result.4,
            request_id: result.5,
            request_part: result.6,
            request_count: result.7,
            payload: result.8,
        },
    ))
}

pub fn parse_push_payload(input: &[u8]) -> IResult<&[u8], (u32, u32)> {
    let (input, result) = tuple((be_u32, be_u32))(input)?;
    Ok((input, (result.0, result.1)))
}
