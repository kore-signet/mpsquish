use std::{borrow::Cow, io::Write};

use crate::msgpack::{ValueEvent, read_value};
use lasso::Key;
use rmp::Marker;
use rmp::encode as rmpe;

pub(crate) struct Packer<'a, W: Write> {
    pub(crate) interner: &'a mut lasso::Rodeo,
    pub(crate) reader: &'a [u8],
    pub(crate) out: &'a mut W,
}

impl<'a, W: Write> Packer<'a, W> {
    pub(crate) fn pack_one(&mut self) {
        match read_value(&mut self.reader) {
            ValueEvent::SInt(sint) => {
                rmpe::write_sint(self.out, sint).unwrap();
            }
            ValueEvent::UInt(uint) => {
                rmpe::write_uint(self.out, uint).unwrap();
            }
            ValueEvent::Str(s) => {
                let s: Cow<str> = json_escape::escape_str(s).into();
                rmpe::write_str(self.out, &s).unwrap();
            }
            ValueEvent::Bytes(s) => {
                rmpe::write_bin(self.out, s).unwrap();
            }
            ValueEvent::F32(v) => {
                rmpe::write_f32(self.out, v).unwrap();
            }
            ValueEvent::F64(v) => {
                rmpe::write_f64(self.out, v).unwrap();
            }
            ValueEvent::Bool(v) => {
                rmpe::write_bool(self.out, v).unwrap();
            }
            ValueEvent::Nil => {
                rmpe::write_nil(self.out).unwrap();
            }
            ValueEvent::StartObject(l) => {
                rmpe::write_map_len(self.out, l as u32).unwrap();
                for _ in 0..l {
                    if matches!(
                        Marker::from_u8(self.reader[0]),
                        Marker::Str8 | Marker::Str16 | Marker::Str32 | Marker::FixStr(_)
                    ) {
                        let ValueEvent::Str(k) = read_value(&mut self.reader) else {
                            unreachable!()
                        };

                        let k: Cow<str> = json_escape::escape_str(k).into();
                        let k = self.interner.get_or_intern(k);
                        rmpe::write_uint(self.out, k.into_usize() as u64).unwrap();
                    } else {
                        self.pack_one();
                    }

                    self.pack_one();
                }
            }
            ValueEvent::StartArray(l) => {
                rmpe::write_array_len(self.out, l as u32).unwrap();
                for _ in 0..l {
                    self.pack_one();
                }
            }
        };
    }
}
