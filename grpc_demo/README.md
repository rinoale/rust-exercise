# gRPC in Rust, with the actual bytes on the wire

A working tonic client/server plus `tap`, a man-in-the-middle TCP proxy that prints
every byte flowing in both directions and decodes it layer by layer. Rust port of
the Ruby demo in `ruby_codes/grpc_demo`.

```
./run_demo.sh                   # server + tap + client
TAP_QUIET=1 ./run_demo.sh       # same, minus PING/WINDOW_UPDATE housekeeping
```

| file | what it is |
| --- | --- |
| `proto/greeter.proto` | service definition (one unary RPC, one server-streaming RPC) |
| `build.rs` + `src/lib.rs` | codegen: tonic-prost-build runs protoc, `include_proto!` pulls the result in |
| `src/bin/server.rs` | listens on **50051**, plaintext h2c so the bytes are readable |
| `src/bin/client.rs` | connects to **50052** (the tap), which forwards to 50051 |
| `src/bin/tap.rs` | the byte dumper: HTTP/2 frames → HPACK → gRPC framing → protobuf |
| `src/hpack.rs`, `src/hpack_huffman.rs` | RFC 7541 HPACK decoder + Huffman table for the tap |
| `src/protowire.rs` | schema-free protobuf wire-format decoder |

Run the pieces separately if you prefer:

```
cargo run --bin server                    # terminal 1
cargo run --bin tap                       # terminal 2  (50052 -> localhost:50051)
cargo run --bin client                    # terminal 3
cargo run --bin client -- localhost:50051 # bypass the tap
```

## The four layers, from the outside in

```
TCP
└── HTTP/2 frame        9-byte header: length(3) type(1) flags(1) stream-id(4)
    ├── HEADERS frame   HPACK-compressed metadata (:path, grpc-status, ...)
    └── DATA frame      1 or more gRPC length-prefixed messages
        └── gRPC frame  1-byte compressed-flag + 4-byte big-endian length
            └── protobuf  tag = (field_number << 3) | wire_type, then the value
```

An RPC is **one HTTP/2 stream**: client HEADERS → client DATA → server HEADERS →
server DATA → server trailing HEADERS. Streams are numbered 1, 3, 5, … (odd = client
initiated), so three RPCs over one connection use streams 1, 3 and 5 and can overlap.

## Real output, annotated

**1. Connection preface** — the 24 fixed bytes every HTTP/2 client sends first:

```
C->S HTTP/2 connection preface (24 bytes, sent once by the client)
     0000  50 52 49 20 2a 20 48 54  54 50 2f 32 2e 30 0d 0a  |PRI * HTTP/2.0..|
     0010  0d 0a 53 4d 0d 0a 0d 0a                           |..SM....|
```

**2. Request HEADERS** — this is where the method name lives. gRPC has no "method"
field of its own; the method *is* the HTTP/2 `:path`:

```
C->S HEADERS stream=1 length=93 flags=0x04 (END_HEADERS)
     frame header: 00005d010400000001  = len(3B) type(1B) flags(1B) stream-id(4B)
     0000  83 86 41 8b a0 e4 1d 13  9d 09 b8 d8 00 d8 bf 04  |..A.............|
     HPACK-decoded headers:
       :method:                 "POST"                        indexed(3)
       :scheme:                 "http"                        indexed(6)
       :authority:              "localhost:50052"             literal+index
       :path:                   "/greeter.Greeter/SayHello"   literal
       x-demo-header:           "hi-from-client"              literal+index
       te:                      "trailers"                    literal+index
       content-type:            "application/grpc"            literal+index
       user-agent:              "tonic/0.14.6"                literal+index
```

One difference from the Ruby (grpc-c) client: tonic Huffman-encodes header strings,
so `:path` is 0x93 = 19 Huffman bytes instead of 25 ASCII bytes — the tap's Huffman
decoder (RFC 7541 Appendix B) recovers the text. `83` and `86` are single bytes that
mean "static table entry 3" (`:method: POST`) and "entry 6" (`:scheme: http`) — that
is the whole point of HPACK.

The *second* RPC on the same connection is 27 bytes instead of 93, because everything
is now in the dynamic table and collapses to one byte each:

```
C->S HEADERS stream=3 length=27 flags=0x04 (END_HEADERS)
       :method:      "POST"                         indexed(3)
       :scheme:      "http"                         indexed(6)
       :authority:   "localhost:50052"              indexed(66)
       :path:        "/greeter.Greeter/CountDown"   literal
       te:           "trailers"                     indexed(64)
       content-type: "application/grpc"             indexed(63)
       user-agent:   "tonic/0.14.6"                 indexed(62)
```

