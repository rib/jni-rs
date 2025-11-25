use jni_macros::bind_java_type;

bind_java_type! {
    rust_type = JTest,
    java_type = "com.example.ConstructorTest",
    constructors {
        /// Creates a new JTest instance with default values.
        fn new(),

        /// Creates a new JTest instance with the given integer value.
        ///
        /// # Arguments
        /// * `value` - The initial value for this instance
        fn with_value(value: jint),

        /// Creates a new JTest instance initialized with a string.
        fn with_string(s: java.lang.String),

        /// Creates a JTest instance from an integer array.
        fn with_array(arr: jint[]),

        fn with_2d_int_array(arr: jint[][]),
        fn with_3d_int_array(arr: jint[][][]),
        fn with_string_array(arr: java.lang.String[]),
        fn with_2d_string_array(arr: java.lang.String[][]),
        fn with_3d_string_array(arr: java.lang.String[][][]),

        // Test constructors with Rust type names from type mappings
        fn with_string_rust_type(s: JString),
        fn with_class_rust_type(c: JClass),
        fn with_string_array_rust_type(arr: JString[]),
        fn with_string_2d_array_rust_type(arr: JString[][]),

        /// Creates a JTest instance with mixed parameter types.
        ///
        /// This demonstrates the ability to combine multiple type-mapped
        /// parameters in a single constructor.
        fn with_mixed_rust_types(name: JString, clazz: JClass, count: jint),
    }
}

fn main() {
    println!("Constructor test compiled successfully!");
}
