# JNI Signature Procedural Macro

A procedural macro for generating JNI method signature string literals at compile time.

## Quick Start

```rust
use jni_macros::jni_signature;

const SIGNATURE: &'static str = jni_signature!(
    (a: jint, b: java.lang.String as JString, c: [java.lang.Object]) -> java.lang.Throwable
);
// Result: "(ILjava/lang/String;[Ljava/lang/Object;)Ljava/lang/Throwable;"
```

## Syntax

The `jni_signature!` macro differentiates method or field signatures:

| Syntax Pattern | Type | Example |
|----------------|------|---------|
| `(args) -> ret` | Method | `(a: jint) -> void` |
| `Type` | Field | `jint` or `java.lang.String` |

### Basic Form

```rust
jni_signature!((arg1: com.example.Type1, arg2: com.example.Type2, ...) -> com.example.ReturnType)
```

### With Type Mappings

```rust
jni_signature!(
    (arg1: RustType1, arg2: RustType2) -> RustReturnType,
    {
        com.example.Type1 as RustType1,
        com.example.Type2 as RustType2,
        com.example.ReturnType as RustReturnType,
    }
)
```

## Supported Types

### Primitives
- `jint`, `int`, `i32` -> `I`
- `jlong`, `long`, `i64` -> `J`
- `jboolean`, `boolean`, `bool` -> `Z`
- `jbyte`, `byte`, `i8` -> `B`
- `jchar`, `char` -> `C`
- `jshort`, `short`, `i16` -> `S`
- `jfloat`, `float`, `f32` -> `F`
- `jdouble`, `double`, `f64` -> `D`
- `void`, `()` -> `V`

### Java Objects
- Fully qualified: `java.lang.String` -> `Ljava/lang/String;`
- Inner classes: `java.lang.Outer::Inner` -> `Ljava/lang/Outer$Inner;`
- Default package: `.ClassName` -> `LClassName;`

### Arrays
- Prefix: `[jint]` -> `[I`
- Suffix: `jint[]` -> `[I`
- Multi-dimensional: `[[jint]]` or `jint[][]` -> `[[I`

### Rust Reference Types
Requires type mapping:
```rust
jni_signature!(
    (s: JString) -> void,
    {
        java.lang.String as JString,
    }
)
```

Or inline:
```rust
jni_signature!((s: java.lang.String as JString) -> void)
```

## Examples

See `examples/signature_examples.rs` for comprehensive examples.

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](../LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](../LICENSE-MIT) or
   https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.