use jni_macros::bind_java_type;

bind_java_type! { JMinimal0 => "min.Minimal" }

bind_java_type! { JMinimal1 => "min.Minimal$Inner" }

bind_java_type! { JMinimal2 => min.Minimal }

bind_java_type! { JMinimal3 => min.Minimal::Inner }

bind_java_type! { JMinimal4 => .Minimal }

bind_java_type! { JMinimal5 => .Minimal::Inner }

bind_java_type! {
    rust_type = JMinimal6,
    java_type = min.Minimal
}

bind_java_type! {
    rust_type = JMinimal7,
    java_type = min.Minimal, // trailing comma is ok
}

bind_java_type! {
    rust_type = JMinimal8,
    java_type = min.Minimal,
    api = JMinimalAPIType,
}

fn main() {
    println!("Compiled successfully!");
}
