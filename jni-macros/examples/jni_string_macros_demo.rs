//! Example demonstrating the jni_cstr! and jni_str! macros
//!
//! These macros convert UTF-8 string literals to MUTF-8 encoded literals.
//!
//! This example also demonstrates roundtrip encoding/decoding using cesu8 to verify
//! that the MUTF-8 encoding is correct.

use jni::strings::JNIStr;
use jni_macros::{jni_cstr, jni_str};
use std::ffi::CStr;

/// Helper function to show UTF-8 vs MUTF-8 encoding comparison
fn show_encoding_comparison(label: &str, original: &str, mutf8_bytes: &[u8]) {
    let utf8_bytes = original.as_bytes();

    println!("   {}:", label);
    println!("     Original string: \"{}\"", original);
    println!("     UTF-8 bytes:     {:?} ({} bytes)", utf8_bytes, utf8_bytes.len());
    println!("     MUTF-8 bytes:    {:?} ({} bytes)", mutf8_bytes, mutf8_bytes.len());

    // Decode MUTF-8 back to UTF-8 to verify correctness
    match cesu8::from_java_cesu8(mutf8_bytes) {
        Ok(decoded) => {
            if decoded == original {
                println!("     ✓ Roundtrip successful: MUTF-8 → UTF-8 matches original");
            } else {
                println!("     ✗ Roundtrip failed: decoded = \"{}\"", decoded);
            }
        }
        Err(e) => {
            println!("     ✗ Decode error: {:?}", e);
        }
    }

    if utf8_bytes == mutf8_bytes {
        println!("     Note: UTF-8 and MUTF-8 are identical for this string");
    } else {
        println!("     Note: MUTF-8 differs from UTF-8 (contains high Unicode or nulls)");
    }
}

fn main() {
    println!("=== JNI String Macro Demonstrations ===\n");

    // Example 1: jni_cstr! - returns &CStr with MUTF-8 encoding
    println!("1. jni_cstr! - Returns &CStr (MUTF-8 encoded)");
    const CLASS_CSTR: &CStr = jni_cstr!("java.lang.String");
    const PACKAGE_CSTR: &CStr = jni_cstr!("com.example.myapp");

    println!("   Class name:   {}", CLASS_CSTR.to_string_lossy());
    println!("   Package name: {}", PACKAGE_CSTR.to_string_lossy());
    println!();

    // Example 2: jni_str! - returns &'static JNIStr with MUTF-8 encoding
    println!("2. jni_str! - Returns &'static JNIStr (MUTF-8 encoded)");
    const CLASS_JSTR: &JNIStr = jni_str!("java.lang.String");
    const PACKAGE_JSTR: &JNIStr = jni_str!("com.example.myapp");

    println!(
        "   Class name:   {}",
        String::from_utf8_lossy(CLASS_JSTR.to_bytes())
    );
    println!(
        "   Package name: {}",
        String::from_utf8_lossy(PACKAGE_JSTR.to_bytes())
    );
    println!();

    // Example 3: Unicode support - Emoji (demonstrates surrogate pair encoding)
    println!("3. Unicode Support - Emoji (High Unicode Requires Surrogate Pairs)");
    const EMOJI_CSTR: &CStr = jni_cstr!("emoji.Type😀");
    const _EMOJI_JSTR: &JNIStr = jni_str!("emoji.Type😀");

    show_encoding_comparison(
        "Class with emoji",
        "emoji.Type😀",
        EMOJI_CSTR.to_bytes(),
    );
    println!();
    println!("   Details: The emoji 😀 (U+1F600) is above U+FFFF");
    println!("            UTF-8:  4 bytes (F0 9F 98 80)");
    println!("            MUTF-8: 6 bytes (ED A0 BD ED B8 80) - surrogate pair encoding");
    println!();

    // Example 4: Unicode support - Japanese (same encoding in UTF-8 and MUTF-8)
    println!("4. Unicode Support - Japanese Characters");
    const JP_CSTR: &CStr = jni_cstr!("jp.こんにちは");
    const _JP_JSTR: &JNIStr = jni_str!("jp.こんにちは");

    show_encoding_comparison(
        "Class with Japanese",
        "jp.こんにちは",
        JP_CSTR.to_bytes(),
    );
    println!();
    println!("   Note: Japanese characters (U+3000-U+30FF) are below U+FFFF,");
    println!("         so UTF-8 and MUTF-8 encode them identically!");
    println!();

    // Example 5: Compatibility - Both produce same encoding
    println!("5. Compatibility - jni_cstr! and jni_str! produce same encoding");
    const TEST_CSTR: &CStr = jni_cstr!("test.Example");
    const TEST_JSTR: &JNIStr = jni_str!("test.Example");

    assert_eq!(TEST_CSTR.to_bytes(), TEST_JSTR.to_bytes());
    println!("   ✓ Both produce identical byte sequences!");
    println!("   CStr:  {:?}", TEST_CSTR.to_bytes());
    println!("   JNIStr: {:?}", TEST_JSTR.to_bytes());
    println!();

    // Example 6: Use cases
    println!("6. Common Use Cases");
    println!("   - JNI class lookups:");
    const ARRAY_LIST: &JNIStr = jni_str!("java.util.ArrayList");
    println!("     {}", String::from_utf8_lossy(ARRAY_LIST.to_bytes()));

    println!("   - Method signatures:");
    const METHOD_NAME: &CStr = jni_cstr!("toString");
    println!("     {}", METHOD_NAME.to_string_lossy());

    println!("   - Field names:");
    const FIELD: &CStr = jni_cstr!("value");
    println!("     {}", FIELD.to_string_lossy());
    println!();

    // Example 7: Understanding MUTF-8 Encoding (additional emoji examples)
    println!("7. Understanding MUTF-8 Encoding - More Examples");
    println!("   High Unicode characters (above U+FFFF) use surrogate pairs:");
    println!();

    const THUMBS_UP: &JNIStr = jni_str!("👍");
    show_encoding_comparison("Thumbs up emoji", "👍", THUMBS_UP.to_bytes());
    println!();

    const ROCKET: &JNIStr = jni_str!("🚀");
    show_encoding_comparison("Rocket emoji", "🚀", ROCKET.to_bytes());

    println!();
    println!("   Key Takeaway:");
    println!("   - MUTF-8 encoded high Unicode (U+10000+) is NOT valid UTF-8!");
    println!("   - The encoding uses UTF-16 surrogate pairs (6 bytes vs 4)");
    println!("   - Must be decoded by Java or JNI functions expecting MUTF-8");
    println!("   - Roundtrip decode proves the encoding is correct!");
    println!();

    println!("=== All demonstrations complete! ===");
}
