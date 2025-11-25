//! Tests signatures can be parsed with or without parameter names.
//!
//! (unnamed parameters will generate arg0, arg1, etc.)

use jni_macros::bind_java_type;

bind_java_type! {
    rust_type = JOptionalParamTest,
    java_type = "org.example.OptionalParamTest",
    constructors = {
        fn new_with_named(name: java.lang.String, value: jint),
        fn new_with_unnamed(java.lang.String, jint),
        fn new_mixed(java.lang.String, value: jint),
    },

    methods {
        fn method_with_named_params {
            sig = (name: java.lang.String, age: jint, active: jboolean) -> void,
        },
        fn method_with_unnamed_params {
            sig = (java.lang.String, jint, jboolean) -> void,
        },
        fn method_with_mixed_params {
            sig = (java.lang.String, age: jint, jboolean) -> void,
        },
        fn method_single_unnamed {
            sig = (jint) -> void,
        },
        fn method_array_params {
            sig = (jint[], java.lang.String[], jdouble[][]) -> void,
        },
    },
}

fn main() {
    println!("Compiled successfully!");
}
