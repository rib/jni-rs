//! Tests overriding the default `jni` crate name.

// Make sure we don't use the jni crate wrapper that injects `with jni22(jni)`
use jni_macros::bind_java_type;

// Minimal test with no properties or doc comment
bind_java_type! {
    jni = ::jni,
    rust_type = Minimal,
    java_type = "com.example.Minimal"
}

// Test jni path with no leading ::
bind_java_type! {
    jni = ::jni,
    rust_type = PrefixMinimal,
    java_type = "com.example.Minimal"
}

// Test with properties
bind_java_type! {
    jni = jni,
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
    jni = jni,
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
