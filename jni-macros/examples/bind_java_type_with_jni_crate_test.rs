//! Tests overriding the default `jni` crate name.

use jni_macros::bind_java_type;

extern crate jni as jni2;

// Minimal test with no properties or doc comment
bind_java_type! {
    jni = ::jni2,
    rust_type = Minimal,
    java_type = "com.example.Minimal"
}

// Test jni path with no leading ::
bind_java_type! {
    jni = ::jni2,
    rust_type = PrefixMinimal,
    java_type = "com.example.Minimal"
}

// Test with properties
bind_java_type! {
    jni = jni2,
    rust_type = WithProperties,
    java_type = "com.example.WithProperties",
    constructors {
        fn new() -> ()
    },
    methods {
        fn test() -> ()
    }
}

// Test with a doc comment for the type
bind_java_type! {
    jni = jni2,
    /// A Rust binding for com.example.OptionalEquals
    rust_type = WithDocs,
    java_type = "com.example.OptionalEquals",
    constructors = {
        fn new() -> ()
    },
    methods = {
        fn test() -> ()
    }
}

fn main() {
    println!("Compiled successfully!");
}
