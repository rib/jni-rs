mod mangle;

/// Mangles a Rust function name into a JNI-compatible, native method name.
///
/// This attribute takes one to three string literal arguments:
/// 1. Package namespace (required)
/// 2. Method name (optional)
/// 3. JNI signature (optional)
///
/// If two arguments are given, the second is inferred to be a method name if it doesn't contain
/// '(', otherwise it's treated as a signature.
///
/// The name is mangled according to the JNI Specification, under "Design" -> "Resolving Native Method Names"
///
/// https://docs.oracle.com/en/java/javase/11/docs/specs/jni/design.html#resolving-native-method-names
///
/// # Method Name Generation
///
/// If no method name is provided, the Rust function name is converted from `snake_case` to `lowerCamelCase`.
///
/// If the Rust function name is not entirely lowercase with underscores (i.e. it contains any uppercase letters),
/// the name is used directly without transformation.
///
/// ## ABI Handling
///
/// The macro requires the ABI to be `extern "system"` (required for JNI).
/// - If no ABI is specified, it will automatically be set to `extern "system"`
/// - If `extern "system"` is already specified, it will be preserved
/// - If any other ABI (e.g., `extern "C"`) is specified, a compile error will be generated
///
/// # Examples
///
/// Basic usage with just namespace (function name converted to lowerCamelCase):
/// ```ignore
/// use jni::{ JNIEnv, objects::{ JClass, JString }, sys::jstring };
/// use jni_macros::jni_mangle;
///
/// // Rust function in snake_case
/// #[jni_mangle("com.example.RustBindings")]
/// pub fn say_hello(mut env: JNIEnv, _: JClass, name: JString) -> jstring {
///     // ...
/// #     unimplemented!()
/// }
/// // Generates: Java_com_example_RustBindings_sayHello
///
/// // Or already in lowerCamelCase (idempotent)
/// #[jni_mangle("com.example.RustBindings")]
/// pub fn sayHello(mut env: JNIEnv, _: JClass, name: JString) -> jstring {
///     // ...
/// #     unimplemented!()
/// }
/// // Generates: Java_com_example_RustBindings_sayHello
/// ```
///
/// With custom method name:
/// ```ignore
/// # use jni::{ JNIEnv, objects::JClass };
/// # use jni_macros::jni_mangle;
/// #[jni_mangle("com.example.RustBindings", "customMethodName")]
/// pub fn some_rust_function(env: JNIEnv, _: JClass) { }
/// // Generates: Java_com_example_RustBindings_customMethodName
/// ```
///
/// With signature only (overloaded method):
/// ```ignore
/// # use jni::{ JNIEnv, objects::JClass };
/// # use jni_macros::jni_mangle;
/// #[jni_mangle("com.example.RustBindings", "(I)Z")]
/// pub fn boolean_method(env: JNIEnv, _: JClass) { }
/// // Generates: Java_com_example_RustBindings_booleanMethod__I
/// // Note: Only argument types are encoded (I), return type (Z) is ignored
/// ```
///
/// With signature and no arguments (overloaded method):
/// ```ignore
/// # use jni::{ JNIEnv, objects::JClass };
/// # use jni_macros::jni_mangle;
/// #[jni_mangle("com.example.RustBindings", "()V")]
/// pub fn no_args_method(env: JNIEnv, _: JClass) { }
/// // Generates: Java_com_example_RustBindings_noArgsMethod__
/// // Note: __ suffix indicates overloaded method even with no arguments
/// ```
///
/// With method name and signature:
/// ```ignore
/// # use jni::{ JNIEnv, objects::JClass };
/// # use jni_macros::jni_mangle;
/// #[jni_mangle("com.example.RustBindings", "customName", "(Ljava/lang/String;)V")]
/// pub fn another_function(env: JNIEnv, _: JClass) { }
/// // Generates: Java_com_example_RustBindings_customName__Ljava_lang_String_2
/// // Note: Only argument types are encoded, return type (V) is ignored
/// ```
///
/// Pre-existing ABI is automatically overridden:
/// ```ignore
/// # use jni::{ JNIEnv, objects::JClass };
/// # use jni_macros::jni_mangle;
/// #[jni_mangle("com.example.RustBindings")]
/// pub extern "C" fn my_function(env: JNIEnv, _: JClass) { }
/// // The "C" ABI is overridden with "system"
/// ```
///
/// The `sayHello` function will automatically be expanded to have the correct ABI specification
/// and the appropriate JNI-compatible name, i.e. in this case -
/// `Java_com_example_RustBindings_sayHello`.
///
/// Then it can be accessed by, for example, Kotlin code as follows:
/// ```kotlin
/// package com.example.RustBindings
///
/// class RustBindings {
///     private external fun sayHello(name: String): String
///
///     fun greetWorld() {
///         println(sayHello("world"))
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn jni_mangle(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    mangle::jni_mangle2(attr.into(), item.into()).into()
}
