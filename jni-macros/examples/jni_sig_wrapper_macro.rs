//! Example demonstrating a wrapper macro that can inject jni crate path
//!
//! This shows how the flexible property ordering allows creating wrapper macros
//! that can unconditionally inject properties without needing to parse the syntax.

use jni::signature::MethodSignature;

// Example wrapper macro that always uses a custom jni path
// This could be useful in a workspace with a renamed jni dependency
macro_rules! my_jni_sig {
    // Pass through all arguments and inject jni = ::jni at the start
    ($($tt:tt)*) => {
        jni_macros::jni_sig!(
            jni = ::jni,
            $($tt)*
        )
    };
}

fn main() {
    println!("=== Wrapper Macro Example ===\n");

    // The wrapper macro works with all syntax variations:

    // 1. Simple unnamed signature
    let sig1: MethodSignature = my_jni_sig!((a: jint) -> void);
    println!("Simple: {}", sig1.sig());
    assert_eq!(sig1.sig().to_bytes(), b"(I)V");

    // 2. With type_map
    let sig2: MethodSignature = my_jni_sig!(
        (a: MyType) -> void,
        type_map = {
            MyType => com.example.MyType,
        }
    );
    println!("With type_map: {}", sig2.sig());
    assert_eq!(sig2.sig().to_bytes(), b"(Lcom/example/MyType;)V");

    // 3. Named signature
    let sig3: MethodSignature = my_jni_sig!(
        sig = (a: jint) -> void
    );
    println!("Named sig: {}", sig3.sig());
    assert_eq!(sig3.sig().to_bytes(), b"(I)V");

    // 4. With trailing comma (no parsing needed!)
    let sig4: MethodSignature = my_jni_sig!(
        (a: jint) -> void,
    );
    println!("With trailing comma: {}", sig4.sig());
    assert_eq!(sig4.sig().to_bytes(), b"(I)V");

    // 5. All properties mixed
    let sig5: MethodSignature = my_jni_sig!(
        type_map = { AnotherType => com.example.Another },
        sig = (a: AnotherType) -> void,
    );
    println!("All mixed: {}", sig5.sig());
    assert_eq!(sig5.sig().to_bytes(), b"(Lcom/example/Another;)V");
}
