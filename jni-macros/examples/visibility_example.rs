//! This example demonstrates visibility specifiers in bind_java_type macro

use jni_macros::bind_java_type;

// Test basic visibility on methods
bind_java_type! {
    rust_type = TestMethods,
    java_type = "com.example.TestMethods",
    methods {
        // Public method (default)
        fn public_method() -> void,

        // Private method using pub(self)
        pub(self) fn private_method() -> void,

        // Private method using priv keyword
        priv fn another_private() -> void,

        // pub(crate) method
        pub(crate) fn crate_method() -> void,
    }
}

// Test visibility on fields
bind_java_type! {
    rust_type = TestFields,
    java_type = "com.example.TestFields",
    fields {
        // Public field (default)
        public_field: int,

        // Private field
        pub(self) private_field: int,

        // Field with priv keyword
        priv another_private_field: int,
    }
}

// Test independent getter/setter visibility
bind_java_type! {
    rust_type = TestGetterSetter,
    java_type = "com.example.TestGetterSetter",
    fields {
        // Public getter, private setter
        mixed {
            sig = int,
            pub get = get_mixed,
            pub(self) set = set_mixed,
        },
    }
}

fn main() {
    println!("This example demonstrates macro expansion only");
}
