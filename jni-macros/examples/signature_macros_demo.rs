//! Example demonstrating the different signature macro variants
//!
//! This example shows how to use:
//! - `jni_sig!` - Returns a `MethodSignature` or `FieldSignature` struct
//! - `jni_sig_str!` - Returns a plain `&str` literal
//! - `jni_sig_cstr!` - Returns a `&CStr` literal (MUTF-8 encoded)
//! - `jni_sig_jstr!` - Returns a `&'static JNIStr` (MUTF-8 encoded)

use jni::signature::{FieldSignature, MethodSignature};
use jni::strings::JNIStr;
use jni_macros::{jni_sig, jni_sig_cstr, jni_sig_jstr, jni_sig_str};
use std::ffi::CStr;

fn main() {
    println!("=== JNI Signature Macro Demonstrations ===\n");

    // Example 1: jni_sig! - returns typed signature structs
    println!("1. jni_sig! - Returns MethodSignature or FieldSignature");
    const METHOD_SIG: MethodSignature = jni_sig!((a: jint, b: java.lang.String) -> void);
    const FIELD_SIG: FieldSignature = jni_sig!(java.lang.String);

    println!("   Method signature: {:?}", METHOD_SIG.sig());
    println!("   Field signature:  {:?}", FIELD_SIG.sig());
    println!();

    // Example 2: jni_sig_str! - returns plain &str
    println!("2. jni_sig_str! - Returns &str");
    const METHOD_STR: &str = jni_sig_str!((a: jint, b: java.lang.String) -> void);
    const FIELD_STR: &str = jni_sig_str!(java.lang.String);

    println!("   Method signature: {}", METHOD_STR);
    println!("   Field signature:  {}", FIELD_STR);
    println!();

    // Example 3: jni_sig_cstr! - returns &CStr (MUTF-8 encoded)
    println!("3. jni_sig_cstr! - Returns &CStr (MUTF-8 encoded)");
    const METHOD_CSTR: &CStr = jni_sig_cstr!((a: jint, b: java.lang.String) -> void);
    const FIELD_CSTR: &CStr = jni_sig_cstr!(java.lang.String);

    println!("   Method signature: {}", METHOD_CSTR.to_string_lossy());
    println!("   Field signature:  {}", FIELD_CSTR.to_string_lossy());
    println!();

    // Example 4: jni_sig_jstr! - returns &'static JNIStr (MUTF-8 encoded)
    println!("4. jni_sig_jstr! - Returns &'static JNIStr (MUTF-8 encoded)");
    const METHOD_JSTR: &JNIStr = jni_sig_jstr!((a: jint, b: java.lang.String) -> void);
    const FIELD_JSTR: &JNIStr = jni_sig_jstr!(java.lang.String);

    println!(
        "   Method signature: {}",
        String::from_utf8_lossy(METHOD_JSTR.to_bytes())
    );
    println!(
        "   Field signature:  {}",
        String::from_utf8_lossy(FIELD_JSTR.to_bytes())
    );
    println!();

    // Example 5: Complex signatures with arrays and inner classes
    println!("5. Complex signatures");
    const COMPLEX_METHOD: &str =
        jni_sig_str!(([jint], java.lang.Thread::State) -> [[java.lang.String]]);
    const COMPLEX_FIELD: &str = jni_sig_str!([java.util.Map]);

    println!("   Complex method: {}", COMPLEX_METHOD);
    println!("   Complex field:  {}", COMPLEX_FIELD);
    println!();

    // Example 6: With type mappings
    println!("6. With type mappings");
    const MAPPED_SIG: &str = jni_sig_str!(
        type_map = {
            MyType => com.example.MyClass,
            OtherType => com.example.OtherClass,
            ResultType => com.example.ResultClass,
        },
        (a: MyType, b: [OtherType]) -> ResultType,
    );

    println!("   Mapped signature: {}", MAPPED_SIG);
    println!();

    // Example 7: Raw JNI signatures (validation at compile time)
    println!("7. Raw JNI signatures");
    const RAW_METHOD: &str = jni_sig_str!("(ILjava/lang/String;)V");
    const RAW_FIELD: &str = jni_sig_str!("Ljava/lang/String;");

    println!("   Raw method: {}", RAW_METHOD);
    println!("   Raw field:  {}", RAW_FIELD);
    println!();

    // Example 8: All variants produce compatible output
    println!("8. All variants produce compatible byte representations");
    let sig1 = jni_sig!((a: jint) -> void);
    let sig2 = jni_sig_cstr!((a: jint) -> void);
    let sig3 = jni_sig_jstr!((a: jint) -> void);

    println!("   jni_sig!:      {:?}", sig1.sig().to_bytes());
    println!("   jni_sig_cstr!: {:?}", sig2.to_bytes());
    println!("   jni_sig_jstr!: {:?}", sig3.to_bytes());

    assert_eq!(sig1.sig().to_bytes(), sig2.to_bytes());
    assert_eq!(sig1.sig().to_bytes(), sig3.to_bytes());
    println!("   ✓ All produce the same bytes!");
}
