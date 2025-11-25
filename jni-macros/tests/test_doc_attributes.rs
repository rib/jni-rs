// Test that doc attributes work on the type definition

#[test]
fn test_custom_doc_attributes() {
    // This test just verifies compilation succeeds
    // The actual doc output would need cargo doc to verify
}

jni_macros::bind_java_type! {
    /// Bindings for the java.foo.Bar class
    ///
    /// This is a custom documentation comment.
    /// It has multiple lines.
    FooBar => "java.foo.Bar"
}

// Test without doc attribute (should get default docs)
jni_macros::bind_java_type! {
    Baz => "java.baz.Baz"
}

// Test with other attributes (like #[allow(dead_code)])
jni_macros::bind_java_type! {
    #[allow(dead_code)]
    /// Custom docs with other attributes
    Qux => "java.qux.Qux"
}

jni_macros::bind_java_type! {
    /// Bindings for the com.example.WithMethods class
    rust_type = WithMethods,
    java_type = "com.example.WithMethods",
    methods {
        /// This is a test method
        fn test_method() -> void,
    }
}

fn main() {}
