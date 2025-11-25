// Test that when all native methods use raw fn, no trait is generated

use jni::EnvUnowned;
use jni::bind_java_type;
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jint};

extern "system" fn raw_is_positive<'local>(
    _env: EnvUnowned<'local>,
    _this: AllRawClass<'local>,
    value: jint,
) -> jboolean {
    if value > 0 { JNI_TRUE } else { JNI_FALSE }
}

extern "system" fn raw_is_even<'local>(
    _env: EnvUnowned<'local>,
    _this: AllRawClass<'local>,
    value: jint,
) -> jboolean {
    if value % 2 == 0 { JNI_TRUE } else { JNI_FALSE }
}

bind_java_type! {
    rust_type = AllRawClass,
    java_type = "com.example.AllRawClass",
    native_methods = {
        // All methods use raw fn
        fn is_positive {
            sig = (value: jint) -> jboolean,
            fn = raw_is_positive,
            raw = true,
        },

        fn is_even {
            sig = (value: jint) -> jboolean,
            fn = raw_is_even,
            raw = true,
        }
    }
}

// No trait implementation needed because all methods use raw fn!
// The AllRawClassNativeInterface trait is not even generated.

fn main() {
    println!("Compiled successfully");
}
