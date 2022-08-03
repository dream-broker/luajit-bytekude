# luajit-bytekude
A **WIP** LuaJIT bytecode decoder/encoder (only tested with `LuaJIT v2.0.5` at present). 

_NOTE: `luajit-bytekude` was only developed to suit my own use case, i.e. manipulating some obfuscated scripts(which is pre-compiled lua bytecodes) in game bundle in order to reuse them in [dream-tutor](https://github.com/dream-broker/dream-tutor)._

## Usage
decode:
```rust
let bytecode = std::fs::read("/path/to/lua/bytecode").unwrap();
let (_, dump) = Dump::from_bytes((&bytecode, 0)).unwrap();
```

then do some random stuff such as
```rust
// append "owoQvQ" for all strings
for Lengthed(pt) in &mut dump.prototypes {
  for kgc in &mut pt.constant_gc {
    if let ConstantGc::Str(LenString(s)) = kgc {
      s.extend_from_slice("owoQvQ".as_bytes());
    } 
  }
}
```

encode:
```rust
let bytes = dump.to_bytes().unwrap();
```
## Acknowledge
This library is built top on [deku](https://docs.rs/deku/latest/deku).
