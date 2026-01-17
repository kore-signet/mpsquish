use nyoom_json::{ArrayWriter, JsonBuffer, Null, ObjectWriter, UnescapedStr, WriteToJson};

use crate::{
    Resolver, UnpackError,
    msgpack::{ValueEvent, read_value},
};

pub struct RmpToJson<'a, 's, I: Resolver> {
    pub(crate) reader: &'a mut &'s [u8],
    pub(crate) interner: &'a I,
}

impl<'a, 's, I: Resolver> RmpToJson<'a, 's, I> {
    pub fn new(reader: &'a mut &'s [u8], interner: &'a I) -> RmpToJson<'a, 's, I> {
        RmpToJson { reader, interner }
    }

    #[inline(always)]
    pub fn try_write_to_json<S: JsonBuffer>(self, out: &mut S) -> Result<(), UnpackError<I>> {
        match read_value(self.reader)? {
            ValueEvent::SInt(v) => v.write_to_json(out),
            ValueEvent::UInt(v) => v.write_to_json(out),
            ValueEvent::Str(s) => UnescapedStr::create(s).write_to_json(out),
            ValueEvent::Bytes(_items) => {
                return Err(UnpackError::Unsupported("binary arrays are not supported"));
            }
            ValueEvent::F32(v) => v.write_to_json(out),
            ValueEvent::F64(v) => v.write_to_json(out),
            ValueEvent::StartObject(l) => {
                let mut obj = ObjectWriter::start(out);
                for _ in 0..l {
                    let ValueEvent::UInt(k) = read_value(self.reader)? else {
                        return Err(UnpackError::Unsupported("map with non-interned/uint key"));
                    };
                    let k = self
                        .interner
                        .resolve(k as usize)
                        .map_err(UnpackError::Resolver)?
                        .ok_or(UnpackError::KeyNotFound)?;
                    obj.field(
                        UnescapedStr::create(&k),
                        RmpToJson {
                            reader: self.reader,
                            interner: self.interner,
                        },
                    );
                }
                obj.end();
            }
            ValueEvent::StartArray(l) => {
                let mut arr = ArrayWriter::start(out);
                for _ in 0..l {
                    arr.add(RmpToJson {
                        reader: self.reader,
                        interner: self.interner,
                    });
                }

                arr.end();
            }
            ValueEvent::Bool(v) => v.write_to_json(out),
            ValueEvent::Nil => Null.write_to_json(out),
        }

        Ok(())
    }
}

impl<'a, 's, S: JsonBuffer, I: Resolver> WriteToJson<S> for RmpToJson<'a, 's, I> {
    #[inline(always)]
    fn write_to_json(self, out: &mut S) {
        self.try_write_to_json(out).unwrap();
    }
}
