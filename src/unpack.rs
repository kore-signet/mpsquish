use nyoom_json::{ArrayWriter, JsonBuffer, Null, ObjectWriter, UnescapedStr, WriteToJson};

use crate::{
    Resolver, UnpackError,
    msgpack::{ValueEvent, read_value},
};

pub struct NoOpInterner;

impl Resolver for NoOpInterner {
    type Err = core::convert::Infallible;

    fn resolve(&self, _k: usize) -> Result<Option<smol_str::SmolStr>, Self::Err> {
        panic!("this interner/resolver should never be used")
    }
}

pub struct NonInternedRmpToJson<'a, 's> {
    pub(crate) reader: &'a mut &'s [u8],
}

impl<'a, 's> NonInternedRmpToJson<'a, 's> {
    pub fn new(reader: &'a mut &'s [u8]) -> NonInternedRmpToJson<'a, 's> {
        NonInternedRmpToJson { reader }
    }

    #[inline(always)]
    pub fn try_write_to_json<S: JsonBuffer>(
        self,
        out: &mut S,
    ) -> Result<(), UnpackError<NoOpInterner>> {
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
                    let ValueEvent::Str(k) = read_value(self.reader)? else {
                        return Err(UnpackError::Unsupported("map with non-str key"));
                    };

                    obj.field(
                        UnescapedStr::create(&k),
                        NonInternedRmpToJson {
                            reader: self.reader,
                        },
                    );
                }
                obj.end();
            }
            ValueEvent::StartArray(l) => {
                let mut arr = ArrayWriter::start(out);
                for _ in 0..l {
                    arr.add(NonInternedRmpToJson {
                        reader: self.reader,
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

impl<'a, 's, S: JsonBuffer> WriteToJson<S> for NonInternedRmpToJson<'a, 's> {
    fn write_to_json(self, out: &mut S) {
        self.try_write_to_json(out).unwrap()
    }
}

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