**3. Request DATA** — the payload. `HelloRequest { name: "kevin", times: 3 }`:

```
C->S DATA stream=1 length=14 flags=0x00
     0000  00 00 00 00 09 0a 05 6b  65 76 69 6e 10 03        |.......kevin..|
     gRPC message frame: compressed-flag=0 length=9 (prefix bytes 00 00000009)
     protobuf payload: 0a056b6576696e1003
       field 1  LEN     (5 bytes) = "kevin"
       field 2  VARINT  = 3
```

Byte by byte:

| bytes | meaning |
| --- | --- |
| `00` | gRPC compressed-flag: 0 = not compressed (1 would mean `grpc-encoding` applies) |
| `00 00 00 09` | gRPC message length, big-endian uint32 = 9 bytes follow |
| `0a` | protobuf tag `0000_1010` = field 1, wire type 2 (length-delimited) |
| `05` | length 5 |
| `6b 65 76 69 6e` | `"kevin"` |
| `10` | tag `0001_0000` = field 2, wire type 0 (varint) |
| `03` | value 3 |

(tonic sends END_STREAM as a separate empty DATA frame right after; grpc-c sets the
flag on the DATA frame itself. Same meaning: "no more request messages" — that flag
is the entire difference between a unary and a client-streaming call.)

Note there is no field for `times` being an `int32` versus `int64`, and no field names
anywhere. The wire format carries only field *numbers* and *wire types*; the `.proto`
supplies the rest. That is why renaming a field is safe and renumbering one is not.

**4. Response** — headers, then data, then *trailers* (a second HEADERS frame, after
the body, carrying the status). This is why gRPC requires HTTP/2 `te: trailers`:

```
S->C HEADERS stream=1 length=59 flags=0x04 (END_HEADERS)
       :status: "200"   x-served-by: "rust-greeter"   content-type: "application/grpc"

S->C DATA stream=1 length=22 flags=0x00
     0000  00 00 00 00 11 0a 0d 48  65 6c 6c 6f 2c 20 6b 65  |.......Hello, ke|
     0010  76 69 6e 21 10 01                                 |vin!..|
       field 1  LEN     (13 bytes) = "Hello, kevin!"
       field 2  VARINT  = 1

S->C HEADERS stream=1 length=12 flags=0x05 (END_STREAM|END_HEADERS)
       grpc-status:  "0"
```

`:status: 200` is the *HTTP* status and is 200 even when the RPC fails. The real
result is `grpc-status` in the trailers. The failing call at the end of `client.rs`
shows `grpc-status: "12"` = UNIMPLEMENTED.

**5. Server streaming** — same stream, one DATA frame per message, then trailers:

```
S->C DATA stream=3 length=17   ->  field 1 = "kevin: 3", field 2 = 3
S->C DATA stream=3 length=17   ->  field 1 = "kevin: 2", field 2 = 2
S->C DATA stream=3 length=17   ->  field 1 = "kevin: 1", field 2 = 1
S->C DATA stream=3 length=21   ->  field 1 = "kevin: liftoff", field 2 = 0
S->C HEADERS stream=3 flags=0x05 (END_STREAM|END_HEADERS)  ->  grpc-status: "0"
```

Nothing about a stream is special on the wire: it is just more DATA frames before the
trailers arrive. Client streaming is the mirror image, bidirectional streaming is both.

## Things worth noticing in the dump

- **PING / WINDOW_UPDATE frames** are HTTP/2 keepalive and flow control, not gRPC.
  Every DATA frame is answered with a `WINDOW_UPDATE` granting back the credit it
  consumed; that is the backpressure mechanism. `TAP_QUIET=1` hides them.
- **A message can straddle DATA frames.** The 5-byte gRPC prefix is what tells the
  receiver where a message ends, independent of frame boundaries — the tap keeps a
  per-stream buffer for exactly this reason (`data_frame` in `tap.rs`).
- **`length=0` DATA with END_STREAM** carries no message, just the stream-closing flag.
- tonic is pure Rust on top of the `h2` crate (unlike grpc-ruby, which wraps the C
  core), but the bytes are the same protocol — `tcpdump -i lo0 -X port 50051` will
  confirm it.

## Doing this without the tap

Wireshark decodes gRPC natively (`Decode As` → HTTP2, load the `.proto` for field
names), and tonic's `h2` layer will log its own frame view:

```
RUST_LOG=h2=trace cargo run --bin client -- localhost:50051   # (needs tracing wired up)
```

The tap exists because it shows the bytes and the decode side by side, in a few
hundred lines of plain Rust you can read.
