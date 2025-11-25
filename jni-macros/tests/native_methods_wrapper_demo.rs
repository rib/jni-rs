// Test to demonstrate that API wrappers are generated for all native methods,
// including those with raw fn pointers

use jni::EnvUnowned;
use jni::bind_java_type;
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jint};

// Raw function implementations
extern "system" fn raw_is_positive<'local>(
    _env: EnvUnowned<'local>,
    _this: DemoClass<'local>,
    value: jint,
) -> jboolean {
    if value > 0 { JNI_TRUE } else { JNI_FALSE }
}

extern "system" fn raw_is_even<'local>(
    _env: EnvUnowned<'local>,
    _this: DemoClass<'local>,
    value: jint,
) -> jboolean {
    if value % 2 == 0 { JNI_TRUE } else { JNI_FALSE }
}

bind_java_type! {
    rust_type = DemoClass,
    java_type = "com.example.DemoClass",
    export_native_methods = true,
    native_methods = {
        // Raw function - API wrapper is generated
        fn is_positive {
            sig = (value: jint) -> jboolean,
            fn = raw_is_positive,
            raw = true,
        },

        // Raw function - API wrapper is generated
        fn is_even {
            sig = (value: jint) -> jboolean,
            fn = raw_is_even,
            raw = true,
        },

        // Trait-based - API wrapper is generated
        fn double_value(value: jint) -> jint
    }
}

// Implement the trait only for the trait-based method
impl DemoClassNativeInterface for DemoClassAPI {
    type Error = jni::errors::Error;

    fn double_value<'local>(
        _env: &mut jni::Env<'local>,
        _this: DemoClass<'local>,
        value: jint,
    ) -> Result<jint, Self::Error> {
        Ok(value * 2)
    }
}

fn main() {
    println!("Test demonstrating API wrappers for all native methods");
    println!("- Raw fn methods: is_positive, is_even");
    println!("- Trait-based method: double_value");
    println!("All methods get API wrappers and can be exported!");
}
