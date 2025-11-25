//! Example demonstrating the new named property syntax for jni_sig!

use jni_macros::jni_sig;

fn main() {
    println!("=== New Named Property Syntax Examples ===\n");

    // Traditional unnamed signature (still works)
    let sig1 = jni_sig!((a: jint) -> void);
    println!("Traditional: {}", sig1.sig());
    assert_eq!(sig1.sig().to_bytes(), b"(I)V");

    // Named signature with sig =
    let sig2 = jni_sig!(sig = (a: jint) -> void);
    println!("Named sig: {}", sig2.sig());
    assert_eq!(sig2.sig().to_bytes(), b"(I)V");

    // Type map with new syntax (requires type_map = prefix)
    let sig3 = jni_sig!(
        CustomType,
        type_map = {
            CustomType => java.lang.Custom,
        }
    );
    println!("With type_map: {}", sig3.sig());
    assert_eq!(sig3.sig().to_bytes(), b"Ljava/lang/Custom;");

    // Named signature with type_map in any order
    let sig4 = jni_sig!(
        type_map = {
            MyType => java.lang.MyType,
        },
        sig = (arg: MyType) -> void
    );
    println!("Named sig + type_map: {}", sig4.sig());
    assert_eq!(sig4.sig().to_bytes(), b"(Ljava/lang/MyType;)V");

    // Unnamed signature first, then type_map
    let sig5 = jni_sig!(
        (arg: AnotherType) -> void,
        type_map = {
            AnotherType => java.lang.AnotherType,
        }
    );
    println!("Unnamed first: {}", sig5.sig());
    assert_eq!(sig5.sig().to_bytes(), b"(Ljava/lang/AnotherType;)V");

    // With custom jni crate path
    let sig6 = jni_sig!(
        jni = ::jni,
        (a: jint) -> void
    );
    println!("With jni override: {}", sig6.sig());
    assert_eq!(sig6.sig().to_bytes(), b"(I)V");

    // All properties together
    let sig7 = jni_sig!(
        jni = ::jni,
        type_map = {
            FullType => java.lang.Full,
        },
        sig = (arg: FullType) -> void,
    );
    println!("All properties: {}", sig7.sig());
    assert_eq!(sig7.sig().to_bytes(), b"(Ljava/lang/Full;)V");

    // Field signatures work the same way
    let field1 = jni_sig!(jint);
    println!("\nField traditional: {}", field1.sig());
    assert_eq!(field1.sig().to_bytes(), b"I");

    let field2 = jni_sig!(sig = jint);
    println!("Field named: {}", field2.sig());
    assert_eq!(field2.sig().to_bytes(), b"I");

    let field3 = jni_sig!(
        MyFieldType,
        type_map = {
            MyFieldType => com.example.CustomType,
        }
    );
    println!("Field with type_map: {}", field3.sig());
    assert_eq!(field3.sig().to_bytes(), b"Lcom/example/CustomType;");

    // Properties can appear in any order (except unnamed sig must be first)
    let field4 = jni_sig!(
        type_map = {
            OrderTest => java.lang.OrderTest,
        },
        sig = OrderTest
    );
    println!("Field properties any order: {}", field4.sig());
    assert_eq!(field4.sig().to_bytes(), b"Ljava/lang/OrderTest;");

    // Demonstrate wrapper macro use case - unnamed signature can be anywhere
    println!("\n=== Wrapper Macro Use Case ===\n");

    // Simulating what a wrapper macro might do: inject jni = at the start
    let sig8 = jni_sig!(
        jni = ::jni,
        (a: jint) -> void
    );
    println!("Injected jni at start: {}", sig8.sig());
    assert_eq!(sig8.sig().to_bytes(), b"(I)V");

    // Or inject before type_map
    let sig9 = jni_sig!(
        jni = ::jni,
        type_map = { WrapperType => com.example.Wrapper },
        (a: WrapperType) -> void
    );
    println!("Injected jni before type_map: {}", sig9.sig());
    assert_eq!(sig9.sig().to_bytes(), b"(Lcom/example/Wrapper;)V");

    // With trailing comma
    let sig10 = jni_sig!(
        jni = ::jni,
        (a: jint) -> void,
    );
    println!("Injected jni at start with trailing comma: {}", sig10.sig());
    assert_eq!(sig10.sig().to_bytes(), b"(I)V");

    println!("\n✓ All new syntax examples passed!");
}
