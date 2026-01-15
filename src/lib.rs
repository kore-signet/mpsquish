//! transform a msgpack stream into a smaller one by interning its keys. then take that compacted stream and turn it to json as fast as possible
//! useful if you have a very specific use case (caching msgpack responses and returning them as json). useless otherwise?
//! note: currently panics extremely easily, TODO on improving that. do not throw anything with non-string map keys at it it will be very sad

use lasso::{Key, Spur};

use crate::{pack::Packer, unpack::RmpToJson};

pub(crate) mod msgpack;
pub mod pack;
pub mod unpack;

#[derive(thiserror::Error)]
pub enum PackError<I: Interner> {
    #[error(transparent)]
    ValueRead(#[from] ReadValueError),
    #[error(transparent)]
    ValueWrite(#[from] rmp::encode::ValueWriteError),
    #[error("unsupported operation: {0}")]
    Unsupported(&'static str),
    #[error(transparent)]
    Interner(I::Err),
}

impl<I: Interner> std::fmt::Debug for PackError<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValueRead(arg0) => f.debug_tuple("ValueRead").field(arg0).finish(),
            Self::ValueWrite(arg0) => f.debug_tuple("ValueWrite").field(arg0).finish(),
            Self::Unsupported(arg0) => f.debug_tuple("Unsupported").field(arg0).finish(),
            Self::Interner(arg0) => f.debug_tuple("Interner").field(arg0).finish(),
        }
    }
}

#[derive(thiserror::Error)]
pub enum UnpackError<R: Resolver> {
    #[error(transparent)]
    ValueRead(#[from] ReadValueError),
    #[error("key was not found in resolver")]
    KeyNotFound,
    #[error("unsupported operation: {0}")]
    Unsupported(&'static str),
    #[error(transparent)]
    Resolver(R::Err),
}

impl<R: Resolver> std::fmt::Debug for UnpackError<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValueRead(arg0) => f.debug_tuple("ValueRead").field(arg0).finish(),
            Self::KeyNotFound => write!(f, "KeyNotFound"),
            Self::Unsupported(arg0) => f.debug_tuple("Unsupported").field(arg0).finish(),
            Self::Resolver(arg0) => f.debug_tuple("Resolver").field(arg0).finish(),
        }
    }
}
// impl<I:

#[derive(thiserror::Error, Debug)]
pub enum ReadValueError {
    #[error(transparent)]
    RmpValueRead(#[from] rmp::decode::ValueReadError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    InvalidUTF8(#[from] std::str::Utf8Error),
    #[error("invalid read call: {0}")]
    InvalidCall(&'static str),
    #[error("unsupported msgpack type: {0}")]
    UnsupportedType(&'static str),
}

impl From<rmp::decode::MarkerReadError> for ReadValueError {
    fn from(value: rmp::decode::MarkerReadError) -> Self {
        ReadValueError::RmpValueRead(rmp::decode::ValueReadError::InvalidMarkerRead(value.0))
    }
}

pub trait Interner {
    type Err: std::error::Error;

    fn intern(&mut self, s: impl AsRef<str>) -> Result<usize, Self::Err>;
}

pub trait Resolver {
    type Err: std::error::Error;

    fn resolve(&self, k: usize) -> Result<Option<&str>, Self::Err>;
}

impl<T: lasso::Interner> Interner for T {
    type Err = core::convert::Infallible;

    fn intern(&mut self, s: impl AsRef<str>) -> Result<usize, Self::Err> {
        Ok(self.get_or_intern(s.as_ref()).into_usize())
    }
}

impl<T: lasso::Resolver> Resolver for T {
    type Err = core::convert::Infallible;
    fn resolve(&self, k: usize) -> Result<Option<&str>, Self::Err> {
        Ok(Spur::try_from_usize(k).and_then(|v| self.try_resolve(&v)))
    }
}

/// "Compacts" a msgpack stream by interning any string keys.
pub fn pack_msgpack_stream(
    stream: &[u8],
    interner: &mut impl Interner,
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
pub fn compacted_stream_to_json(mut stream: &[u8], interner: &impl Resolver, out: &mut String) {
    nyoom_json::WriteToJson::write_to_json(
        RmpToJson {
            interner,
            reader: &mut stream,
        },
        out,
    );
}
