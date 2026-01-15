use std::{borrow::Cow, io::Write};

use crate::msgpack::{ValueEvent, read_value};
use crate::{Interner, PackError};
use rmp::Marker;
use rmp::encode as rmpe;

pub struct Packer<'a, W: Write, I: Interner> {
    pub(crate) interner: &'a mut I,
    pub(crate) reader: &'a [u8],
    pub(crate) out: &'a mut W,
}

impl<'a, W: Write, I: Interner> Packer<'a, W, I> {
    pub fn new(interner: &'a mut I, reader: &'a [u8], out: &'a mut W) -> Packer<'a, W, I> {
        Packer {
            interner,
            reader,
            out,
        }
    }

    pub fn pack_one(&mut self) -> Result<(), PackError<I>> {
        match read_value(&mut self.reader)? {
            ValueEvent::SInt(sint) => {
                rmpe::write_sint(self.out, sint)?;
            }
            ValueEvent::UInt(uint) => {
                rmpe::write_uint(self.out, uint)?;
            }
            ValueEvent::Str(s) => {
                let s: Cow<str> = json_escape::escape_str(s).into();
                rmpe::write_str(self.out, &s)?;
            }
            ValueEvent::Bytes(s) => {
                rmpe::write_bin(self.out, s)?;
            }
            ValueEvent::F32(v) => {
                rmpe::write_f32(self.out, v)?;
            }
            ValueEvent::F64(v) => {
                rmpe::write_f64(self.out, v)?;
            }
            ValueEvent::Bool(v) => {
                rmpe::write_bool(self.out, v)
                    .map_err(rmp::encode::ValueWriteError::InvalidMarkerWrite)?;
            }
            ValueEvent::Nil => {
                rmpe::write_nil(self.out)
                    .map_err(rmp::encode::ValueWriteError::InvalidMarkerWrite)?;
            }
            ValueEvent::StartObject(l) => {
                rmpe::write_map_len(self.out, l as u32)?;
                for _ in 0..l {
                    if matches!(
                        Marker::from_u8(self.reader[0]),
                        Marker::Str8 | Marker::Str16 | Marker::Str32 | Marker::FixStr(_)
                    ) {
                        let ValueEvent::Str(k) = read_value(&mut self.reader)? else {
                            return Err(PackError::Unsupported(
                                "mpsquish only supports maps with string keys",
                            ));
                        };

                        let k: Cow<str> = json_escape::escape_str(k).into();
                        let k = self.interner.intern(k).map_err(PackError::Interner)?;
                        rmpe::write_uint(self.out, k as u64)?;
                    } else {
                        self.pack_one()?;
                    }

                    self.pack_one()?;
                }
            }
            ValueEvent::StartArray(l) => {
                rmpe::write_array_len(self.out, l as u32)?;
                for _ in 0..l {
                    self.pack_one()?;
                }
            }
        };

        Ok(())
    }
}
