# mpsquish
tiny library that squishes a msgpack stream by interning all its map keys, and then unpacks that into json really fast.
if that's useful to you, somehow.

usage:
```rust

// compress a msgpack stream from src into out, using interner
mpsquish::pack_msgpack_stream(src: &msgpack_data[..], interner: &mut lasso::Rodeo, out: &mut impl Write)

// decompress an interned msgpack stream into json, using interner
mpsquish::compacted_stream_to_json(src: &msgpack_packed_data[..], interner: &lasso::RodeoResolver, out: &mut String)
```

# benchmarks (citm_catalog.json)

```
msgpack -> compressed msgpack
    time:   [1.4506 ms 1.4556 ms 1.4618 ms]

compressed msgpack -> json
    time:   [642.43 µs 643.64 µs 645.37 µs]

uncompressed msgpack -> json (using serde_transcode)
    time:   [1.3902 ms 1.3939 ms 1.4002 ms]
```