use lasso::{Key, Spur};
use nyoom_json::{ArrayWriter, JsonBuffer, Null, ObjectWriter, UnescapedStr, WriteToJson};

use crate::msgpack::{ValueEvent, read_value};

pub(crate) struct RmpToJson<'a, 's, I: lasso::Resolver> {
    pub(crate) reader: &'a mut &'s [u8],
    pub(crate) interner: &'a I,
}

impl<'a, 's, S: JsonBuffer, I: lasso::Resolver> WriteToJson<S> for RmpToJson<'a, 's, I> {
    #[inline(always)]
    fn write_to_json(self, out: &mut S) {
        match read_value(self.reader) {
            ValueEvent::SInt(v) => v.write_to_json(out),
            ValueEvent::UInt(v) => v.write_to_json(out),
            ValueEvent::Str(s) => UnescapedStr::create(s).write_to_json(out),
            ValueEvent::Bytes(_items) => todo!(),
            ValueEvent::F32(v) => v.write_to_json(out),
            ValueEvent::F64(v) => v.write_to_json(out),
            ValueEvent::StartObject(l) => {
                let mut obj = ObjectWriter::start(out);
                for _ in 0..l {
                    let ValueEvent::UInt(k) = read_value(self.reader) else {
                        todo!()
                    };
                    let k = self
                        .interner
                        .resolve(&Spur::try_from_usize(k as usize).unwrap());
                    obj.field(
                        UnescapedStr::create(k),
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
    }
}
