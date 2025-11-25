//! Example usage of the `jni_sig!` macro for both method and field signatures

use jni::signature::{FieldSignature, MethodSignature};
use jni_macros::jni_sig;

// Check that we can define const signatures

const TO_STRING_METHOD: MethodSignature = jni_sig!(() -> java.lang.String);
const EQUALS_METHOD: MethodSignature = jni_sig!((other: java.lang.Object) -> jboolean);
const HASH_CODE_METHOD: MethodSignature = jni_sig!(() -> jint);
const STRING_FIELD: FieldSignature = jni_sig!(java.lang.String);
const INT_FIELD: FieldSignature = jni_sig!(jint);
const PROCESS_ARRAY_METHOD: MethodSignature =
    jni_sig!((data: [jint], labels: [java.lang.String]) -> java.lang.Object);

fn main() {
    println!("=== Global Constant Signatures ===\n");

    println!("TO_STRING_METHOD: {}", TO_STRING_METHOD.sig());
    println!("  Args: {}", TO_STRING_METHOD.args().len());
    println!("  Return type: {:?}\n", TO_STRING_METHOD.ret());

    println!("EQUALS_METHOD: {}", EQUALS_METHOD.sig());
    println!("  Args: {}", EQUALS_METHOD.args().len());
    println!("  Return type: {:?}\n", EQUALS_METHOD.ret());

    println!("HASH_CODE_METHOD: {}", HASH_CODE_METHOD.sig());
    println!("  Args: {}", HASH_CODE_METHOD.args().len());
    println!("  Return type: {:?}\n", HASH_CODE_METHOD.ret());

    println!("STRING_FIELD: {}", STRING_FIELD.sig());
    println!("  Type: {:?}\n", STRING_FIELD.ty());

    println!("INT_FIELD: {}", INT_FIELD.sig());
    println!("  Type: {:?}\n", INT_FIELD.ty());

    println!("PROCESS_ARRAY_METHOD: {}", PROCESS_ARRAY_METHOD.sig());
    println!("  Args: {}", PROCESS_ARRAY_METHOD.args().len());
    println!("  Return type: {:?}\n", PROCESS_ARRAY_METHOD.ret());

    // Verify the signatures are correct
    assert_eq!(TO_STRING_METHOD.sig().to_bytes(), b"()Ljava/lang/String;");
    assert_eq!(EQUALS_METHOD.sig().to_bytes(), b"(Ljava/lang/Object;)Z");
    assert_eq!(HASH_CODE_METHOD.sig().to_bytes(), b"()I");
    assert_eq!(STRING_FIELD.sig().to_bytes(), b"Ljava/lang/String;");
    assert_eq!(INT_FIELD.sig().to_bytes(), b"I");
    assert_eq!(
        PROCESS_ARRAY_METHOD.sig().to_bytes(),
        b"([I[Ljava/lang/String;)Ljava/lang/Object;"
    );

    println!("✓ All global constant signatures are valid!\n");

    println!("=== Method Signatures ===\n");

    // Basic method with primitives
    let method1 = jni_sig!((a: jint, b: jboolean) -> void);
    println!("Method: (jint, jboolean) -> void");
    println!("  Signature: {}\n", method1.sig());
    assert_eq!(method1.sig().to_bytes(), b"(IZ)V");

    // Method with Java objects
    let method2 = jni_sig!(
        (a: jint, b: java.lang.String) -> java.lang.Object
    );
    println!("Method: (jint, String) -> Object");
    println!("  Signature: {}\n", method2.sig());
    assert_eq!(
        method2.sig().to_bytes(),
        b"(ILjava/lang/String;)Ljava/lang/Object;"
    );

    // Method with arrays
    let method3 = jni_sig!(
        (a: [jint], b: [java.lang.String]) -> [[jint]]
    );
    println!("Method: ([jint], [String]) -> [[jint]]");
    println!("  Signature: {}\n", method3.sig());
    assert_eq!(method3.sig().to_bytes(), b"([I[Ljava/lang/String;)[[I");

    // Method with built-in types (no explicit type mappings needed)
    let method4 = jni_sig!(
        (a: jint, b: JString, c: [JObject]) -> JThrowable
    );
    println!("Method: (jint, JString, [JObject]) -> JThrowable (built-in types)");
    println!("  Signature: {}\n", method4.sig());
    assert_eq!(
        method4.sig().to_bytes(),
        b"(ILjava/lang/String;[Ljava/lang/Object;)Ljava/lang/Throwable;"
    );

    // Method with no arguments
    let method5 = jni_sig!(() -> jint);
    println!("Method: () -> jint");
    println!("  Signature: {}\n", method5.sig());
    assert_eq!(method5.sig().to_bytes(), b"()I");

    // Method with inner class
    let method6 = jni_sig!(
        (a: java.lang.Outer::Inner) -> void
    );
    println!("Method: (Outer::Inner) -> void");
    println!("  Signature: {}\n", method6.sig());
    assert_eq!(method6.sig().to_bytes(), b"(Ljava/lang/Outer$Inner;)V");

    println!("=== Field Signatures ===\n");

    // Primitive field
    let field1 = jni_sig!(jint);
    println!("Field: jint");
    println!("  Signature: {}\n", field1.sig());
    assert_eq!(field1.sig().to_bytes(), b"I");

    // Object field
    let field2 = jni_sig!(java.lang.String);
    println!("Field: String");
    println!("  Signature: {}\n", field2.sig());
    assert_eq!(field2.sig().to_bytes(), b"Ljava/lang/String;");

    // Array field (prefix syntax)
    let field3 = jni_sig!([jint]);
    println!("Field: [jint]");
    println!("  Signature: {}\n", field3.sig());
    assert_eq!(field3.sig().to_bytes(), b"[I");

    // Array field (suffix syntax)
    let field4 = jni_sig!(java.lang.String[][]);
    println!("Field: String[][]");
    println!("  Signature: {}\n", field4.sig());
    assert_eq!(field4.sig().to_bytes(), b"[[Ljava/lang/String;");

    // Field with built-in type (no explicit type mapping needed)
    let field5 = jni_sig!(JString);
    println!("Field: JString (built-in type)");
    println!("  Signature: {}\n", field5.sig());
    assert_eq!(field5.sig().to_bytes(), b"Ljava/lang/String;");

    // Inner class field
    let field6 = jni_sig!(java.lang.Outer::Inner);
    println!("Field: Outer::Inner");
    println!("  Signature: {}\n", field6.sig());
    assert_eq!(field6.sig().to_bytes(), b"Ljava/lang/Outer$Inner;");

    println!("=== Optional Parameter Names ===\n");

    // Method with optional parameter names (types only)
    let method7 = jni_sig!((jint, jboolean, java.lang.String) -> void);
    println!("Method: (jint, jboolean, String) -> void [unnamed params]");
    println!("  Signature: {}\n", method7.sig());
    assert_eq!(method7.sig().to_bytes(), b"(IZLjava/lang/String;)V");

    // Method mixing named and unnamed parameters
    let method8 = jni_sig!((jint, name: java.lang.String) -> void);
    println!("Method: (jint, name: String) -> void [mixed params]");
    println!("  Signature: {}\n", method8.sig());
    assert_eq!(method8.sig().to_bytes(), b"(ILjava/lang/String;)V");

    println!("=== Comparison: Method vs Field ===\n");

    // Example: Java class with field and accessor methods
    println!("Java class:");
    println!("  private int count;");
    println!("  public void setCount(int value) {{ }}");
    println!("  public int getCount() {{ }}\n");

    let field = jni_sig!(jint);
    let setter = jni_sig!((value: jint) -> void);
    let getter = jni_sig!(() -> jint);

    println!("Field 'count': {:?}", field.sig());
    println!("Method 'setCount': {:?}", setter.sig());
    println!("Method 'getCount': {:?}", getter.sig());

    assert_eq!(field.sig().to_bytes(), b"I");
    assert_eq!(setter.sig().to_bytes(), b"(I)V");
    assert_eq!(getter.sig().to_bytes(), b"()I");

    println!("\n✓ All examples passed!");
}
