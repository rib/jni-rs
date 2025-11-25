//! This aims to test mixing shorthand and block syntax for methods and fields.

use jni_macros::bind_java_type;

bind_java_type! {
    rust_type = Test,
    java_type = "com.example.Test",
    constructors {
        // ============ SHORTHAND SYNTAX ============
        fn new(),

        fn with_id(id: jint),
        fn with_name(name: java.lang.String),
        fn with_id_and_name(id: jint, name: java.lang.String),

        // ============ BLOCK SYNTAX ============
        fn create {
            sig = () -> void,
        },

        fn create_with_id {
            sig = (id: jint) -> void,
        },

        fn create_with_name {
            sig = (name: java.lang.String) -> void,
        },
    },

    methods = {
        // ============ SHORTHAND SYNTAX ============
        fn get_id() -> jint,
        fn set_id(id: jint) -> void,
        fn get_name() -> java.lang.String,
        fn get_names() -> java.lang.String[],
        fn set_name(name: java.lang.String) -> void,
        fn is_valid() -> jboolean,
        fn calculate_total(a: jint, b: jint) -> jint,

        // ============ BLOCK SYNTAX (NO NAME OVERRIDE) ============
        fn get_status {
            sig = () -> java.lang.String,
        },
        fn update_data {
            sig = (data: java.lang.String) -> void,
        },

        // ============ BLOCK SYNTAX (WITH NAME OVERRIDE) ============
        fn get_user_info {
            name = "getUserDetails",
            sig = () -> java.lang.String,
        },
        fn set_user_info {
            name = "updateUserDetails",
            sig = (info: java.lang.String) -> void,
        },

        // ============ SHORTHAND SYNTAX ============
        static fn get_version() -> java.lang.String,
        static fn create_instance(name: java.lang.String) -> com.example.Test,
        static fn validate_input(data: java.lang.String) -> jboolean,
        static fn parse_int(s: java.lang.String) -> jint,

        // ============ BLOCK SYNTAX (NO NAME OVERRIDE) ============
        static fn get_default_instance {
            sig = () -> com.example.Test,
        },
        static fn validate_config {
            sig = (config: java.lang.String) -> jboolean,
        },

        // ============ BLOCK SYNTAX (WITH NAME OVERRIDE) ============
        static fn get_singleton {
            name = "getInstance",
            sig = () -> com.example.Test,
        },
        static fn create_from_json {
            name = "fromJSON",
            sig = (json: java.lang.String) -> com.example.Test,
        },
    },

    native_methods = {
        // ============ SHORTHAND SYNTAX ============
        fn native_add(a: jint, b: jint) -> jint,

        // ============ BLOCK SYNTAX ============
        fn process_native_string {
            sig = (input: java.lang.String) -> java.lang.String,
        },

        fn custom_native {
            name = "myCustomNativeMethod",
            sig = (x: jint) -> jint,
        },

        // Export configuration (only valid for native methods)
        fn exported_native {
            sig = (value: jint) -> jint,
            export = false,
        },
    },

    fields {
        // ============ SHORTHAND SYNTAX ============
        value: jint,
        field_name: java.lang.String,
        field_names: java.lang.String[],

        // ============ BLOCK SYNTAX (IMPLICIT GET/SET) ============
        value2 {
            sig = jint,
        },
        name2 {
            sig = java.lang.String,
        },
        names2 {
            sig = java.lang.String[],
        },
        // ============ BLOCK SYNTAX (WITH GET/SET) ============
        value3 {
            sig = jint,
            get = get_value3,
            set = set_value3,
        },
        name3 {
            sig = java.lang.String,
            get = get_name3,
            set = set_name3,
        },
        names3 {
            sig = java.lang.String[],
            get = get_names3,
            set = set_names3,
        },
    },
}

// Implement the native methods trait
impl TestNativeInterface for TestAPI {
    type Error = jni::errors::Error;

    fn native_add<'local>(
        _env: &mut jni::Env<'local>,
        _this: Test<'local>,
        a: jni::sys::jint,
        b: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(a + b)
    }

    fn process_native_string<'local>(
        _env: &mut jni::Env<'local>,
        _this: Test<'local>,
        _input: jni::objects::JString<'local>,
    ) -> Result<jni::objects::JString<'local>, Self::Error> {
        Ok(jni::objects::JString::null())
    }

    fn custom_native<'local>(
        _env: &mut jni::Env<'local>,
        _this: Test<'local>,
        x: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(x * 2)
    }

    fn exported_native<'local>(
        _env: &mut jni::Env<'local>,
        _this: Test<'local>,
        value: jni::sys::jint,
    ) -> Result<jni::sys::jint, Self::Error> {
        Ok(value + 1)
    }
}

fn main() {
    println!("Compiled successfully!");
}
