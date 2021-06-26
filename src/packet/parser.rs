use std::convert::TryFrom;

use nom::bytes::streaming::take;
use nom::combinator::{map, map_res};
use nom::IResult;
use nom::number::streaming::{be_u16, be_u32};
use nom::sequence::tuple;

use super::raw::RawPacket;
use super::types::{Operation, Protocol};

fn parse_proto(input: &[u8]) -> IResult<&[u8], Protocol> {
    map_res(be_u16, Protocol::try_from)(input)
}

fn parse_op(input: &[u8]) -> IResult<&[u8], Operation> {
    map(be_u32, Operation::from)(input)
}

pub fn parse(input: &[u8]) -> IResult<&[u8], RawPacket> {
    let (input, (packet_length, header_length, protocol_version, op, seq_id)) =
        tuple((be_u32, be_u16, parse_proto, parse_op, be_u32))(input)?;
    let (input, data) = take(packet_length - u32::from(header_length))(input)?;
    Ok((
        input,
        RawPacket {
            packet_length,
            header_length,
            protocol_version,
            op,
            seq_id,
            data: data.to_vec(),
        },
    ))
}
