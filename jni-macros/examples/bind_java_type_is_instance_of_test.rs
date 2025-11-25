//! Test is_instance_of codegen

// First define some parent types
jni_macros::bind_java_type! { BaseClass => "com.example.BaseClass" }
jni_macros::bind_java_type! { AnotherClass => "com.example.AnotherClass" }
jni_macros::bind_java_type! { YetAnotherClass => "com.example.YetAnotherClass" }

// Now define MyClass which is an instance of the above types
jni_macros::bind_java_type! {
    rust_type = MyClass,
    java_type = "com.example.MyClass",
    type_map = {
        BaseClass => "com.example.BaseClass",
        AnotherClass => "com.example.AnotherClass",
        YetAnotherClass => "com.example.YetAnotherClass",
    },

    is_instance_of = {
        // With explicit stem names using = separator
        base = BaseClass,
        // With explicit stem names using : separator
        another: AnotherClass,
        // Without stem
        YetAnotherClass,
        // Can alias built-in types too
        JThrowable,
        // But shouldn't be allowed to explicitly alias JObject
        // JObject
    }
}

fn main() {
    println!("Compiled successfully!");
}
