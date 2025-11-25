//! Test the new bind_java_type syntax

use jni_macros::bind_java_type;

// Test shorthand syntax
bind_java_type! { TestShorthand => "com.example.TestShorthand" }

// Test canonical syntax with minimal properties
bind_java_type! {
    rust_type = TestCanonical,
    java_type = "com.example.TestCanonical"
}

// Test canonical syntax with jni override as first property
bind_java_type! {
    jni = ::jni,
    rust_type = TestWithJni,
    java_type = "com.example.TestWithJni"
}

// Test canonical syntax with properties
bind_java_type! {
    rust_type = TestWithProperties,
    java_type = "com.example.TestWithProperties",
    constructors {
        fn new() -> ()
    },
    methods {
        fn test() -> ()
    }
}

// Test canonical syntax with trailing commas
bind_java_type! {
    rust_type = TestTrailingCommas,
    java_type = "com.example.TestTrailingCommas",
}

// Test canonical syntax with optional equals before blocks
bind_java_type! {
    rust_type = TestOptionalEquals,
    java_type = "com.example.TestOptionalEquals",
    constructors = {
        fn new() -> ()
    },
    methods = {
        fn test() -> ()
    },
}

// First generate the types that will be used in type_map
bind_java_type! { CustomType => "com.example.CustomType" }
bind_java_type! { AnotherType => "com.example.AnotherType" }

// Test type_map with new syntax (=> instead of as)
bind_java_type! {
    rust_type = TestTypeMap,
    java_type = "com.example.TestTypeMap",
    type_map {
        CustomType => "com.example.CustomType",
        AnotherType => "com.example.AnotherType",
    },
    methods {
        fn get_custom() -> CustomType,
        fn process(arg: AnotherType) -> ()
    }
}

fn main() {
    println!("All syntax variations compiled successfully!");
}
