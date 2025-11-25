// Test that raw fn methods can be exported

use jni::EnvUnowned;
use jni::bind_java_type;
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jint};

extern "system" fn raw_is_positive<'local>(
    _env: EnvUnowned<'local>,
    _this: TestExport<'local>,
    value: jint,
) -> jboolean {
    if value > 0 { JNI_TRUE } else { JNI_FALSE }
}

extern "system" fn raw_is_even<'local>(
    _env: EnvUnowned<'local>,
    _this: TestExport<'local>,
    value: jint,
) -> jboolean {
    if value % 2 == 0 { JNI_TRUE } else { JNI_FALSE }
}

bind_java_type! {
    rust_type = TestExport,
    java_type = "com.example.TestExport",
    export_native_methods = true,
    native_methods = {
        // This method uses raw fn and should be exported
        fn is_positive {
            sig = (value: jint) -> jboolean,
            fn = raw_is_positive,
            raw = true,
        },

        // This method uses raw fn but explicitly not exported
        fn is_even {
            sig = (value: jint) -> jboolean,
            fn = raw_is_even,
            raw = true,
            export = false
        }
    }
}

fn main() {
    // Verify that the exported function exists by referencing it
    // The mangled name for is_positive should be: Java_com_example_TestExport_isPositive__I
    let _exported_fn: unsafe extern "system" fn(jni::EnvUnowned, TestExport, jint) -> jboolean =
        Java_com_example_TestExport_isPositive__I;

    println!("Raw fn methods with export work!");
}
