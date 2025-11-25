// Test the shorthand syntax for fields in the bind_java_type! macro

use jni_macros::bind_java_type;

bind_java_type! {
    rust_type = TestFieldShorthand,
    java_type = "com.example.TestFieldShorthand",
    // Test instance fields with shorthand syntax
    fields {
        // Primitive fields using shorthand syntax
        int_field: jint,
        boolean_field: jboolean,
        long_field: jlong,
        double_field: jdouble,
        byte_field: jbyte,
        char_field: jchar,
        short_field: jshort,
        float_field: jfloat,

        // Object fields using shorthand syntax
        string_field: java.lang.String,
        class_field: java.lang.Class,
        object_field: java.lang.Object,

        // Array fields using shorthand syntax
        int_array_field: jint[],
        string_array_field: java.lang.String[],
        int_2d_array_field: jint[][],
        object_2d_array_field: java.lang.Object[][],

        // Mix shorthand and block syntax
        simple_field: jint,
        complex_field {
            name = "complexJavaName",
            sig = jint[],
        },
        another_simple_field: java.lang.String,

        // Test that snake_case converts to lowerCamelCase for Java name
        my_custom_field: jlong,
        another_custom_field: java.lang.String,

        // Block syntax with custom settings
        special_field {
            name = "specialValue",
            sig = jdouble,
            /// Special getter doc comment
            get = get_special,
            /// Special setter doc comment
            set = set_special,
        },

        // Test static fields with shorthand syntax

        // Static primitive fields using shorthand syntax
        static static_int: jint,
        static static_boolean: jboolean,
        static static_long: jlong,

        // Static object fields using shorthand syntax
        static static_string: java.lang.String,
        static static_class: java.lang.Class,

        // Static array fields using shorthand syntax
        static static_int_array: jint[],
        static static_string_array: java.lang.String[],

        // Mix shorthand and block syntax for static fields
        static simple_static: jint,
        static complex_static {
            name = "complexStaticValue",
            sig = java.lang.Object[],
        },
        static another_simple_static: jboolean,
    }
}

fn main() {
    // Verify that the types exist
    let _null_ref = TestFieldShorthand::null();

    println!("Compiled successfully!");
}
