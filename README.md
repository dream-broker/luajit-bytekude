# luajit-bytekude
A **WIP** LuaJIT bytecode decoder/encoder (only tested with `LuaJIT v2.0.5` at present). 

_NOTE: `luajit-bytekude` was only developed to suit my own use case, i.e. manipulating some obfuscated scripts(which is pre-compiled lua bytecodes) in game bundle in order to reuse them in [dream-tutor](https://github.com/dream-broker/dream-tutor)._

## Usage
For a simple compiled lua file at `/path/to/lua/bytecode`:
```lua
return "hello!"
```

1. decode:
```rust
use deku::prelude::*;

let bytecode = std::fs::read("/path/to/lua/bytecode").unwrap();
let (_, dump) = Dump::from_bytes((&bytecode, 0)).unwrap();
```

2. do some random stuff such as
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

3. encode:
```rust
use deku::prelude::*;

let bytes = dump.to_bytes().unwrap();
std::fs::write("/path/to/lua/new").unwrap();
```

test:
```lua
print(loadfile("/path/to/lua/new")()) -- output: hello!owoQvQ
```
## Acknowledge
This library is built top on [deku](https://docs.rs/deku/latest/deku).
