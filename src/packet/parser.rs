use std::convert::{TryFrom, TryInto};

use nom::combinator::map_res;
use nom::sequence::tuple;
use nom::{bytes::streaming::take, IResult};

use super::raw::RawPacket;
use super::types::{Operation, Protocol};

fn to_u32(input: &[u8]) -> Result<u32, std::array::TryFromSliceError> {
    Ok(u32::from_be_bytes(input.try_into()?))
}

fn to_u16(input: &[u8]) -> Result<u16, std::array::TryFromSliceError> {
    Ok(u16::from_be_bytes(input.try_into()?))
}

fn parse_4bit(input: &[u8]) -> IResult<&[u8], u32> {
    map_res(take(4usize), to_u32)(input)
}

fn parse_2bit(input: &[u8]) -> IResult<&[u8], u16> {
    map_res(take(2usize), to_u16)(input)
}

fn parse_proto(input: &[u8]) -> IResult<&[u8], Protocol> {
    map_res(parse_2bit, Protocol::try_from)(input)
}

fn parse_op(input: &[u8]) -> IResult<&[u8], Operation> {
    map_res(parse_4bit, Operation::try_from)(input)
}

pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], RawPacket> {
    let (input, (packet_length, header_length, protocol_version, op, seq_id)) =
        tuple((parse_4bit, parse_2bit, parse_proto, parse_op, parse_4bit))(input)?;
    if let Protocol::Buffer = protocol_version {
        unimplemented!();
    }
    let (input, data) = take(packet_length - header_length as u32)(input)?;
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
