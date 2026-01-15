use byteorder::{BE, ReadBytesExt};
use rmp::{Marker, decode as rmpd, decode::RmpRead};

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
pub(crate) fn len_from_marker(marker: &Marker, buf: &mut &[u8]) -> usize {
    match marker {
        Marker::FixMap(l) => *l as usize,
        Marker::FixArray(l) => *l as usize,
        Marker::FixStr(l) => *l as usize,
        Marker::Bin8 => RmpRead::read_u8(buf).unwrap() as usize,
        Marker::Bin16 => buf.read_u16::<BE>().unwrap() as usize,
        Marker::Bin32 => buf.read_u32::<BE>().unwrap() as usize,
        Marker::Str8 => RmpRead::read_u8(buf).unwrap() as usize,
        Marker::Str16 => buf.read_u16::<BE>().unwrap() as usize,
        Marker::Str32 => buf.read_u32::<BE>().unwrap() as usize,
        Marker::Array16 => buf.read_u16::<BE>().unwrap() as usize,
        Marker::Array32 => buf.read_u32::<BE>().unwrap() as usize,
        Marker::Map16 => buf.read_u16::<BE>().unwrap() as usize,
        Marker::Map32 => buf.read_u32::<BE>().unwrap() as usize,
        _ => todo!(),
    }
}

#[inline(always)]
pub(crate) fn read_value<'a>(buf: &mut &'a [u8]) -> ValueEvent<'a> {
    let marker = rmpd::read_marker(buf).unwrap();
    match marker {
        Marker::Null => ValueEvent::Nil,
        Marker::False => ValueEvent::Bool(false),
        Marker::True => ValueEvent::Bool(true),
        Marker::F32 => ValueEvent::F32(buf.read_f32::<BE>().unwrap()),
        Marker::F64 => ValueEvent::F64(buf.read_f64::<BE>().unwrap()),
        Marker::FixPos(v) => ValueEvent::UInt(v as u64),
        Marker::U8 => ValueEvent::UInt(RmpRead::read_u8(buf).unwrap() as u64),
        Marker::U16 => ValueEvent::UInt(buf.read_u16::<BE>().unwrap() as u64),
        Marker::U32 => ValueEvent::UInt(buf.read_u32::<BE>().unwrap() as u64),
        Marker::U64 => ValueEvent::UInt(buf.read_u64::<BE>().unwrap()),
        Marker::FixNeg(v) => ValueEvent::SInt(v as i64),
        Marker::I8 => ValueEvent::SInt(buf.read_i8().unwrap() as i64),
        Marker::I16 => ValueEvent::SInt(buf.read_i16::<BE>().unwrap() as i64),
        Marker::I32 => ValueEvent::SInt(buf.read_i32::<BE>().unwrap() as i64),
        Marker::I64 => ValueEvent::SInt(buf.read_i64::<BE>().unwrap()),
        Marker::FixMap(len) => ValueEvent::StartObject(len as usize),
        Marker::FixArray(len) => ValueEvent::StartArray(len as usize),
        Marker::FixStr(len) => {
            ValueEvent::Str(std::str::from_utf8(buf.split_off(..(len as usize)).unwrap()).unwrap())
        }
        Marker::Bin8 | Marker::Bin16 | Marker::Bin32 => {
            let len = len_from_marker(&marker, buf);
            ValueEvent::Bytes(buf.split_off(..len).unwrap())
        }
        Marker::Str8 | Marker::Str16 | Marker::Str32 => {
            let len = len_from_marker(&marker, buf);
            ValueEvent::Str(std::str::from_utf8(buf.split_off(..len).unwrap()).unwrap())
        }
        Marker::Array16 | Marker::Array32 => {
            let len = len_from_marker(&marker, buf);
            ValueEvent::StartArray(len)
        }
        Marker::Map16 | Marker::Map32 => {
            let len = len_from_marker(&marker, buf);
            ValueEvent::StartObject(len)
        }
        Marker::FixExt1 => todo!(),
        Marker::FixExt2 => todo!(),
        Marker::FixExt4 => todo!(),
        Marker::FixExt8 => todo!(),
        Marker::FixExt16 => todo!(),
        Marker::Ext8 => todo!(),
        Marker::Ext16 => todo!(),
        Marker::Ext32 => todo!(),
        Marker::Reserved => todo!(),
    }
}
