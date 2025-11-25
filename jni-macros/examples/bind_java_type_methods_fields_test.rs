//! Test the code generated for various method and field signatures
//! by the bind_java_type! macro
//!
use jni_macros::bind_java_type;

bind_java_type! {
    rust_type = JMethodFieldTest,
    java_type = "org.example.MethodFieldTest",
    // Test instance methods with various signatures
    methods {
        // Primitive return types
        /// Gets the integer value from this instance.
        ///
        /// # Returns
        /// The current integer value
        fn get_int {
            sig = () -> jint,
        },

        /// Gets the boolean flag from this instance.
        fn get_boolean {
            sig = () -> jboolean,
        },

        fn get_long {
            sig = () -> jlong,
        },
        fn get_double {
            sig = () -> jdouble,
        },

        // Object return types
        /// Returns the String value associated with this instance.
        ///
        /// This method returns a local reference to a Java String object.
        fn get_string {
            sig = () -> java.lang.String,
        },

        fn get_class {
            sig = () -> java.lang.Class,
        },
        fn get_object {
            sig = () -> java.lang.Object,
        },

        // Single-dimensional primitive arrays
        fn get_int_array {
            sig = () -> jint[],
        },
        fn get_byte_array {
            sig = () -> jbyte[],
        },
        fn get_float_array {
            sig = () -> jfloat[],
        },

        // Multi-dimensional primitive arrays
        fn get_int_2d_array {
            sig = () -> jint[][],
        },
        fn get_int_3d_array {
            sig = () -> jint[][][],
        },
        fn get_byte_2d_array {
            sig = () -> jbyte[][],
        },

        // Single-dimensional object arrays with default JObject elements
        fn get_object_array {
            sig = () -> java.lang.Object[],
        },

        // Single-dimensional object arrays with type-mapped elements
        fn get_string_array {
            sig = () -> java.lang.String[],
        },
        fn get_class_array {
            sig = () -> java.lang.Class[],
        },

        // Multi-dimensional object arrays with default JObject elements
        fn get_object_2d_array {
            sig = () -> java.lang.Object[][],
        },
        fn get_object_3d_array {
            sig = () -> java.lang.Object[][][],
        },

        // Multi-dimensional object arrays with type-mapped elements
        fn get_string_2d_array {
            sig = () -> java.lang.String[][],
        },
        fn get_string_3d_array {
            sig = () -> java.lang.String[][][],
        },
        fn get_class_2d_array {
            sig = () -> java.lang.Class[][],
        },

        // Methods with parameters
        /// Sets the integer value for this instance.
        fn set_int {
            sig = (value: jint) -> void,
        },

        fn set_string {
            sig = (s: java.lang.String) -> void,
        },
        fn set_int_array {
            sig = (arr: jint[]) -> void,
        },
        fn set_string_array {
            sig = (arr: java.lang.String[]) -> void,
        },
        fn set_int_2d_array {
            sig = (arr: jint[][]) -> void,
        },
        fn set_string_2d_array {
            sig = (arr: java.lang.String[][]) -> void,
        },

        // Mixed parameter types
        /// Processes data with multiple parameters.
        ///
        /// # Arguments
        /// * `id` - The identifier
        /// * `name` - The name string
        /// * `values` - Array of double values to process
        ///
        /// # Returns
        /// `true` if processing was successful, `false` otherwise
        fn process_data {
            sig = (id: jint, name: java.lang.String, values: jdouble[]) -> jboolean,
        },

        fn transform_arrays {
            sig = (input: jint[][], labels: java.lang.String[]) -> java.lang.Object[][],
        },

        // Test using Rust type names from type mappings
        fn get_string_rust_type {
            sig = () -> JString,
        },
        fn get_class_rust_type {
            sig = () -> JClass,
        },
        fn set_string_rust_type {
            sig = (s: JString) -> void,
        },
        fn set_class_rust_type {
            sig = (c: JClass) -> void,
        },
        fn get_string_array_rust_type {
            sig = () -> JString[],
        },
        fn get_string_2d_array_rust_type {
            sig = () -> JString[][],
        },
        fn process_with_rust_types {
            sig = (name: JString, clazz: JClass) -> JString,
        },

        // Test static methods

        /// Gets the singleton instance of MethodFieldTest.
        ///
        /// This static factory method returns a shared instance.
        static fn static_get_instance {
            sig = () -> java.lang.Object,
        },

        static fn static_create_array {
            sig = (size: jint) -> jint[],
        },
        static fn static_create_2d_array {
            sig = (rows: jint, cols: jint) -> jint[][],
        },
        static fn static_process_strings {
            sig = (strings: java.lang.String[]) -> java.lang.String,
        },
        static fn static_merge_arrays {
            sig = (arr1: jint[], arr2: jint[]) -> jint[],
        },

        // Test static methods with Rust type names
        static fn static_get_string_rust_type {
            sig = () -> JString,
        },
        static fn static_format {
            sig = (template: JString, args: JString[]) -> JString,
        },
    },

    // Test instance fields with various types
    fields = {
        // Primitive fields
        /// The integer field stores a simple integer value.
        ///
        /// This field can be read and written using the generated
        /// getter and setter methods.
        int_field {
            sig = jint,
        },

        /// A boolean flag indicating some state.
        boolean_field {
            sig = jboolean,
        },

        long_field {
            sig = jlong,
        },
        double_field {
            sig = jdouble,
        },

        // Object fields
        string_field {
            sig = java.lang.String,
        },
        class_field {
            sig = java.lang.Class,
        },
        object_field {
            sig = java.lang.Object,
        },

        // Single-dimensional primitive array fields
        int_array_field {
            sig = jint[],
        },
        byte_array_field {
            sig = jbyte[],
        },

        // Multi-dimensional primitive array fields
        int_2d_array_field {
            sig = jint[][],
        },
        int_3d_array_field {
            sig = jint[][][],
        },

        // Single-dimensional object array fields with default JObject elements
        object_array_field {
            sig = java.lang.Object[],
        },

        // Single-dimensional object array fields with type-mapped elements
        string_array_field {
            sig = java.lang.String[],
        },
        class_array_field {
            sig = java.lang.Class[],
        },

        // Multi-dimensional object array fields with default JObject elements
        object_2d_array_field {
            sig = java.lang.Object[][],
        },
        object_3d_array_field {
            sig = java.lang.Object[][][],
        },

        // Multi-dimensional object array fields with type-mapped elements
        string_2d_array_field {
            sig = java.lang.String[][],
        },
        string_3d_array_field {
            sig = java.lang.String[][][],
        },
        class_2d_array_field {
            sig = java.lang.Class[][],
        },

        // Custom getter/setter names with custom documentation for each
        /// This field has custom getter and setter names.
        ///
        /// The underlying Java field is named "customValue".
        custom_value {
            name = "customValue",
            sig = jint,

            /// Gets the custom value with special processing.
            ///
            /// This getter has its own documentation separate from the field.
            get = get_custom,

            /// Sets the custom value with validation.
            ///
            /// This setter has its own documentation separate from the field.
            ///
            /// # Arguments
            /// * `val` - The new value to set (must be positive)
            set = set_custom,
        },

        // Test fields with Rust type names
        string_field_rust_type {
            sig = JString,
        },
        class_field_rust_type {
            sig = JClass,
        },
        string_array_field_rust_type {
            sig = JString[],
        },
        string_2d_array_field_rust_type {
            sig = JString[][],
        },

        // Test static fields

        /// A static counter that tracks something across all instances.
        static static_counter {
            sig = jint,
        },

        static static_name {
            sig = java.lang.String,
        },
        static static_values {
            sig = jdouble[],
        },
        static static_string_array {
            sig = java.lang.String[],
        },
        static static_int_2d_array {
            sig = jint[][],
        },
        static static_string_2d_array {
            sig = java.lang.String[][],
        },
        static static_object_array {
            sig = java.lang.Object[],
        },

        // Test static fields with Rust type names
        static static_string_rust_type {
            sig = JString,
        },
        static static_class_rust_type {
            sig = JClass,
        },
        static static_string_array_rust_type {
            sig = JString[],
        },
    }
}

fn main() {
    println!("Compiled successfully!");
}
