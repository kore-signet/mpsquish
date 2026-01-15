//! transform a msgpack stream into a smaller one by interning its keys. then take that compacted stream and turn it to json as fast as possible
//! useful if you have a very specific use case (caching msgpack responses and returning them as json). useless otherwise?
//! note: currently panics extremely easily, TODO on improving that. do not throw anything with non-string map keys at it it will be very sad

use crate::{pack::Packer, unpack::RmpToJson};

pub(crate) mod msgpack;
pub(crate) mod pack;
pub(crate) mod unpack;

/// "Compacts" a msgpack stream by interning any string keys.
pub fn pack_msgpack_stream(
    stream: &[u8],
    interner: &mut lasso::Rodeo,
    out: &mut impl std::io::Write,
) {
    let mut packer = Packer {
        reader: stream,
        out,
        interner,
    };

    while !packer.reader.is_empty() {
        packer.pack_one();
    }
}

/// Unpacks a compacted (i.e, msgpack w interned keys) stream directly into a JSON output string.
pub fn compacted_stream_to_json(
    mut stream: &[u8],
    interner: &impl lasso::Resolver,
    out: &mut String,
) {
    nyoom_json::WriteToJson::write_to_json(
        RmpToJson {
            interner: &interner,
            reader: &mut stream,
        },
        out,
    );
}
