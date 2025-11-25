use jni_macros::bind_java_type;

bind_java_type! {
    rust_type = TestAttributeVisibility,
    java_type = "com.example.TestAttributeVisibility",
    fields {
        // Test that attributes before visibility work correctly
        test_field {
            sig = i32,
            #[doc = "A test getter"]
            pub(self) get = private_getter,

            #[doc = "A test setter"]
            pub(crate) set = public_setter,
        },

        another_field {
            sig = i32,
            #[doc = "Another getter with priv"]
            priv get = another_getter,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_attribute_visibility_combination() {
        // This test just checks that the code compiles successfully
        // with attributes before visibility specifiers
    }
}
