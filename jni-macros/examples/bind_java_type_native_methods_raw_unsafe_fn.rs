// Test that raw function pointers work correctly with native methods

use jni::EnvUnowned;
use jni::bind_java_type;
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jint};

// A raw JNI function implementation
extern "system" fn raw_check_positive<'local>(
    _env: EnvUnowned<'local>,
    _this: TestClass<'local>,
    value: jint,
) -> jboolean {
    if value > 0 { JNI_TRUE } else { JNI_FALSE }
}

// Another raw JNI function for static methods
extern "system" fn raw_static_is_even<'local>(
    _env: EnvUnowned<'local>,
    _class: jni::objects::JClass<'local>,
    value: jint,
) -> jboolean {
    if value % 2 == 0 { JNI_TRUE } else { JNI_FALSE }
}

bind_java_type! {
    rust_type = TestClass,
    java_type = "com.example.TestClass",
    native_methods = {
        // Regular trait-based method
        fn regular_method() -> jint,

        // Raw function pointer
        raw fn check_positive {
            sig = (value: jint) -> jboolean,
            fn = raw_check_positive,
        },

        // Raw function pointer for static method
        static raw fn is_even {
            sig = (value: jint) -> jboolean,
            fn = raw_static_is_even,
        }
    },
}

// Implement the native trait only for the regular method
impl TestClassNativeInterface for TestClassAPI {
    type Error = jni::errors::Error;

    fn regular_method<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestClass<'local>,
    ) -> Result<jint, Self::Error> {
        Ok(42)
    }
}

fn main() {}
