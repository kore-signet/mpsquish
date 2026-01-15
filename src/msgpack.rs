use byteorder::{BE, ReadBytesExt};
use rmp::{Marker, decode as rmpd, decode::RmpRead};

use crate::ReadValueError;

#[derive(Debug)]
pub(crate) enum ValueEvent<'a> {
    SInt(i64),
    UInt(u64),
    Str(&'a str),
    Bytes(&'a [u8]),
    F32(f32),
    F64(f64),
    StartObject(usize),
    StartArray(usize),
    Bool(bool),
    Nil,
}

// enum BasicV

#[inline(always)]
pub(crate) fn len_from_marker(marker: &Marker, buf: &mut &[u8]) -> Result<usize, ReadValueError> {
    Ok(match marker {
        Marker::FixMap(l) => *l as usize,
        Marker::FixArray(l) => *l as usize,
        Marker::FixStr(l) => *l as usize,
        Marker::Bin8 => RmpRead::read_u8(buf)? as usize,
        Marker::Bin16 => buf.read_u16::<BE>()? as usize,
        Marker::Bin32 => buf.read_u32::<BE>()? as usize,
        Marker::Str8 => RmpRead::read_u8(buf)? as usize,
        Marker::Str16 => buf.read_u16::<BE>()? as usize,
        Marker::Str32 => buf.read_u32::<BE>()? as usize,
        Marker::Array16 => buf.read_u16::<BE>()? as usize,
        Marker::Array32 => buf.read_u32::<BE>()? as usize,
        Marker::Map16 => buf.read_u16::<BE>()? as usize,
        Marker::Map32 => buf.read_u32::<BE>()? as usize,
        _ => {
            return Err(ReadValueError::InvalidCall(
                "called len_from_marker on a marker without length information!",
            ));
        }
    })
}

#[inline(always)]
pub(crate) fn read_value<'a>(buf: &mut &'a [u8]) -> Result<ValueEvent<'a>, ReadValueError> {
    let marker = rmpd::read_marker(buf)?;
    Ok(match marker {
        Marker::Null => ValueEvent::Nil,
        Marker::False => ValueEvent::Bool(false),
        Marker::True => ValueEvent::Bool(true),
        Marker::F32 => ValueEvent::F32(buf.read_f32::<BE>()?),
        Marker::F64 => ValueEvent::F64(buf.read_f64::<BE>()?),
        Marker::FixPos(v) => ValueEvent::UInt(v as u64),
        Marker::U8 => ValueEvent::UInt(RmpRead::read_u8(buf)? as u64),
        Marker::U16 => ValueEvent::UInt(buf.read_u16::<BE>()? as u64),
        Marker::U32 => ValueEvent::UInt(buf.read_u32::<BE>()? as u64),
        Marker::U64 => ValueEvent::UInt(buf.read_u64::<BE>()?),
        Marker::FixNeg(v) => ValueEvent::SInt(v as i64),
        Marker::I8 => ValueEvent::SInt(buf.read_i8()? as i64),
        Marker::I16 => ValueEvent::SInt(buf.read_i16::<BE>()? as i64),
        Marker::I32 => ValueEvent::SInt(buf.read_i32::<BE>()? as i64),
        Marker::I64 => ValueEvent::SInt(buf.read_i64::<BE>()?),
        Marker::FixMap(len) => ValueEvent::StartObject(len as usize),
        Marker::FixArray(len) => ValueEvent::StartArray(len as usize),
        Marker::FixStr(len) => ValueEvent::Str(std::str::from_utf8(
            buf.split_off(..(len as usize))
                .ok_or(std::io::Error::from(std::io::ErrorKind::UnexpectedEof))?,
        )?),
        Marker::Bin8 | Marker::Bin16 | Marker::Bin32 => {
            let len = len_from_marker(&marker, buf)?;
            ValueEvent::Bytes(
                buf.split_off(..len)
                    .ok_or(std::io::Error::from(std::io::ErrorKind::UnexpectedEof))?,
            )
        }
        Marker::Str8 | Marker::Str16 | Marker::Str32 => {
            let len = len_from_marker(&marker, buf)?;
            ValueEvent::Str(std::str::from_utf8(
                buf.split_off(..len)
                    .ok_or(std::io::Error::from(std::io::ErrorKind::UnexpectedEof))?,
            )?)
        }
        Marker::Array16 | Marker::Array32 => {
            let len = len_from_marker(&marker, buf)?;
            ValueEvent::StartArray(len)
        }
        Marker::Map16 | Marker::Map32 => {
            let len = len_from_marker(&marker, buf)?;
            ValueEvent::StartObject(len)
        }
        Marker::FixExt1
        | Marker::FixExt2
        | Marker::FixExt4
        | Marker::FixExt8
        | Marker::FixExt16 => return Err(ReadValueError::UnsupportedType("FixExt")),
        Marker::Ext8 | Marker::Ext16 | Marker::Ext32 => {
            return Err(ReadValueError::UnsupportedType("Ext"));
        }
        Marker::Reserved => return Err(ReadValueError::UnsupportedType("Reserved")),
    })
}
