//! Test native methods code generation by the bind_java_type! macro
//!
//! This verifies that the bind_java_type! macro correctly generates:
//! - Native method trait definitions
//! - Native method wrapper functions
//! - Native method export functions (with proper JNI mangling)
//! - Native method registration functions
//!
//! Run with: cargo expand --example bind_java_type_native_methods_test

use jni_macros::bind_java_type;

// Test 1: Basic native methods with different signatures
bind_java_type! {
    rust_type = TestNativeBasic,
    java_type = "com.example.TestNativeBasic",
    native_methods = {
        // Instance method with primitive arguments
        fn native_add {
            name = "nativeAdd",
            sig = (a: jint, b: jint) -> jint,
        },

        // Instance method with object argument
        fn native_process_string {
            name = "nativeProcessString",
            sig = (input: java.lang.String) -> java.lang.String,
        },

        // Instance method with void return
        fn native_log {
            name = "nativeLog",
            sig = (message: java.lang.String) -> void,
        },

        // Instance method with array argument
        fn native_sum_array {
            name = "nativeSumArray",
            sig = (arr: jint[]) -> jint,
        },
    }
}

// Implement the trait
impl TestNativeBasicNativeInterface for TestNativeBasicAPI {
    type Error = jni::errors::Error;

    fn native_add<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestNativeBasic<'local>,
        a: jni::sys::jint,
        b: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(a + b)
    }

    fn native_process_string<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestNativeBasic<'local>,
        _input: jni::objects::JString<'local>,
    ) -> Result<jni::objects::JString<'local>, Self::Error> {
        Ok(jni::objects::JString::null())
    }

    fn native_log<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestNativeBasic<'local>,
        _message: jni::objects::JString<'local>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn native_sum_array<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestNativeBasic<'local>,
        _array: jni::objects::JPrimitiveArray<'local, jni::sys::jint>,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(42)
    }
}

// Test 2: Static native methods
bind_java_type! {
    rust_type = TestNativeStatic,
    java_type = "com.example.TestNativeStatic",
    native_methods = {
        // Static method with no arguments
        static fn native_get_version {
            name = "nativeGetVersion",
            sig = () -> jint,
        },

        // Static method with multiple arguments
        static fn native_initialize {
            name = "nativeInitialize",
            sig = (config: java.lang.String, flags: jint) -> jboolean,
        },
    }
}

impl TestNativeStaticNativeInterface for TestNativeStaticAPI {
    type Error = jni::errors::Error;

    fn native_get_version<'local>(
        _env: &mut jni::Env<'local>,
        _class: jni::objects::JClass<'local>,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(1)
    }

    fn native_initialize<'local>(
        _env: &mut jni::Env<'local>,
        _class: jni::objects::JClass<'local>,
        _config: jni::objects::JString<'local>,
        _flags: jni::sys::jint,
    ) -> Result<jni::sys::jboolean, Self::Error> {
        Ok(true)
    }
}

// Test 3: Custom error policy
bind_java_type! {
    rust_type = TestNativeErrorPolicy,
    java_type = "com.example.TestNativeErrorPolicy",
    native_methods = {
        // Method with custom error policy
        fn native_risky_operation {
            name = "nativeRiskyOperation",
            sig = (value: jint) -> jint,
            error_policy = jni::errors::LogErrorAndDefault,
        },
    }
}

impl TestNativeErrorPolicyNativeInterface for TestNativeErrorPolicyAPI {
    type Error = jni::errors::Error;

    fn native_risky_operation<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestNativeErrorPolicy<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        if value < 0 {
            Err(jni::errors::Error::JniCall(jni::errors::JniError::Unknown))
        } else {
            Ok(value * 2)
        }
    }
}

// Test 4: Exported native methods (for JNI discovery)
bind_java_type! {
    rust_type = TestNativeExport,
    java_type = "com.example.TestNativeExport",
    native_methods = {
        // Exported method - should generate proper JNI mangled name
        extern fn native_exported_method {
            name = "nativeExportedMethod",
            sig = (value: jint, str: java.lang.String) -> void,
        },

        // Exported with custom export name
        fn native_custom_export {
            name = "nativeCustomExport",
            sig = (value: jlong) -> jlong,
            export = "customExportedName",
        },
    }
}

impl TestNativeExportNativeInterface for TestNativeExportAPI {
    type Error = jni::errors::Error;

    fn native_exported_method<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestNativeExport<'local>,
        _value: jni::sys::jint,
        _str: jni::objects::JString<'local>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn native_custom_export<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestNativeExport<'local>,
        value: jni::sys::jlong,
    ) -> Result<jni::sys::jlong, Self::Error> {
        Ok(value + 1)
    }
}

// Test 5: Mixed instance and static native methods
bind_java_type! {
    rust_type = TestNativeMixed,
    java_type = "com.example.TestNativeMixed",
    native_methods = {
        fn native_instance_method {
            name = "nativeInstanceMethod",
            sig = (value: jint) -> jint,
        },
        static fn native_static_method {
            name = "nativeStaticMethod",
            sig = (value: jint) -> jint,
        },
    }
}

impl TestNativeMixedNativeInterface for TestNativeMixedAPI {
    type Error = jni::errors::Error;

    fn native_instance_method<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestNativeMixed<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(value * 2)
    }

    fn native_static_method<'local>(
        _env: &mut jni::Env<'local>,
        _class: jni::objects::JClass<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(value * 3)
    }
}

// Test 6: Custom trait name
bind_java_type! {
    rust_type = TestNativeCustomName,
    java_type = "com.example.TestNativeCustomName",
    native_trait = MyCustomTrait,
    native_methods = {
        fn native_method {
            name = "nativeMethod",
            sig = (value: jint) -> jint,
        },
    }
}

impl MyCustomTrait for TestNativeCustomNameAPI {
    type Error = jni::errors::Error;

    fn native_method<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestNativeCustomName<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(value + 100)
    }
}

// Test 7: Complex signatures with arrays and nested types
bind_java_type! {
    rust_type = TestNativeComplex,
    java_type = "com.example.TestNativeComplex",
    native_methods = {
        // Byte array
        fn native_process_bytes {
            name = "nativeProcessBytes",
            sig = (bytes: jbyte[]) -> jint,
        },

        // Object array
        fn native_process_objects {
            name = "nativeProcessObjects",
            sig = (objects: java.lang.Object[]) -> jint,
        },

        // Multiple array arguments
        fn native_multi_arrays {
            name = "nativeMultiArrays",
            sig = (ints: jint[], doubles: jdouble[]) -> jboolean,
        },
    }
}

impl TestNativeComplexNativeInterface for TestNativeComplexAPI {
    type Error = jni::errors::Error;

    fn native_process_bytes<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestNativeComplex<'local>,
        _bytes: jni::objects::JPrimitiveArray<'local, jni::sys::jbyte>,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(0)
    }

    fn native_process_objects<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestNativeComplex<'local>,
        _objects: jni::objects::JObjectArray<'local>,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(0)
    }

    fn native_multi_arrays<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestNativeComplex<'local>,
        _ints: jni::objects::JPrimitiveArray<'local, jni::sys::jint>,
        _doubles: jni::objects::JPrimitiveArray<'local, jni::sys::jdouble>,
    ) -> Result<jni::sys::jboolean, Self::Error> {
        Ok(true)
    }
}

fn main() {
    println!("Compiled successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_implementations_compile() {
        // This test just verifies that the trait implementations compile correctly
        // We can't actually run them without a JVM
        println!("All trait implementations compiled successfully");
    }
}
