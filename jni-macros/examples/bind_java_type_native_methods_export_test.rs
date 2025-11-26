//! Test native method export functionality
//!
//! This example verifies that the bind_java_type! macro correctly handles the export_native_methods
//! property and per-method export overrides:
//! - export_native_methods = true (default): exports all native methods by default
//! - export_native_methods = false: doesn't export native methods by default
//! - Per-method export = true: export with auto-mangled name
//! - Per-method export = false: don't export
//! - Per-method export = "customName": export with custom name
//!
//! Run with: cargo expand --example bind_java_type_native_export_test

use jni_macros::bind_java_type;

// Test 1: Default behavior (export_native_methods = true by default)
// All native methods should be exported with auto-mangled names
bind_java_type! {
    rust_type = TestExportDefault,
    java_type = "com.example.TestExportDefault",
    native_methods = {
        // Should be exported with auto-mangled name (default behavior)
        fn method_one {
            sig = (value: jint) -> jint,
        },

        // Should NOT be exported (explicitly disabled)
        fn method_two {
            sig = (value: jint) -> jint,
            export = false,
        },

        // Should be exported with custom name
        fn method_three {
            sig = (value: jint) -> jint,
            export = "Java_customMethodThreeDefault",
        },
    }
}

impl TestExportDefaultNativeInterface for TestExportDefaultAPI {
    type Error = jni::errors::Error;

    fn method_one<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestExportDefault<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(value * 2)
    }

    fn method_two<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestExportDefault<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(value * 3)
    }

    fn method_three<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestExportDefault<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(value * 4)
    }
}

// Test 2: Explicit export_native_methods = true
// Should behave the same as Test 1
bind_java_type! {
    rust_type = TestExportExplicitTrue,
    java_type = "com.example.TestExportExplicitTrue",
    export_native_methods = true,
    native_methods = {
        fn method_one {
            sig = (value: jint) -> jint,
        },

        fn method_two {
            sig = (value: jint) -> jint,
            export = false,
        },
    }
}

impl TestExportExplicitTrueNativeInterface for TestExportExplicitTrueAPI {
    type Error = jni::errors::Error;

    fn method_one<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestExportExplicitTrue<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(value * 2)
    }

    fn method_two<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestExportExplicitTrue<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(value * 3)
    }
}

// Test 3: export_native_methods = false
// No methods should be exported by default, only those explicitly marked
bind_java_type! {
    rust_type = TestExportDisabled,
    java_type = "com.example.TestExportDisabled",
    export_native_methods = false,
    native_methods = {
        // With the short-form method signature syntax, use 'extern' to explicitly export
        extern fn method_zero(value: jint) -> JString,

        // Should NOT be exported (global default is false, no override)
        fn method_one {
            sig = (value: jint) -> jint,
        },

        // Should be exported (explicitly enabled)
        fn method_two {
            sig = (value: jint) -> jint,
            export = true,
        },

        // Should be exported with custom name
        fn method_three {
            sig = (value: jint) -> jint,
            export = "Java_customMethodThreeDisabled",
        },
    }
}

impl TestExportDisabledNativeInterface for TestExportDisabledAPI {
    type Error = jni::errors::Error;

    fn method_zero<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestExportDisabled<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::objects::JString<'local>, Self::Error> {
        let s = format!("Value: {}", value);
        let jstr = jni::objects::JString::from_str(_env, &s)?;
        Ok(jstr)
    }

    fn method_one<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestExportDisabled<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(value * 2)
    }

    fn method_two<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestExportDisabled<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(value * 3)
    }

    fn method_three<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestExportDisabled<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(value * 4)
    }
}

// Test 4: Static native methods with export control
bind_java_type! {
    rust_type = TestExportStatic,
    java_type = "com.example.TestExportStatic",
    export_native_methods = true,
    native_methods = {
        // Should be exported
        static fn static_method_one {
            sig = () -> jint,
        },

        // Should NOT be exported
        static fn static_method_two {
            sig = () -> jint,
            export = false,
        },
    }
}

impl TestExportStaticNativeInterface for TestExportStaticAPI {
    type Error = jni::errors::Error;

    fn static_method_one<'local>(
        _env: &mut jni::Env<'local>,
        _class: jni::objects::JClass<'local>,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(42)
    }

    fn static_method_two<'local>(
        _env: &mut jni::Env<'local>,
        _class: jni::objects::JClass<'local>,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(100)
    }
}

// Test 5: Mixed instance and static with different export settings
bind_java_type! {
    rust_type = TestExportMixed,
    java_type = "com.example.TestExportMixed",
    export_native_methods = false,
    native_methods = {
        fn instance_exported {
            sig = (value: jint) -> jint,
            export = true,
        },
        fn instance_not_exported {
            sig = (value: jint) -> jint,
        },

        static fn static_exported {
            sig = () -> jint,
            export = true,
        },
        static fn static_not_exported {
            sig = () -> jint,
        },
    }
}

impl TestExportMixedNativeInterface for TestExportMixedAPI {
    type Error = jni::errors::Error;

    fn instance_exported<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestExportMixed<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(value * 2)
    }

    fn instance_not_exported<'local>(
        _env: &mut jni::Env<'local>,
        _this: TestExportMixed<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(value * 3)
    }

    fn static_exported<'local>(
        _env: &mut jni::Env<'local>,
        _class: jni::objects::JClass<'local>,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(42)
    }

    fn static_not_exported<'local>(
        _env: &mut jni::Env<'local>,
        _class: jni::objects::JClass<'local>,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(100)
    }
}

fn main() {
    println!("Compiled successfully!");
}
