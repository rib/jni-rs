mod bind_java_type;
mod mangle;
mod native_method;
mod signature;
mod str;
mod types;
mod utils;

/// Converts UTF-8 string literals to a MUTF-8 encoded `&'static JNIStr`.
///
/// This macro takes one or more literals and encodes them using Java's Modified UTF-8
/// (MUTF-8) format, returning a `&'static JNIStr`.
///
/// Like the `concat!` macro, multiple literals can be provided and will be converted to
/// strings and concatenated before encoding.
///
/// Supported literal types:
/// - String literals (`"..."`)
/// - Character literals (`'c'`)
/// - Integer literals (`42`, `-10`)
/// - Float literals (`3.14`, `1.0`)
/// - Boolean literals (`true`, `false`)
/// - Byte literals (`b'A'` - formatted as numeric value)
/// - C-string literals (`c"..."` - must be valid UTF-8)
///
/// MUTF-8 is Java's variant of UTF-8 that:
/// - Encodes the null character (U+0000) as `0xC0 0x80` instead of `0x00`
/// - Encodes Unicode characters above U+FFFF using CESU-8 (surrogate pairs)
///
/// This is the most type-safe way to create JNI string literals, as it returns a
/// `JNIStr` which is directly compatible with the jni crate's API.
///
/// # Syntax
///
/// ```ignore
/// jni_str!("string literal")
/// jni_str!("part1", "part2", "part3")  // Concatenates before encoding
/// jni_str!("value: ", 42)               // Mix different literal types
/// jni_str!(jni = path::to::jni, "string literal")  // Override jni crate path (must be first)
/// ```
///
/// # Examples
///
/// ```ignore
/// use jni::strings::JNIStr;
///
/// const CLASS_NAME: &JNIStr = jni_str!("java.lang.String");
/// // Result: &'static JNIStr for "java.lang.String" (MUTF-8 encoded)
///
/// const EMOJI_CLASS: &JNIStr = jni_str!("unicode.Type😀");
/// // Result: &'static JNIStr with emoji encoded as surrogate pair
///
/// const PACKAGE_CLASS: &JNIStr = jni_str!("java.lang.", "String");
/// // Result: &'static JNIStr for "java.lang.String" (concatenated then MUTF-8 encoded)
///
/// const PORT: &JNIStr = jni_str!("localhost:", 8080);
/// // Result: &'static JNIStr for "localhost:8080" (mixed literal types concatenated)
/// ```
#[proc_macro]
pub fn jni_str(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    str::jni_str_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Converts UTF-8 string literals to a MUTF-8 encoded CStr literal.
///
/// This macro is equivalent to [`jni_str!`] but returns a `&CStr` instead of a `&'static JNIStr`.
///
/// See the [`jni_str!`] macro documentation for detailed syntax and examples.
#[proc_macro]
pub fn jni_cstr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    str::jni_cstr_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Parses a JNI method or field signature at compile time.
///
/// This macro parses method and field signatures with syntax like `(arg0: JString, arg1: jint) ->
/// JString` and generates a [MethodSignature] or [FieldSignature] struct to represent the
/// corresponding JNI signature, including the raw string like
/// "(Ljava/lang/String;I)Ljava/lang/String;" and enumerated argument plus return types.
///
/// This macro can also parse raw JNI signature strings in order to validate them at compile time
/// but it's recommended to use the structured syntax for better readability.
///
/// [MethodSignature]: https://docs.rs/jni/latest/jni/signature/struct.MethodSignature.html
/// [FieldSignature]: https://docs.rs/jni/latest/jni/signature/struct.FieldSignature.html
///
/// # Syntax
///
/// The macro accepts named properties separated by commas:
/// ```ignore
/// jni_sig!(
///     [jni = <path>],
///     [sig =] <signature>,
///     [type_map = { ... }],
/// )
/// ```
/// The parser automatically detects whether it's a method signature (has parentheses) or a field
/// signature (a single, bare type).
///
/// ## Properties
///
/// - `jni = <path>` - Optionally override the jni crate path (default: auto-detected via
///   `proc_macro_crate`, must come first if given)
/// - `sig = <signature>` - The signature ('`sig =`' prefix is optional for the signature)
/// - `type_map = { RustType => java.lang.ClassName, ... }` - Optional type mappings for Rust types
///
/// The `sig` and `type_map` properties can appear in any order.
///
/// The `type_map` property can be provided multiple times (mappings are merged).
///
/// The design allows for a `macro_rules` wrapper to inject `jni =` or `type_map =` properties,
/// without needing to parse anything else.
///
/// # Type Syntax
///
/// ## Primitive Types
/// - Java primitives: `jboolean`, `jbyte`, `jchar`, `jshort`, `jint`, `jlong`, `jfloat`, `jdouble`
/// - Aliases: `boolean`/`bool`, `byte`/`i8`, `char`, `short`/`i16`, `int`/`i32`, `long`/`i64`,
///   `float`/`f32`, `double`/`f64`
/// - Void: `void` or `()` or elided return type defaults to `void`
///
/// ## Java Object Types
/// - Fully qualified: `java.lang.String`, `java.util.List` or as string literal: `"java.util.List"`
/// - With inner classes: `java.lang.Outer::Inner` or as string literal: `"java.lang.Outer$Inner"`
/// - Default package: `.ClassName` or as string literal: `".ClassName"`
///
/// _(Notice that Java object types _always_ contain at least one `.` dot)_
///
/// ## Rust Reference Types
/// - Single identifier or path: `JString`, `JObject`, `jni::objects::JString`, `RustType`,
///   `custom::RustType`
///
/// ## Array Types
/// - Prefix syntax: `[jint]`, `[[java.lang.String]]`, `[RustType]`
/// - Suffix syntax: `jint[]`, `java.lang.String[][]`, `RustType[]`
///
/// ### Built-in Types
/// - Types like `JObject`, `JClass`, `JString` etc from the `jni` crate can be used without a
///   `type_map`
/// - Built-in types can also be referenced like `jni::objects::JString`
/// - Java types like `java.lang.Class` are automatically mapped to built-in types like `JClass`
///
/// ### Core Types
/// - The core types `java.lang.Object`, `java.lang.Class`, `java.lang.String` and
///   `java.lang.Throwable` can not be mapped to custom types.
/// - Other built-in types, such as `JList` (`java.util.List`) can be overridden by mapping them to
///   a different type via a `type_map`
///
/// ## Type Mappings
/// - Explicit mapping block for Rust types: `type_map = { RustType => java.class.Name }`
/// - Java types without any `type_map` entry will map to `JObject` (`java.lang.Object`)
///
/// # Method Signature Syntax
///
/// A method can be given in one of these forms:
/// - `( [args...] ) -> TYPE`
/// - `( [args...] )`
/// - `"RAW_JNI_SIG"`
///
/// An argument can be given in these forms:
/// - `name: TYPE`
/// - `TYPE`
///
/// _(with a `TYPE` as described in the `Type Syntax` section above)_
///
/// A `TYPE` may have an optional `&` prefix that is ignored
///
/// ```
/// # use jni::{jni_sig, signature::{MethodSignature, JavaType, Primitive}};
/// const JNI_SIG: MethodSignature =
///     jni_sig!((arg1: com.example.Type, arg2: JString, arg3: jint) -> JString);
/// # fn main() {
/// assert!(JNI_SIG.sig().to_bytes() == b"(Lcom/example/Type;Ljava/lang/String;I)Ljava/lang/String;");
/// assert!(JNI_SIG.args().len() == 3);
/// assert!(JNI_SIG.args()[0] == JavaType::Object);
/// assert!(JNI_SIG.args()[1] == JavaType::Object);
/// assert!(JNI_SIG.args()[2] == JavaType::Primitive(Primitive::Int));
/// assert!(JNI_SIG.ret() == JavaType::Object);
/// # }
/// ```
///
/// Traditional JNI signature syntax is also supported:
/// ```ignore
/// jni_sig!("(IILjava/lang/String;)V")
/// ```
///
/// Explicitly named 'sig' property:
/// ```ignore
/// jni_sig!(sig = (arg1: Type1, arg2: Type2, ...) -> ReturnType)
/// ```
///
/// With type mappings:
/// ```
/// # use jni::{jni_sig, signature::{MethodSignature, JavaType, Primitive}};
/// const JNI_SIG: MethodSignature = jni_sig!(
///     type_map = {
///         CustomType => java.class.Type,
///         ReturnType => java.class.ReturnType,
///     },
///     (arg1: CustomType, arg2: JString, arg3: jint) -> ReturnType,
/// );
/// ```
///
/// # Field Signature Syntax
///
/// ```ignore
/// jni_sig!(Type)
/// ```
///
/// Traditional JNI signature syntax is also supported:
/// ```ignore
/// jni_sig!("Ljava/lang/String;")
/// ```
///
/// Named:
/// ```ignore
/// jni_sig!(sig = Type)
/// ```
///
/// With type mappings:
/// ```ignore
/// jni_sig!(
///     Type,
///     type_map = {
///         RustType as java.class.Name,
///         ...
///     }
/// )
/// ```
///
/// # Examples
///
/// ## Method Signatures
///
/// Basic primitive types:
/// ```ignore
/// const SIG: MethodSignature = jni_sig!((a: jint, b: jboolean) -> void);
/// // Result: MethodSignature for "(IZ)V"
/// ```
///
/// Java object types:
/// ```ignore
/// const SIG: MethodSignature = jni_sig!(
///     (a: jint, b: java.lang.String) -> java.lang.Object
/// );
/// // Result: MethodSignature for "(ILjava/lang/String;)Ljava/lang/Object;"
/// ```
///
/// Array types:
/// ```ignore
/// const SIG: MethodSignature = jni_sig!(
///     (a: [jint], b: [java.lang.String]) -> [[jint]]
/// );
/// // Result: MethodSignature for "([I[Ljava/lang/String;)[[I"
/// ```
///
/// With type mappings:
/// ```ignore
/// const SIG: MethodSignature = jni_sig!(
///     (a: jint, b: MyString, c: [MyObject]) -> MyThrowable,
///     type_map = {
///         MyString as java.lang.String,
///         MyObject as java.lang.Object,
///         MyThrowable as java.lang.Throwable,
///     }
/// );
/// // Result: MethodSignature for "(ILjava/lang/String;[Ljava/lang/Object;)Ljava/lang/Throwable;"
/// ```
/// Multiple type_maps:
/// ```ignore
/// const SIG: MethodSignature = jni_sig!(
///     jni = ::my_jni,
///     type_map = { MyType0 => custom.Type0 },
///     type_map = { MyType1 => custom.Type1 },
///     sig = (arg0: MyType0, arg1: MyType1) -> JString,
/// );
/// ```
///
/// This makes it possible to write wrapper macros to inject a `type_map` without blocking the use
/// of `type_map` for additional types.
///
/// With named signature property:
/// ```ignore
/// const SIG: MethodSignature = jni_sig!(
///     sig = (a: jint) -> void,
///     type_map = { MyType => java.lang.MyType }
/// );
/// ```
///
/// With custom jni crate path:
/// ```ignore
/// const SIG: MethodSignature = jni_sig!(
///     jni = ::my_jni, // must come first!
///     (a: jint) -> void,
/// );
/// ```
///
/// ## Field Signatures
///
/// Primitive field:
/// ```ignore
/// const SIG: FieldSignature = jni_sig!(jint);
/// // Result: FieldSignature for "I"
/// ```
///
/// Object field:
/// ```ignore
/// const SIG: FieldSignature = jni_sig!(java.lang.String);
/// // Result: FieldSignature for "Ljava/lang/String;"
/// ```
///
/// Array field:
/// ```ignore
/// const SIG: FieldSignature = jni_sig!([jint]);
/// // Result: FieldSignature for "[I"
/// ```
///
/// Field with type mapping:
/// ```ignore
/// const SIG: FieldSignature = jni_sig!(
///     MyType,
///     type_map = {
///         MyType as custom.Type,
///     }
/// );
/// // Result: FieldSignature for "Lcustom/Type;"
/// ```
#[proc_macro]
pub fn jni_sig(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    signature::jni_sig_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Parses a JNI method or field signature at compile time and returns a `&str` literal.
///
/// This macro is similar to `jni_sig!` but returns a plain UTF-8 string literal instead
/// of a `MethodSignature` or `FieldSignature` struct.
///
/// See the `jni_sig!` macro documentation for detailed syntax and examples.
///
/// # Examples
///
/// ```ignore
/// const SIG: &str = jni_sig_str!((a: jint, b: jboolean) -> void);
/// // Result: "(IZ)V"
///
/// const FIELD_SIG: &str = jni_sig_str!(java.lang.String);
/// // Result: "Ljava/lang/String;"
/// ```
#[proc_macro]
pub fn jni_sig_str(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    signature::jni_sig_str_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Parses a JNI method or field signature at compile time and returns a CStr literal.
///
/// This macro is similar to `jni_sig!` but returns a C string literal (e.g., `c"(IZ)V"`)
/// with MUTF-8 encoding instead of a `MethodSignature` or `FieldSignature` struct.
///
/// The output is encoded using Java's modified UTF-8 (MUTF-8) format via `cesu8::to_java_cesu8`.
///
/// See the `jni_sig!` macro documentation for detailed syntax and examples.
///
/// # Examples
///
/// ```ignore
/// const SIG: &CStr = jni_sig_cstr!((a: jint, b: jboolean) -> void);
/// // Result: c"(IZ)V" (MUTF-8 encoded)
///
/// const FIELD_SIG: &CStr = jni_sig_cstr!(java.lang.String);
/// // Result: c"Ljava/lang/String;" (MUTF-8 encoded)
/// ```
#[proc_macro]
pub fn jni_sig_cstr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    signature::jni_sig_cstr_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Parses a JNI method or field signature at compile time and returns a `&'static JNIStr`.
///
/// This macro is similar to `jni_sig!` but returns a `&'static JNIStr` with MUTF-8 encoding
/// instead of a `MethodSignature` or `FieldSignature` struct.
///
/// The output is encoded using Java's modified UTF-8 (MUTF-8) format via `cesu8::to_java_cesu8`
/// and wrapped in a `JNIStr` via `jni::strings::JNIStr::from_cstr_unchecked()`.
///
/// See the `jni_sig!` macro documentation for detailed syntax and examples.
///
/// # Examples
///
/// ```ignore
/// const SIG: &JNIStr = jni_sig_jstr!((a: jint, b: jboolean) -> void);
/// // Result: &'static JNIStr for "(IZ)V" (MUTF-8 encoded)
///
/// const FIELD_SIG: &JNIStr = jni_sig_jstr!(java.lang.String);
/// // Result: &'static JNIStr for "Ljava/lang/String;" (MUTF-8 encoded)
/// ```
#[proc_macro]
pub fn jni_sig_jstr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    signature::jni_sig_jstr_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Export a Rust function with a JNI-compatible, mangled method name.
///
/// This adds an appropriate `#[export_name = "..."]` attribute and `extern
/// "system"` ABI to the function, to allow it to be resolved by a JVM when
/// calling an associated native method.
///
/// This attribute takes one to three string literal arguments:
/// 1. Package namespace (required)
/// 2. Method name (optional)
/// 3. JNI signature (optional)
///
/// If two arguments are given, the second is inferred to be a method name if it
/// doesn't contain '(', otherwise it's treated as a signature.
///
/// The name is mangled according to the JNI Specification, under "Design" ->
/// "Resolving Native Method Names"
///
/// <https://docs.oracle.com/en/java/javase/11/docs/specs/jni/design.html#resolving-native-method-names>
///
/// ## Method Name Generation
///
/// If no method name is provided, the Rust function name is converted from
/// `snake_case` to `lowerCamelCase`.
///
/// If the Rust function name is not entirely lowercase with underscores (i.e.
/// it contains any uppercase letters), the name is used directly without
/// transformation.
///
/// ## `snake_case` to `lowerCamelCase` Conversion Rules
///
/// If the input contains any uppercase letters, it's returned unchanged to
/// preserve intentional casing.
///
/// Leading underscores are preserved except for one underscore that is removed.
///
/// Trailing underscores are preserved.
///
/// When capitalizing segments after underscores, the first non-numeric
/// character is capitalized. This ensures that segments with numeric prefixes
/// are properly capitalized.
///
/// Examples:
/// - `"say_hello"` -> `"sayHello"`
/// - `"get_user_name"` -> `"getUserName"`
/// - `"_private_method"` -> `"privateMethod"` (one leading underscore removed)
/// - `"__dunder__"` -> `"_dunder__"` (one leading underscore removed)
/// - `"___priv"` -> `"__priv"` (one leading underscore removed)
/// - `"trailing_"` -> `"trailing_"`
/// - `"sayHello"` -> `"sayHello"` (unchanged)
/// - `"getUserName"` -> `"getUserName"` (unchanged)
/// - `"Foo_Bar"` -> `"Foo_Bar"` (unchanged - contains uppercase)
/// - `"XMLParser"` -> `"XMLParser"` (unchanged - contains uppercase)
/// - `"init"` -> `"init"` (unchanged - no underscores)
/// - `"test_αλφα"` -> `"testΑλφα"` (Unicode-aware)
/// - `"array_2d_foo"` -> `"array2DFoo"` (capitalizes first char after digits)
/// - `"test_3d"` -> `"test3D"` (capitalizes first char after digits)
///
/// # `snake_case` to `lowerCamelCase` Conversion Rules
///
/// If the input contains any uppercase letters, it's returned unchanged to preserve intentional casing.
///
/// Leading underscores are preserved except for one underscore that is removed.
///
/// Trailing underscores are preserved.
///
/// When capitalizing segments after underscores, the first non-numeric character is capitalized.
/// This ensures that segments with numeric prefixes are properly capitalized.
///
/// Examples:
/// - "say_hello" -> "sayHello"
/// - "get_user_name" -> "getUserName"
/// - "_private_method" -> "privateMethod" (one leading underscore removed)
/// - "__dunder__" -> "_dunder__" (one leading underscore removed)
/// - "___priv" -> "__priv" (one leading underscore removed)
/// - "trailing_" -> "trailing_"
/// - "sayHello" -> "sayHello" (unchanged)
/// - "getUserName" -> "getUserName" (unchanged)
/// - "Foo_Bar" -> "Foo_Bar" (unchanged - contains uppercase)
/// - "XMLParser" -> "XMLParser" (unchanged - contains uppercase)
/// - "init" -> "init" (unchanged - no underscores)
/// - "test_αλφα" -> "testΑλφα" (Unicode-aware)
/// - "array_2d_foo" -> "array2DFoo" (capitalizes first char after digits)
/// - "test_3d" -> "test3D" (capitalizes first char after digits)
///
/// ## ABI Handling
///
/// The macro requires the ABI to be `extern "system"` (required for JNI).
/// - If no ABI is specified, it will automatically be set to `extern "system"`
/// - If `extern "system"` is already specified, it will be preserved
/// - If any other ABI (e.g., `extern "C"`) is specified, a compile error will
///   be generated
///
/// ## Examples
///
/// Basic usage with just namespace (function name converted to lowerCamelCase):
/// ```
/// # use jni::{ EnvUnowned, objects::{ JObject, JString } };
/// # use jni_macros::jni_mangle;
///
/// // Rust function in snake_case
/// #[jni_mangle("com.example.RustBindings")]
/// pub fn say_hello<'local>(mut env: EnvUnowned<'local>, _: JObject<'local>, name: JString<'local>) -> JString<'local> {
///     // ...
/// #     unimplemented!()
/// }
/// // Generates: Java_com_example_RustBindings_sayHello
/// ```
///
/// Or already in lowerCamelCase (idempotent):
/// ```
/// # use jni::{ EnvUnowned, objects::{ JObject, JString } };
/// # use jni_macros::jni_mangle;
/// #[allow(non_snake_case)]
/// #[jni_mangle("com.example.RustBindings")]
/// pub fn sayHello<'local>(mut env: EnvUnowned<'local>, _: JObject<'local>, name: JString<'local>) -> JString<'local> {
///     // ...
/// #     unimplemented!()
/// }
/// // Generates: Java_com_example_RustBindings_sayHello
/// ```
///
/// The `sayHello` function will automatically be expanded to have the correct
/// ABI specification and the appropriate JNI-compatible name, i.e. in this case
/// - `Java_com_example_RustBindings_sayHello`.
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
///
/// With custom method name:
/// ```
/// # use jni::{ EnvUnowned, objects::JObject };
/// # use jni_macros::jni_mangle;
/// #[jni_mangle("com.example.RustBindings", "customMethodName")]
/// pub fn some_rust_function<'local>(env: EnvUnowned<'local>, _: JObject<'local>) { }
/// // Generates: Java_com_example_RustBindings_customMethodName
/// ```
///
/// With signature only (overloaded method):
/// ```
/// # use jni::{ EnvUnowned, objects::JObject };
/// # use jni_macros::jni_mangle;
/// #[jni_mangle("com.example.RustBindings", "(I)Z")]
/// pub fn boolean_method<'local>(env: EnvUnowned<'local>, _: JObject<'local>) { }
/// // Generates: Java_com_example_RustBindings_booleanMethod__I
/// // Note: Only argument types are encoded (I), return type (Z) is ignored
/// ```
///
/// With method name and signature:
/// ```
/// # use jni::{ EnvUnowned, objects::JObject };
/// # use jni_macros::jni_mangle;
/// #[jni_mangle("com.example.RustBindings", "customName", "(Ljava/lang/String;)V")]
/// pub fn another_function<'local>(env: EnvUnowned<'local>, _: JObject<'local>) { }
/// // Generates: Java_com_example_RustBindings_customName__Ljava_lang_String_2
/// // Note: Only argument types are encoded, return type (V) is ignored
/// ```
///
/// Pre-existing "system" ABI is preserved:
/// ```
/// # use jni::{ EnvUnowned, objects::JObject };
/// # use jni_macros::jni_mangle;
/// #[jni_mangle("com.example.RustBindings")]
/// pub extern "system" fn my_function<'local>(env: EnvUnowned<'local>, _: JObject<'local>) { }
/// // The ABI will be set to "system" but you can also set it explicitly
/// ```
#[proc_macro_attribute]
pub fn jni_mangle(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    mangle::jni_mangle2(attr.into(), item.into()).into()
}

/// Creates a compile-time type-checked `NativeMethod` descriptor for a native method.
///
/// This macro generates a `NativeMethod` struct with a compile-time guarantee that the
/// provided function pointer matches the JNI signature.
///
/// By default the native method implementation is automatically wrapped with a
/// call to `EnvUnowned::with_env` and any returned `Result` is unwrapped with
/// an `ErrorPolicy` (default `ThrowRuntimeExAndDefault`).
///
/// It also optionally generates a JNI export symbol for the method.
///
/// # Syntax
///
/// ## Property-based syntax
///
/// ```ignore
/// native_method! {
///     [jni = <path>,]          // Override jni crate path (default: auto-detected, must come first)
///     [rust_type = <Type>,]    // Type for 'this' parameter (default: JObject)
///     [java_type = <Type>,]    // Fully-qualified Java class name (required if export = true)
///     [name = "<methodName>",] // Java method name
///     [sig = (args) -> ret,]   // JNI signature (see `jni_sig!` macro for syntax)
///     [static = true,]         // Indicates static method with a `class` parameter instead of `this`
///     [export = true,]         // Generate mangled JNI export symbol like `Java_package_Class_method` that JVM can resolve (requires `java_type`)
///     [fn = <function_path>,]  // Path to Rust function
///     [type_map = { ... },]    // Type mappings for custom types
///
///     // Combine with shorthand syntax (see below):
///     [static] [raw] [extern] fn Type::method_name(args) -> ret,
/// }
/// ```
///
/// # Properties
///
/// - `jni` - Optional override for the jni crate path (must come first if provided)
/// - `rust_type` - Optional type for the `this` parameter (e.g., `MyType`). If omitted, uses `JObject`
/// - `java_type` - Fully-qualified Java class name, required in combination with `export = true` / `extern` native methods
/// - `name` - The Java method name as a string literal
/// - `sig` - The method signature (see [`jni_sig!`] macro for syntax)
/// - `fn` - Path to the Rust function that implements this native method (defaults to `RustType::method_name` if shorthand syntax is used)
/// - `type_map` - Optional type mappings from Rust types to Java class names
///
/// ## Shorthand syntax
///
/// ```ignore
/// native_method! {
///     [jni = <path>,]          // Optional: Override jni crate path (must come first)
///     [static] [raw] [extern] fn Type::method_name(args) -> ret
/// }
/// ```
///
/// The shorthand syntax:
/// - Uses `Type` as the `rust_type` parameter
/// - Converts `method_name` from snake_case to lowerCamelCase for the Java method `name`
/// - Uses `RustType::method_name` as the `fn` path unless overridden
/// - `static` indicates a static method (emits a `class` parameter instead of `this`)
/// - `raw` indicates the function receives a raw `EnvUnowned` instead of `&mut Env`, with no `catch_unwind` wrapper and does not return a `Result`
/// - `extern` says that the function should have a JNI mangled export symbol generated (`java_type` must also be provided)
///
/// # Non-raw Function Signature Requirements
///
/// If `raw` is not specified, the function must return a `Result` and accept a mutable `Env` reference.
///
/// Non-static, instance method signature:
///
/// ```ignore
/// fn<'local>(
///     env: &mut Env<'local>,
///     this: ThisType<'local>,  // Or JObject<'local> if 'for' is not specified
///     param1: Type1,
///     param2: Type2,
///     ...
/// ) -> Result<ReturnType, E>
/// where
///     E: Into<jni::errors::Error>,
/// ```
///
/// Static method signature:
///
/// ```ignore
/// fn<'local>(
///     env: &mut Env<'local>,
///     class: JClass<'local>,
///     param1: Type1,
///     param2: Type2,
///     ...
/// ) -> Result<ReturnType, E>
/// where
///     E: Into<jni::errors::Error>,
/// ```
///
/// Where:
/// - `Env<'local>` represents the JNI environment attached by the JVM
/// - `ThisType<'local>` is the type specified in the `rust_type` property, or `JObject<'local>`
/// - Parameter types must match the JNI signature types
/// - `ReturnType` must match the JNI signature return type
///
/// # Raw Function Signature Requirements
///
/// If `raw` is specified, the function must accept an `EnvUnowned` parameter and return the exact type specified in the JNI signature.
///
/// Note that in this case there is no `catch_unwind` wrapper and no automatic error handling.
///
/// Non-static, instance method signature:
///
/// ```ignore
/// fn<'local>(
///     unowned_env: EnvUnowned<'local>,
///     this: ThisType<'local>,  // Or JObject<'local> if 'for' is not specified
///     param1: Type1,
///     param2: Type2,
///     ...
/// ) -> ReturnType
/// ```
///
/// Static method signature:
///
/// ```ignore
/// fn<'local>(
///     unowned_env: EnvUnowned<'local>,
///     class: JClass<'local>,
///     param1: Type1,
///     param2: Type2,
///     ...
/// ) -> ReturnType
/// ```
///
/// Where:
/// - `EnvUnowned<'local>` is the JNI environment attached by the JVM
/// - `ThisType<'local>` is the type specified in the `rust_type` property, or `JObject<'local>`
/// - Parameter types must match the JNI signature types
/// - `ReturnType` must match the JNI signature return type
///
/// # Examples
///
/// ## Basic Usage
///
/// TODO
///
/// # Type Safety
///
/// The macro generates a wrapper function with the exact signature required by JNI,
/// and then calls your implementation function through it. This ensures:
///
/// - The function pointer passed to `NativeMethod::from_raw_parts` has the correct ABI
/// - Parameter types match the JNI signature
/// - Return type matches the JNI signature
/// - Any type mismatch results in a compile-time error
///
/// **Note:** This macro can not automatically check whether the native method is
/// `static` or not and so it's important that the `static` property is set correctly
/// to ensure the type for the second parameter (`this` vs `class`) is correct.
///
/// # See Also
///
/// - [`NativeMethod`](https://docs.rs/jni/latest/jni/struct.NativeMethod.html) - The struct created by this macro
#[proc_macro]
pub fn native_method(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    native_method::native_method_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Binds a Java type to a Rust type with methods, fields, and native methods.
///
/// This macro generates a complete [Reference] type binding for a Java class,
/// including:
/// - API struct for caching a class reference with method/field IDs
/// - Reference trait implementation
/// - Constructor, method, and field bindings
/// - Safe native methods binding trait
/// - Native method registration and exports
/// - Type aliases
///
/// The generated bindings include self-check assertions to ensure that declared
/// casts and type mappings are valid at runtime.
///
/// [Reference]: https://docs.rs/jni/latest/jni/refs/trait.Reference.html
///
/// # Basic Example
///
/// ```
/// # use jni::Env;
/// # use jni::objects::{JClass, JString};
/// # use jni::sys::jint;
/// # use jni::refs::LoaderContext;
/// use jni::bind_java_type;
///
/// // Minimal shorthand syntax for trivial bindings
/// bind_java_type! { CustomType => com.example.CustomClass }
///
/// // Full example with all common properties
/// bind_java_type! {
///     rust_type = MyType,
///     java_type = "com.example.MyClass",
///     type_map = {
///         CustomType => com.example.CustomClass,
///     },
///     is_instance_of = {
///        collection: JCollection,
///     },
///     constructors = {
///         fn new(),
///         fn new_with_value(value: jint),
///     },
///     methods = {
///         fn my_method(arg: jint) -> CustomType,
///         priv fn _my_private_method() -> jint,
///         static fn my_static_method() -> jint,
///     },
///     fields = {
///         my_field: jint,
///         static my_static_field: jint,
///     },
///     native_methods = {
///         fn my_native(val: jint) -> JString,
///     }
/// }
///
/// // Implement the native methods trait
/// impl MyTypeNativeInterface for MyTypeAPI {
///    type Error = jni::errors::Error;
///    fn my_native<'local>(
///        env: &mut Env<'local>,
///        this: MyType<'local>,
///        val: jint
///    ) -> Result<JString<'local>, Self::Error> {
///        JString::from_str(env, &format!("Value: {}", val))
///    }
/// }
///
/// // Use the generated bindings
/// fn use_my_type<'local>(env: &mut Env<'local>) -> jni::errors::Result<()> {
///     let my_obj = MyType::new_with_value(env, 42)?;
///     let result = my_obj.my_method(env, 100)?;
///     let field_value = my_obj.my_field(env)?;
///     my_obj.set_my_field(env, 200)?;
///     Ok(())
/// }
/// ```
///
/// # Generated Code
///
/// The macro generates:
/// - `struct MyType<'local>` - Reference wrapper type
/// - `struct MyTypeAPI` - Singleton API struct with cached class and
///   method/field IDs
/// - `impl Reference for MyType<'local>` - Reference trait implementation
/// - `trait MyTypeNativeInterface` - Safe trait for implementing native methods
/// - `impl From<MyType<'local>> for JObject<'local>` - Conversion to JObject
/// - `impl From<MyType<'local>> for <IsInstanceOf>` - Conversions for declared
///   parent types
///
/// The `MyTypeAPI::get(&Env, &LoaderContext)` method:
/// - Lazily loads and caches the Java class reference
/// - Caches all method IDs and field IDs
/// - Validates type mappings and `is_instance_of` relationships at runtime
/// - Registers native methods with the JVM
///
/// # Automatic Name Conversion: `snake_case` to `lowerCamelCase`
///
/// When no explicit Java method name is provided, for methods or fields, Rust
/// names are automatically converted from `snake_case` to `lowerCamelCase` with
/// the following rules:
///
/// - Names with uppercase letters are preserved as-is (e.g., `My_Method`,
///   `MY_METHOD`)
/// - One leading underscore is removed (e.g., `_my_method` → `myMethod`) (This
///   allows private bindings like `_my_method` to map to `myMethod`, leaving
///   the `my_method` name available for public wrapper methods)
/// - When capitalizing after underscores, the first non-digit character is
///   capitalized (e.g., `my_2d_api` → `my2DApi`, `array_3d` → `array3D`)
///
/// Examples:
/// - `get_user_name` → `getUserName`
/// - `_my_private` → `myPrivate`
/// - `my_2d_array` → `my2DArray`
/// - `MY_CONSTANT` → `MY_CONSTANT` (unchanged)
/// - `myMethod` → `myMethod` (unchanged)
///
/// # Properties Reference
///
/// ## `rust_type` (required)
///
/// The Rust type name for the generated Reference wrapper.
///
/// ```ignore
/// rust_type = MyType
/// ```
///
/// ## `java_type` (required)
///
/// The fully-qualified Java class name. Can be specified as:
/// - Dot-separated identifiers: `java.lang.String` or `com.example.MyClass`
/// - String literal: `"java.lang.String"` or `"com.example.MyClass"`
///
/// For inner classes:
/// - Use `::` in dot-separated form: `com.example.Outer::Inner`
/// - Use `$` in string form: `"com.example.Outer$Inner"`
///
/// ```ignore
/// java_type = com.example.MyClass
/// // or
/// java_type = "com.example.Outer$Inner"
/// ```
///
/// ## `type_map`
///
/// Maps Rust type names to Java class names for use in method/field signatures.
/// This allows using custom Rust types in signatures throughout the binding.
///
/// Supports three types of mappings:
///
/// ### Reference Type Mappings
///
/// Maps Rust reference types to Java classes:
///
/// ```ignore
/// type_map = {
///     CustomType => com.example.CustomClass,
///     AnotherType => "com.example.AnotherClass",
///     InnerType => com.example.Outer::Inner,
///     my_crate::MyType => com.example.MyType,
/// }
/// ```
///
/// ### Unsafe Primitive Type Mappings
///
/// Maps Rust types to Java primitive types using the `unsafe` keyword. This is
/// particularly useful for Rust types that transparently wrap a pointer (e.g.,
/// handles) that need to be passed to Java as a `long`:
///
/// ```ignore
/// type_map = {
///     unsafe MyHandle => long,
///     unsafe MyBoxedPointer => long,
///     unsafe MyRawFd => int,
/// }
/// ```
///
/// These mappings are marked `unsafe` because the macro cannot verify type
/// safety between the Rust type and Java primitive type.
///
/// ### Type Aliases
///
/// Creates aliases for existing type mappings using the `typealias` keyword.
/// This can improve readability in signatures before defining full type
/// bindings:
///
/// ```ignore
/// type_map = {
///     MyType => com.example.MyType,
///     typealias MyAlias => MyType,
///     typealias MyObjectAlias => JObject,
/// }
/// ```
///
/// Note: Aliases for array types are not supported.
///
/// ## `is_instance_of`
///
/// Declares that this type can be safely cast to other Reference types. The
/// macro generates `From` implementations and runtime validation to ensure the
/// relationships are valid.
///
/// Supports two syntaxes:
/// - With explicit stem name: `stem: Type` (generates `as_stem()` method and `From` traits)
/// - Without stem: `Type` (generates only `From` traits, no `as_` method)
///
/// ```ignore
/// is_instance_of = {
///     base: BaseClass,        // as_base() -> BaseClass (+ From traits)
///     collection: JCollection, // as_collection() -> JCollection (+ From traits)
///     JThrowable,             // From traits only, no as_ method
/// }
/// ```
///
/// ## All Method Blocks: `constructors`, `methods`, `native_methods`
///
/// These blocks define constructor, method, and native method bindings using
/// either shorthand or block syntax. See the [`jni_sig!`] macro for signature
/// syntax details.
///
/// ### Shorthand Syntax
///
/// ```custom
/// [visibility] [static] [raw] [extern] fn name(params) -> return_type
/// ```
///
/// - `visibility`: Optional visibility modifier (`pub`, `priv`, `pub(crate)`,
///   etc.) (not applicable to `native_methods`)
/// - `static`: Marks method as static (applies to `methods` and
///   `native_methods`)
/// - `raw`: For `native_methods` only - function receives `EnvUnowned`
///   directly, with no `catch_unwind` wrapper or `Result` error mapping
/// - `extern`: For `native_methods` only - generates JNI export symbol
///
/// ### Block Syntax
///
/// ```ignore
/// [visibility] [static] [raw] [extern] fn name {
///     [name = "javaMethodName",]
///     sig = (params) -> return_type,
///     [error_policy = ErrorPolicy,]  // native_methods only
///     [export = true | false | "CustomName",]  // native_methods only
///     [fn = function_path,]  // native_methods only
/// }
/// ```
///
/// The leading qualifiers (`[visibility]`, `[static]`, `[raw]`, `[extern]`) are
/// the same as for shorthand syntax.
///
/// Block syntax properties:
///
/// - `name`: *(methods and native_methods only)* Optional custom Java method
///   name (string literal). If omitted, uses automatic name conversion. Not
///   applicable to constructors (which always use `<init>`).
/// - `sig`: Method signature (required). See [`jni_sig!`] macro for syntax
///   details.
/// - `error_policy`: *(native_methods only)* Custom error handling policy
///   (e.g., `jni::errors::LogErrorAndDefault`). Controls how `Result` errors
///   are converted to JNI exceptions or default values.
/// - `export`: *(native_methods only)* Controls JNI export symbol generation:
///   - `true`: Generate auto-mangled JNI export name
///   - `false`: Don't generate export (override global `export_native_methods`)
///   - `"CustomName"`: Use custom export name (string literal)
/// - `fn`: *(native_methods only)* Path to function implementing this native
///   method. When specified, bypasses the trait implementation and directly
///   uses the provided function. The expected function signature depends on
///   whether `raw` is used:
///   - Without `raw`: Function receives `&mut Env` and returns `Result<T, E>`
///   - With `raw`: Function receives `EnvUnowned` and returns the value
///     directly
///
/// ## `constructors`
///
/// ```ignore
/// constructors = {
///     // Shorthand: no-arg constructor
///     fn new(),
///
///     // Shorthand: constructor with parameters
///     fn with_value(value: jint),
///
///     // Block syntax (no name property - constructors are always named "<init>")
///     fn with_string {
///         sig = (value: java.lang.String) -> void,
///     },
/// }
/// ```
///
/// ## `methods`
///
/// ```ignore
/// methods = {
///     // Shorthand instance method
///     fn get_value() -> jint,
///
///     // Shorthand static method
///     static fn get_default() -> MyType,
///
///     // Private visibility
///     priv fn internal_helper() -> jint,
///
///     // Block syntax with custom name
///     fn get_user_info {
///         name = "getUserDetails",
///         sig = () -> java.lang.String,
///     },
///
///     // Static method with block syntax
///     static fn create_instance {
///         sig = (name: java.lang.String) -> MyType,
///     },
/// }
/// ```
///
/// ## `native_methods`
///
/// Native methods can be implemented in two ways: via a trait implementation
/// (default) or by directly providing a function with the `fn` property.
///
/// #### Default: Trait Implementation with Automatic Wrapping
///
/// By default, trait implementations receive `&mut Env` and return `Result<T,
/// E>`. The macro automatically wraps these implementations with:
///
/// 1. **Panic safety**: `catch_unwind` via `EnvUnowned::with_env`
/// 2. **Error handling**: `ErrorPolicy` to convert `Result` to a value
///
/// The generated wrapper effectively does this:
///
/// ```ignore
/// fn _generated_wrapper<'local>(
///     mut unowned_env: EnvUnowned<'local>,
///     this: MyType<'local>,
///     a: jint,
///     b: jint,
/// ) -> jint {
///     let outcome = unowned_env.with_env(|env| -> jni::errors::Result<_> {
///         // Your trait implementation is called here
///         <MyTypeAPI as MyTypeNativeInterface>::native_add(env, this, a, b)
///     });
///     // Convert Result to value using error policy (default: ThrowRuntimeExAndDefault)
///     outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
/// }
/// ```
///
/// **Builtin Error Policies:**
/// - `ThrowRuntimeExAndDefault` - Throws `RuntimeException` and returns default
///   value (default)
/// - `LogErrorAndDefault` - Logs error and returns default value
///
/// You can specify a custom error policy per method:
///
/// ```ignore
/// native_methods = {
///     fn native_risky {
///         sig = (value: jint) -> jint,
///         error_policy = jni::errors::LogErrorAndDefault,
///     },
/// }
/// ```
///
/// #### Raw Native Methods: Direct Implementation
///
/// The `raw` qualifier bypasses the automatic wrapping. Raw methods:
/// - Receive `EnvUnowned` directly (not `&mut Env`)
/// - Return values directly (not `Result`)
/// - Have **no** `catch_unwind` wrapper
/// - Have **no** automatic error handling
///
/// Raw methods can still be implemented via the trait or with `fn = function`:
///
/// ```ignore
/// native_methods = {
///     // Raw method via trait
///     raw fn native_raw_trait(value: jint) -> jint,
///
///     // Raw method with direct function
///     raw fn native_raw_direct {
///         sig = (value: jint) -> jint,
///         fn = my_raw_function,
///     },
/// }
///
/// // Trait implementation for raw method
/// impl MyTypeNativeInterface for MyTypeAPI {
///     fn native_raw_trait<'local>(
///         env: EnvUnowned<'local>,
///         this: MyType<'local>,
///         value: jint,
///     ) -> jint {
///         value * 2
///     }
/// }
///
/// // Direct function for raw method
/// fn my_raw_function<'local>(
///     env: EnvUnowned<'local>,
///     this: MyType<'local>,
///     value: jint,
/// ) -> jint {
///     value * 3
/// }
/// ```
///
/// #### Direct Function Implementation: Bypassing the Trait
///
/// The `fn` property allows you to bypass the trait entirely and provide a
/// direct function implementation. This works with both normal and raw methods:
///
/// ```ignore
/// native_methods = {
///     // Non-raw with direct function (still gets wrapped with catch_unwind)
///     fn native_with_function {
///         sig = (value: jint) -> jint,
///         fn = my_safe_function,
///     },
///
///     // Raw with direct function (no wrapping)
///     raw fn native_raw_function {
///         sig = (value: jint) -> jint,
///         fn = my_raw_function,
///     },
/// }
///
/// // Non-raw function (returns Result, gets automatic error handling)
/// fn my_safe_function<'local>(
///     env: &mut Env<'local>,
///     this: MyType<'local>,
///     value: jint,
/// ) -> Result<jint, jni::errors::Error> {
///     Ok(value * 2)
/// }
///
/// // Raw function (no Result, no catch_unwind wrapping)
/// fn my_raw_function<'local>(
///     env: EnvUnowned<'local>,
///     this: MyType<'local>,
///     value: jint,
/// ) -> jint {
///     value * 3
/// }
/// ```
///
/// #### Complete Example
///
/// ```ignore
/// native_methods = {
///     // Shorthand: instance method via trait
///     fn native_add(a: jint, b: jint) -> jint,
///
///     // Static method via trait
///     static fn native_initialize() -> jboolean,
///
///     // Export with auto-mangled JNI name
///     extern fn native_exported(value: jint) -> jint,
///
///     // Custom error handling policy
///     fn native_risky {
///         sig = (value: jint) -> jint,
///         error_policy = jni::errors::LogErrorAndDefault,
///     },
///
///     // Direct function implementation (bypasses trait)
///     fn native_direct {
///         sig = (value: jint) -> jint,
///         fn = my_implementation,
///     },
///
///     // Raw method via trait
///     raw fn native_raw_trait(value: jint) -> jint,
///
///     // Raw method with direct function
///     raw fn native_raw_direct {
///         sig = (value: jint) -> jint,
///         fn = my_raw_function,
///     },
///
///     // Control export behavior
///     fn native_not_exported {
///         sig = (value: jint) -> jint,
///         export = false,
///     },
/// }
/// ```
///
/// Implement the trait for methods without `fn` property:
///
/// ```ignore
/// impl MyTypeNativeInterface for MyTypeAPI {
///     type Error = jni::errors::Error;
///
///     fn native_add<'local>(
///         env: &mut Env<'local>,
///         this: MyType<'local>,
///         a: jint,
///         b: jint,
///     ) -> Result<jint, Self::Error> {
///         Ok(a + b)
///     }
///
///     fn native_initialize<'local>(
///         env: &mut Env<'local>,
///         class: JClass<'local>,
///     ) -> Result<jboolean, Self::Error> {
///         Ok(true)
///     }
///
///     fn native_exported<'local>(
///         env: &mut Env<'local>,
///         this: MyType<'local>,
///         value: jint,
///     ) -> Result<jint, Self::Error> {
///         Ok(value + 1)
///     }
///
///     fn native_risky<'local>(
///         env: &mut Env<'local>,
///         this: MyType<'local>,
///         value: jint,
///     ) -> Result<jint, Self::Error> {
///         if value < 0 {
///             Err(jni::errors::Error::JniCall(jni::errors::JniError::Unknown))
///         } else {
///             Ok(value * 2)
///         }
///     }
///
///     fn native_raw_trait<'local>(
///         env: EnvUnowned<'local>,
///         this: MyType<'local>,
///         value: jint,
///     ) -> jint {
///         value * 2
///     }
///
///     fn native_not_exported<'local>(
///         env: &mut Env<'local>,
///         this: MyType<'local>,
///         value: jint,
///     ) -> Result<jint, Self::Error> {
///         Ok(value + 10)
///     }
/// }
/// ```
///
/// #### Native Method Exporting
///
/// By default, all native methods are exported with JNI mangled names so that
/// the JVM can discover them. This can be disabled globally by setting the
/// `export_native_methods` property to `false` and then enabled per-method with
/// the `extern` qualifier or `export` property.
///
/// Exporting is independent of `raw` and `fn` - you can export any native method.
///
/// When exporting is enabled (via `extern`, `export = true`, or the default
/// `export_native_methods = true`), the macro generates an additional wrapper
/// function with the proper JNI mangled name and `extern "system"` ABI:
///
/// ```ignore
/// #[unsafe(no_mangle)]
/// #[allow(non_snake_case)]
/// pub unsafe extern "system" fn Java_com_example_MyType_myMethod__I<'local>(
///     mut unowned_env: ::jni::EnvUnowned<'local>,
///     this: MyType<'local>,
///     value: ::jni::sys::jint,
/// ) -> ::jni::sys::jint {
///     // Calls the internal wrapper (which may or may not have catch_unwind/error handling)
///     MyTypeAPI::my_method_native_method(unowned_env, this, value)
/// }
/// ```
///
/// This export wrapper allows the JVM to discover and call the native method
/// using standard JNI name resolution. The export wrapper simply forwards to
/// the internal implementation, which may be:
/// - A trait method with automatic wrapping (default)
/// - A raw trait method without wrapping
/// - A direct function with `fn = function`
///
/// Control export behavior:
///
/// ```ignore
/// bind_java_type! {
///     rust_type = MyType,
///     java_type = "com.example.MyType",
///     export_native_methods = false,  // Disable exports by default
///     native_methods = {
///         // Not exported (global default)
///         fn method_one(value: jint) -> jint,
///
///         // Explicitly exported with auto-mangled name
///         extern fn method_two(value: jint) -> jint,
///
///         // Explicitly exported with custom name
///         fn method_three {
///             sig = (value: jint) -> jint,
///             export = "Java_com_custom_CustomName",
///         },
///
///         // Explicitly not exported (override global default)
///         fn method_four {
///             sig = (value: jint) -> jint,
///             export = false,
///         },
///     }
/// }
/// ```
///
/// #### Complete Working Example
///
/// ```
/// # use jni::bind_java_type;
/// # use jni::{Env, EnvUnowned};
/// # use jni::objects::{JClass, JString};
/// # use jni::sys::jint;
/// #
/// bind_java_type! {
///     rust_type = ExampleType,
///     java_type = "com.example.ExampleType",
///     export_native_methods = false,
///     native_methods = {
///         // Trait implementation, wrapped with catch_unwind
///         fn native_add(a: jint, b: jint) -> jint,
///
///         // Static method via trait
///         static fn native_initialize() -> jint,
///
///         // Exported method (generates JNI export symbol)
///         extern fn native_exported(value: jint) -> jint,
///
///         // Direct function implementation (bypasses trait)
///         fn native_direct {
///             sig = (value: jint) -> jint,
///             fn = my_implementation,
///         },
///
///         // Raw method via trait (no wrapping)
///         raw fn native_raw_trait(value: jint) -> jint,
///
///         // Raw method with direct function
///         raw fn native_raw_direct {
///             sig = (value: jint) -> jint,
///             fn = my_raw_function,
///         },
///     }
/// }
///
/// // Implement the trait for methods without `fn` property
/// impl ExampleTypeNativeInterface for ExampleTypeAPI {
///     type Error = jni::errors::Error;
///
///     fn native_add<'local>(
///         _env: &mut Env<'local>,
///         _this: ExampleType<'local>,
///         a: jint,
///         b: jint,
///     ) -> Result<jint, Self::Error> {
///         Ok(a + b)
///     }
///
///     fn native_initialize<'local>(
///         _env: &mut Env<'local>,
///         _class: JClass<'local>,
///     ) -> Result<jint, Self::Error> {
///         Ok(42)
///     }
///
///     fn native_exported<'local>(
///         _env: &mut Env<'local>,
///         _this: ExampleType<'local>,
///         value: jint,
///     ) -> Result<jint, Self::Error> {
///         Ok(value + 1)
///     }
///
///     fn native_raw_trait<'local>(
///         _env: EnvUnowned<'local>,
///         _this: ExampleType<'local>,
///         value: jint,
///     ) -> jint {
///         value * 2
///     }
/// }
///
/// // Direct function implementation (non-raw, returns Result)
/// fn my_implementation<'local>(
///     _env: &mut Env<'local>,
///     _this: ExampleType<'local>,
///     value: jint,
/// ) -> Result<jint, jni::errors::Error> {
///     Ok(value * 3)
/// }
///
/// // Raw function implementation (no Result, no wrapping)
/// fn my_raw_function<'local>(
///     _env: EnvUnowned<'local>,
///     _this: ExampleType<'local>,
///     value: jint,
/// ) -> jint {
///     value * 4
/// }
/// ```
///
/// ## Field Block: `fields`
///
/// Defines field bindings with getter and setter methods. Fields can be
/// instance or static, and use either shorthand or block syntax.
///
/// Field names follow the same [automatic name
/// conversion](#automatic-name-conversion-snake_case-to-lowercamelcase) as
/// methods when no explicit Java field name is provided.
///
/// ### Shorthand Syntax
///
/// ```custom
/// [visibility] [static] name: type
/// ```
///
/// Generates getter and setter methods as `name()` and `set_name()`.
///
/// ### Block Syntax
///
/// ```ignore
/// [visibility] [static] name {
///     [name = "javaFieldName",]
///     sig = type,
///     [get = getter_name,]
///     [set = setter_name,]
/// }
/// ```
///
/// Custom getter/setter names and documentation can be specified:
///
/// ```ignore
/// fields = {
///     // Shorthand: generates value() and set_value()
///     value: jint,
///
///     // Static field
///     static default_value: jint,
///
///     // Block syntax with custom Java name
///     internal_state {
///         name = "internalState",
///         sig = jboolean,
///     },
///
///     // Custom getter/setter names with separate documentation
///     special_field {
///         sig = jint,
///         /// Gets the special field value
///         get = get_special_value,
///         /// Sets the special field value
///         set = set_special_value,
///     },
///
///     // Different visibility for getter and setter
///     pub user_name {
///         sig = java.lang.String,
///         get = user_name,          // public getter
///         priv set = set_user_name, // private setter
///     },
/// }
/// ```
///
/// ## Special Properties
///
/// ### `jni`
///
/// Override the path to the `jni` crate. Must be specified first if provided.
///
/// ```ignore
/// bind_java_type! {
///     jni = ::my_jni_crate,
///     rust_type = MyType,
///     java_type = "com.example.MyClass",
/// }
/// ```
///
/// ### `api`
///
/// Custom name for the generated API struct (default: `{Type}API`).
///
/// ```ignore
/// api = MyCustomAPI
/// ```
///
/// ### `native_trait`
///
/// Custom name for the generated native methods trait (default:
/// `{Type}NativeInterface`).
///
/// ```ignore
/// native_trait = MyCustomTrait
/// ```
///
/// ### `export_native_methods`
///
/// Controls whether native methods generate JNI export symbols by default
/// (default: `true`). Individual methods can override this with `export =
/// true/false`.
///
/// ```ignore
/// export_native_methods = false  // Don't export by default
/// ```
///
/// ### `priv_type` and `hooks`
///
/// For advanced use cases, you can inject custom data into the API struct and
/// override class loading behavior.
///
/// ```ignore
/// bind_java_type! {
///     rust_type = MyType,
///     java_type = "com.example.MyClass",
///     priv_type = MyPrivateData,
///     hooks {
///         load_class = |env, load_context, initialize| {
///             load_context.load_class_for_type::<MyType>(env, initialize)
///         },
///         init_priv = |env, class, load_context| {
///             Ok(MyPrivateData::new())
///         },
///     },
/// }
/// ```
///
/// The `priv_type` is stored as a `private` field in the API struct and must
/// implement `Send + Sync`. The `init_priv` hook is called during API
/// initialization.
#[proc_macro]
pub fn bind_java_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    bind_java_type::bind_java_type_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

