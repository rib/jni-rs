/// Map primitive type to internal descriptor code (e.g., jint -> "I")
#[doc(hidden)]
#[macro_export]
macro_rules! __prim_to_internal {
    (void) => {
        "V"
    };
    (jboolean) => {
        "Z"
    };
    (jbyte) => {
        "B"
    };
    (jchar) => {
        "C"
    };
    (jshort) => {
        "S"
    };
    (jint) => {
        "I"
    };
    (jlong) => {
        "J"
    };
    (jfloat) => {
        "F"
    };
    (jdouble) => {
        "D"
    };
}

/// Convert a dot-separated package plus `::` separated inner classes to internal form with `/` and `$`
#[doc(hidden)]
#[macro_export]
macro_rules! __java_name_to_internal {
    ($(.)? $outer:ident $( :: $inner:ident )* ) => {
        concat!( stringify!($outer) $(, "$", stringify!($inner) )* )
    };
    ($first:ident . $($rest:tt)+) => { $crate::__java_name_to_internal!(@accum [$first] $($rest)+) };
    (@accum [$($pkg:ident)+] $next:ident . $($more:tt)+) => {
        $crate::__java_name_to_internal!(@accum [$($pkg)+ $next] $($more)+)
    };
    (@accum [$($pkg:ident)+] $outer:ident $( :: $inner:ident )* ) => {
        concat!(
            $( stringify!($pkg), "/", )*
            stringify!($outer)
            $(, "$", stringify!($inner) )*
        )
    };
}

/// Convert a dot-separated package plus `::` separated inner classes to internal form with `/` and `$`
/// and wrap with 'L' and ';' for object types within descriptors.
#[doc(hidden)]
#[macro_export]
macro_rules! __java_obj_to_internal {
    ( $($name:ident).+ $(:: $($inner:ident)::+ )? ) => { concat!("L", $crate::__java_name_to_internal!($($name).+ $(:: $($inner)::+ )?), ";") };
}

/// Maps a normalized Java array type (such as `[jint]` or `[java.lang.String]`) to its internal
/// descriptor form `[I` or `[Ljava/lang/String;`
#[doc(hidden)]
#[macro_export]
macro_rules! __java_array_to_internal {
    ($kind:ident, $elem:tt, ([] $($rest:tt)*), $lt:lifetime)  => {
        //compile_error!(concat!("DEBUG: kind = '", stringify!($kind), "' elem = '", stringify!($elem), "' rest = '", stringify!($($rest)*), "'"));
        concat!("[", $crate::__java_array_to_internal!($kind, $elem, ($($rest)*), $lt) )
    };
    (prim, ( $elem:ident ), (), $lt:lifetime)  => {
        $crate::__prim_to_internal!( $elem )
    };
    (obj, ( $( $obj:tt)* ), (), $lt:lifetime)  => {
        $crate::__java_obj_to_internal!( $($obj)*)
    };
}

/// Maps normalized Java types to their internal descriptor forms.
///
/// Rust types trigger a compile-time error since they are not supported in literal signatures.
#[doc(hidden)]
#[macro_export]
macro_rules! __java_type_to_internal_literal {
    ( prim ( $p:ident ) as $as:tt ) => { $crate::__prim_to_internal!($p) };
    ( obj ( ( $($obj:tt)+ ), $lt:lifetime ) as $as:tt ) => { $crate::__java_obj_to_internal!{ $($obj)+ } };

    // The macros should never try to expand a literal signature with a Rust type
    ( rust $rust:tt as $as:tt ) => { compile_error!("BUG: Rust types are not supported in literal signatures") };

    (array (rust, ($elem:ident), $dims:tt, $lt:lifetime) as $as:tt) => {
        compile_error!("BUG: Rust array types are not supported in literal signatures")
    };
    (array ($kind:ident, $elem:tt, $dims:tt, $lt:lifetime) as $as:tt) => {
        $crate::__java_array_to_internal!{$kind, $elem, $dims, $lt }
    };
}

/// Maps normalized Java or Rust types to their internal descriptor forms.
///
/// Rust types are dynamically mapped to their class names via the `Reference::class_name()` method.
#[doc(hidden)]
#[macro_export]
macro_rules! __any_type_to_internal_owned {
    // rust(...) is only used in formatted signatures
    ( rust $rust:tt as ($($r:tt)+) ) => {
        match <$($r)+ as $crate::refs::Reference>::class_name() {
            Cow::Borrowed(s) => s.to_str(),
            Cow::Owned(s) => Cow::Owned(s.to_string()),
        }
    };

    // prim | obj
    ( $kind:ident $arg:tt as $as:tt ) => {
        $crate::__java_type_to_internal_literal!( $kind $arg as $as )
    };
}

/// Normalize one raw type and immediately invoke a callback macro with the
/// normalized form.
///
/// # Usage:
///
///  ```
///   __jsig_normalize_type_then!(CB, { lifetimes... }, ( <raw-type> ) [, extra tokens... ])
///  ```
///
/// # Calls:
///
///   ```
///   CB!( (<normalized-type>), ( arg_lifetime_opt ), { leftover_lifetimes... } [, extra tokens...] )
///   ```
///
/// # Normalized shapes:
///
///   - Primitive types: `prim( jint ) as (jint)`
///   - Java types: `obj( (<java.name::Inner>), 'lifetime ) as ( <Ty> )`
///     (default Ty = JObject if no `as`)
///   - Rust `Reference` types: `rust( (<Ty>), 'lifetime ) as ( <Ty> )` (default
///     Ty = JObject if no `as`)
///   - Array types: `array( prim|obj|rust, (<elem-type>), (<[]..dims>),
///     'lifetime ) as ( <Ty> )` (default Ty = JObjectArray<JObject> or
///     JPrimitiveArray<T> if no `as`)
///
/// # Normalization examples:
///
/// ```
///   - `jint`                           -> `prim(jint) as (jint)`
///   - `int`                            -> `prim(jint) as (jint)`
///   - `i32`                            -> `prim(jint) as (jint)`
///   - `java.lang.String`               -> `obj((java.lang.String), 'lifetime)` as `(JObject)`
///   - `java.lang.String` as `JString`  -> `obj((java.lang.String), 'lifetime)` as `(JString)`
///   - `.NoPackage`                     -> `obj((NoPackage), 'lifetime)` as `(JObject)`
///   - `.NoPackage` as `JString`        -> `obj((NoPackage), 'lifetime)` as `(JString)`
///   - `&JString`                       -> `rust((JString), 'lifetime) as (JString)`
///   - `[[jint]]`                       -> `array(prim, (jint), ([][]), 'lifetime) as (JObjectArray<JPrimitiveArray<jint>>) `
///   - `&[jint]`                        -> `array(prim, (jint), ([]), 'lifetime) as (JPrimitiveArray<jint>)`
///   - `jint[]`                         -> `array(prim, (jint), ([]), 'lifetime) as (JPrimitiveArray<jint>)`
///   - `int[][]`                        -> `array(prim, (jint), ([][]), 'lifetime) as (JObjectArray<JPrimitiveArray<jint>>) `
///   - `[i32]`                          -> `array(prim, (jint), ([]), 'lifetime) as (JPrimitiveArray<jint>)`
///   - `[java.lang.String]`             -> `array(obj,  (java.lang.String), ([]), 'lifetime) as (JObjectArray<JObject>)`
///   - `java.lang.String[]`             -> `array(obj,  (java.lang.String), ([]), 'lifetime) as (JObjectArray<JObject>)`
///   - `&[JString]`                     -> `array(rust, (JString), ([]), 'lifetime) as (JObjectArray<JString>)`
///   - `[JString]`                      -> `array(rust, (JString), ([]), 'lifetime) as (JObjectArray<JString>)`
///   - `&[[JString]]`                   -> `array(rust, (JString), ([][]), 'lifetime) as (JObjectArray<JObjectArray<JString>>)`
/// ```
///
/// Note that obj, rust and array types are all given a lifetime parameter
/// (taken from a bag of lifetimes) which is included in the normalized argument
/// and also passed to the callback macro as an optional argument lifetime. This
/// can be used when emitting method signatures to allow each argument to have
/// its own lifetime and is required because Rust declarative macros can't
/// concatenate unique lifetime names.
///
/// Key cases to handle:
///  - Primitive types and their aliases
///  - Java object types with or without an 'as' clause
///      - Fully qualified names (with package) like `java.lang.String`
///      - Default package names like `.NoPackage`
///  - Rust `Reference` types like JString with or without a leading '&' and
///    with or without an 'as' clause
///  - Array types like `[[jint]]`, `[[java.lang.String]]`, `[[.NoPackage]]` or
///    `[[JString]]`
///  - Suffix array types like `jint[][]`, `java.lang.String[][]`,
///    `.NoPackage[][]` or `JString[][]`
///
/// In terms of disambiguating the input, the rules (in order of precedence)
/// are:
///  - If the type is a known primitive `:ident` or alias, it's a primitive
///  - Else, if the type starts with a dot or an identifier followed by a dot,
///    it's a Java type
///  - Else, if the type looks like a path (contains '::' or is a single
///    identifier), it's a Rust type
///
/// To allow Rust Reference-type arguments to be specified with a leading '&'
/// (for convenience), we simply strip/ignore any leading '&' before continuing
/// parsing. This also allows things like `&java.lang.String` as an
/// implementation detail but won't be recommended.
///
/// Questions:
/// - Should `as` clauses be more-clearly "unsafe", like `as unsafe(JString)` to
///   make it clear that the user is taking responsibility for ensuring the cast
///   is valid?
///
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_normalize_type_then {

    ( $cb:ident, $lifetimes:tt, ( & $($rest:tt)* ) $(, $($pass:tt)* )? ) => {
        // Strip any leading '&' and continue (allowed for convenience)
        $crate::__jsig_normalize_type_then!{$cb, $lifetimes, ( $($rest)* ) $(, $($pass)* )? }
    };

    ( $cb:ident, $lifetimes:tt, ( [ $($inner:tt)+ ] ) $(, $($pass:tt)* )? ) => {
        //compile_error!(concat!("DEBUG: normalizing: [", stringify!($($inner)+), "]"));
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () (unknown) ([]) () [ $($inner)+ ] $(, $($pass)* )? }
    };

    ( $cb:ident, $lifetimes:tt, ( [ $($inner:tt)+ ] as $( $as:tt )+ ) $(, $($pass:tt)* )? ) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () (unknown) ([]) ( $($as)* ) [ $($inner)+ ] $(, $($pass)* )? }
    };

    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ & $($rest:tt)* ] $(, $($pass:tt)* )?) => {
        // Ignore any leading '&' and continue
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, ($($left)*) ($($right)*) (unknown) $dims $as [ $($rest)* ] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) ($($dims:tt)*) $as:tt [ [ $($inner:tt)+ ] ] $(, $($pass:tt)* )?) => {
        // More dimensions: wrap in JObjectArray<...> and recurse
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, (JObjectArray< $($left)*) ($($right)* >) (unknown) ([]$($dims)*) $as [ $($inner)+ ] $(, $($pass)* )? }
    };

    (@arr $cb:ident, { $lt:lifetime, $($lt_rest:tt)* }, ()  () (prim($prim:ident, $($default:tt)*)) $dims:tt () [] $(, $($pass:tt)* )?) => {
        $crate::$cb!{ ( array(prim, ($prim), $dims, $lt) as ( $($default)* ) ), ($lt), { $($lt_rest)* } $(, $($pass)* )? }
    };
    (@arr $cb:ident, { $lt:lifetime, $($lt_rest:tt)* }, ()  () (prim($prim:ident, $($default:tt)*)) $dims:tt $as:tt [] $(, $($pass:tt)* )?) => {
        $crate::$cb!{ ( array(prim, ($prim), $dims, $lt) as $as ), ($lt), { $($lt_rest)* } $(, $($pass)* )? }
    };

    (@arr $cb:ident, { $lt:lifetime, $($lt_rest:tt)* }, ()  () (obj($elem:tt, ($($default:tt)*))) $dims:tt () [] $(, $($pass:tt)* )?) => {
        //compile_error!(concat!("DEBUG: normalized arg as array(obj,", stringify!($elem), stringify!($dims), ") as (", stringify!($($default)*), "), calling: ", stringify!($crate::$cb!{ array(obj, $elem, $dims) as ( $($default)* ) $(, $($pass)* )? })));
        $crate::$cb!{ ( array(obj, $elem, $dims, $lt) as ( $($default)* ) ), ($lt), { $($lt_rest)* } $(, $($pass)* )? }
    };
    (@arr $cb:ident, { $lt:lifetime, $($lt_rest:tt)* }, ()  () (obj($obj:tt, $($default:tt)*)) $dims:tt $as:tt [] $(, $($pass:tt)* )?) => {
        $crate::$cb!{ ( array(obj, $obj, $dims, $lt) as $as ), ($lt), { $($lt_rest)* } $(, $($pass)* )? }
    };

    (@arr $cb:ident, { $lt:lifetime, $($lt_rest:tt)* }, ()  () (rust($elem:tt, ($($default:tt)*))) $dims:tt () [] $(, $($pass:tt)* )?) => {
        $crate::$cb!{ ( array(rust, $elem, $dims, $lt) as ( $($default)* ) ), ($lt), { $($lt_rest)* } $(, $($pass)* )? }
    };
    (@arr $cb:ident, { $lt:lifetime, $($lt_rest:tt)* }, ()  () (rust($elem:tt, $($default:tt)*)) $dims:tt $as:tt [] $(, $($pass:tt)* )?) => {
        $crate::$cb!{ ( array(rust, $elem, $dims, $lt) as $as ), ($lt), { $($lt_rest)* } $(, $($pass)* )? }
    };

    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ jboolean ] $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jboolean, $($left)* JPrimitiveArray<$crate::sys::jboolean> $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ boolean ]  $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jboolean, $($left)* JPrimitiveArray<$crate::sys::jboolean> $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ bool ]     $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jboolean, $($left)* JPrimitiveArray<$crate::sys::jboolean> $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ jbyte ]    $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jbyte, $($left)* JPrimitiveArray<$crate::sys::jbyte>    $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ byte ]     $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jbyte, $($left)* JPrimitiveArray<$crate::sys::jbyte>    $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ i8 ]       $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jbyte, $($left)* JPrimitiveArray<$crate::sys::jbyte>    $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ jchar ]    $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jchar, $($left)* JPrimitiveArray<$crate::sys::jchar>    $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ char ]     $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jchar, $($left)* JPrimitiveArray<$crate::sys::jchar>    $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ jshort ]   $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jshort, $($left)* JPrimitiveArray<$crate::sys::jshort>   $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ short ]    $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jshort, $($left)* JPrimitiveArray<$crate::sys::jshort>   $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ i16 ]      $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jshort, $($left)* JPrimitiveArray<$crate::sys::jshort>   $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ jint ]     $(, $($pass:tt)* )?) => {
        //compile_error!(concat!("DEBUG: jint array hit: left = '", stringify!($($left)*), "' right = '", stringify!($($right)*), "' dims = '", stringify!($dims), "' as = '", stringify!($as), "'"));
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jint, $($left)* JPrimitiveArray<$crate::sys::jint>     $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ int ]      $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jint, $($left)* JPrimitiveArray<$crate::sys::jint>     $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ i32 ]      $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jint, $($left)* JPrimitiveArray<$crate::sys::jint>     $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ jlong ]    $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jlong, $($left)* JPrimitiveArray<$crate::sys::jlong>    $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ long ]     $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jlong, $($left)* JPrimitiveArray<$crate::sys::jlong>    $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ i64 ]      $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jlong, $($left)* JPrimitiveArray<$crate::sys::jlong>    $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ jfloat ]   $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jfloat, $($left)* JPrimitiveArray<$crate::sys::jfloat>   $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ float ]    $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jfloat, $($left)* JPrimitiveArray<$crate::sys::jfloat>   $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ f32 ]      $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jfloat, $($left)* JPrimitiveArray<$crate::sys::jfloat>   $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ jdouble ]  $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jdouble, $($left)* JPrimitiveArray<$crate::sys::jdouble>  $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ double ]   $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jdouble, $($left)* JPrimitiveArray<$crate::sys::jdouble>  $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ f64 ]   $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( prim(jdouble, $($left)* JPrimitiveArray<$crate::sys::jdouble>  $($right)* ) ) $dims $as [] $(, $($pass)* )? }
    };

    // Java objects, with package prefix...
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ $first:ident $( . $seg:ident )+ $( :: $inner:ident )* ]   $(, $($pass:tt)* )?) => {
        //compile_error!(concat!("DEBUG: object array hit: left = '", stringify!($($left)*), "' right = '", stringify!($($right)*), "' dims = '", stringify!($dims), "' as = '", stringify!($as), "': ", stringify!($first $( . $seg )+ $( :: $inner )* )));
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( obj(( $first $( . $seg )+ $( :: $inner )* ), ( $($left)* JObjectArray<JObject>  $($right)* )) ) $dims $as [] $(, $($pass)* )? }
    };
    // Default-package Java objects...
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ . $outer:ident $( :: $inner:ident )* ]   $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( obj(( $outer $( :: $inner )* ), ( $($left)* JObjectArray<JObject>  $($right)* ) ) ) $dims $as [] $(, $($pass)* )? }
    };

    // Rust types...
    (@arr $cb:ident, $lifetimes:tt, ($($left:tt)*)  ($($right:tt)*) (unknown) $dims:tt $as:tt [ $path0:ident $( :: $pathX:ident )* $( < $(, $param:ident)+ > )? ]   $(, $($pass:tt)* )?) => {
        $crate::__jsig_normalize_type_then!{@arr $cb, $lifetimes, () () ( rust(( $path0 $( :: $pathX )* $(< $(, $param)+ > )? ), ( $($left)* JObjectArray< $path0 $( :: $pathX )* $(< $(, $param)+ > )? >  $($right)* ) ) ) $dims $as [] $(, $($pass)* )? }
    };

    // ----- Primitives (canonicalize) -----
    ( $cb:ident, $lifetimes:tt, ( () )       $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( void )     as (())),       (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( void )     $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( void )     as (())),       (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( jboolean ) $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jboolean ) as (jboolean)), (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( boolean )  $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jboolean ) as (jboolean)), (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( bool )     $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jboolean ) as (jboolean)), (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( jbyte )    $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jbyte )    as (jbyte)),    (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( byte )     $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jbyte )    as (jbyte)),    (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( i8 )       $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jbyte )    as (jbyte)),    (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( jchar )    $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jchar )    as (jchar)),    (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( char )     $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jchar )    as (jchar)),    (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( jshort )   $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jshort )   as (jshort)),   (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( short )    $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jshort )   as (jshort)),   (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( i16 )      $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jshort )   as (jshort)),   (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( jint )     $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jint )     as (jint)),     (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( int )      $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jint )     as (jint)),     (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( i32 )      $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jint )     as (jint)),     (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( jlong )    $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jlong )    as (jlong)),    (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( long )     $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jlong )    as (jlong)),    (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( i64 )      $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jlong )    as (jlong)),    (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( jfloat )   $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jfloat )   as (jfloat)),   (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( float )    $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jfloat )   as (jfloat)),   (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( f32 )      $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jfloat )   as (jfloat)),   (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( jdouble )  $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jdouble )  as (jdouble)),  (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( double )   $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jdouble )  as (jdouble)),  (), $lifetimes $(, $($pass)* )? } };
    ( $cb:ident, $lifetimes:tt, ( f64 )      $(, $($pass:tt)* )? ) => { $crate::$cb!{ ( prim( jdouble )  as (jdouble)),  (), $lifetimes $(, $($pass)* )? } };


    // ----- Java object type -----
    ( $cb:ident, { $lt:lifetime, $($lt_rest:tt)* }, ( $first:ident $( . $seg:ident )+ $( :: $inner:ident )* as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
       $crate::$cb!{ ( obj( ($first $( . $seg )+ $( :: $inner )* ), $lt) as ( $as_ty ) ), ($lt), { $($lt_rest)* } $(, $($pass)* )? }
    };
    ( $cb:ident, { $lt:lifetime, $($lt_rest:tt)* }, ( $first:ident $( . $seg:ident )+ $( :: $inner:ident )* ) $(, $($pass:tt)* )? ) => {
      //compile_error!(concat!("DEBUG: Using default type JObject for java type ", stringify!($first $( . $seg )+ $( :: $inner )* ), "cb = ", stringify!($cb), ", pass = ", stringify!($($($pass)*)?)));
      $crate::$cb!{ ( obj( ($first $( . $seg )+ $( :: $inner )* ), $lt) as ( $crate::objects::JObject ) ), ($lt), { $($lt_rest)* } $(, $($pass)* )? }
    };

    // ----- Java object type (default package) -----
    ( $cb:ident, { $lt:lifetime, $($lt_rest:tt)* }, ( . $outer:ident $( :: $inner:ident )* as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $crate::$cb!{ ( obj( ( $outer $( :: $inner )* ), $lt) as ( $as_ty ) ), ($lt), { $($lt_rest)* } $(, $($pass)* )? }
    };
    ( $cb:ident, { $lt:lifetime, $($lt_rest:tt)* }, ( . $outer:ident $( :: $inner:ident )* ) $(, $($pass:tt)* )? ) => {
        $crate::$cb!{ ( obj( ( $outer $( :: $inner )* ), $lt) as ( $crate::objects::JObject ) ), ($lt), { $($lt_rest)* } $(, $($pass)* )? }
    };

    // Suffix array with Java primitive or object or Rust path with optional as clause
    ( $cb:ident, $lifetimes:tt, ( $first:ident $( . $seg:ident )* $( :: $inner:ident )* [] $($rest:tt)* ) $(, $($pass:tt)* )? ) => {
        $crate::__jsig_normalize_type_then! { @arr_suffix $cb, $lifetimes,
            ( [ $first $(. $seg )* $( :: $inner )* ] ),
            [ $($rest)* ] $(, $($pass)* )?
        }
    };
    // Suffix array with default-package Java object with optional as clause
    ( $cb:ident, $lifetimes:tt, ( . $first:ident $( :: $inner:ident )* [] $($rest:tt)* ) $(, $($pass:tt)* )? ) => {
        $crate::__jsig_normalize_type_then! { @arr_suffix $cb, $lifetimes,
            ( [ . $first $( :: $inner )* ] ),
            [ $($rest)* ] $(, $($pass)* )?
        }
    };
    ( @arr_suffix $cb:ident, $lifetimes:tt, ( $($base:tt)+ ), [ [] $($rest:tt)* ] $(, $($pass:tt)* )? ) => {
        $crate::__jsig_normalize_type_then! { @arr_suffix $cb, $lifetimes, ( [ $($base)+ ]), [ $($rest)* ] $(, $($pass)* )? }
    };
    ( @arr_suffix $cb:ident, $lifetimes:tt, ( $($base:tt)+ ), [ [] as $as_ty:ty ] $(, $($pass:tt)* )? ) => {
        $crate::__jsig_normalize_type_then!{ $cb, $lifetimes, ( [ $($base)+ ] as $as_ty ) $(, $($pass)* )? }
    };
    ( @arr_suffix $cb:ident, $lifetimes:tt, ( $($base:tt)+ ), [ $(, $rest:tt)* ] $(, $($pass:tt)* )? ) => {
        $crate::__jsig_normalize_type_then!{ $cb, $lifetimes, ( $($base)+ ) $(, $($pass)* )? }
    };

    // ----- Rust type: JString, JObject, etc. (must come after primitives) -----
    ( $cb:ident, { $lt:lifetime, $($lt_rest:tt)* }, ( $first:ident $(:: $rest:ident )* ) $(, $($pass:tt)* )? ) => {
        $crate::$cb!{ ( rust( ($first $(:: $rest )* ), $lt)  as ( $first $(:: $rest )* ) ), ($lt), { $($lt_rest)* } $(, $($pass)* )? }
    };
    ( $cb:ident, { $lt:lifetime, $($lt_rest:tt)* }, ( $first:ident $(:: $rest:ident )* as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $crate::$cb!{ ( rust( ($first $(:: $rest )* ), $lt) as ( $as_ty ) ), ($lt), { $($lt_rest)* } $(, $($pass)* )? }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_normalize_ret_then__shim {
    ( $ret:tt, $_ret_lts:tt, $lifetimes:tt, $real_cb:ident $(, $($rest:tt)* )? ) => {
        $crate::$real_cb!{ $ret, $lifetimes $(, $($rest)* )? }
    };
}

/// Normalize a return type, then invoke a callback macro that expects a normalized type
///
/// # Usage
///
///  - `__jsig_normalize_ret_then!(CB, { lifetimes... }, ( RetTy ) [, extra tokens...])`
///
/// Calls `CB!( ( normalized RetTy ), { lifetimes... } [, extra tokens...])` when done.
///
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_normalize_ret_then {
    ( $cb:ident, $lifetimes:tt, $ret:tt $(, $($extra:tt)* )? ) => {
        //compile_error!(concat!("DEBUG: Normalizing return type: '", stringify!($($ret)+), "' pass = '", stringify!($($($extra)*)? ), "'"));
        $crate::__jsig_normalize_type_then!{ __jsig_normalize_ret_then__shim, $lifetimes, $ret, $cb $(, $($extra)* )? }
    };
}

/// Push one normalized arg into accumulator and continue munching.
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_normalize_args_then__push {
    ( ( $($norm:tt)+ ), ($type_lt:lifetime), $lifetimes:tt, $cb:ident, [ $($acc:tt)* ], [ $(,)? $($acc_lts:lifetime),* ], [$($rest:tt)*], $n:ident, $extra:tt ) => {
        $crate::__jsig_normalize_args_then__munch!{ $cb, $lifetimes, [ $($acc)*, $n: $($norm)+ ], [ $($acc_lts),* , $type_lt ], [$($rest)*], $extra  }
    };
    ( ( $($norm:tt)+ ), (), $lifetimes:tt, $cb:ident, [ $($acc:tt)* ], [ $(,)? $($acc_lts:lifetime),* ], [$($rest:tt)*], $n:ident, $extra:tt ) => {
        $crate::__jsig_normalize_args_then__munch!{ $cb, $lifetimes, [ $($acc)*, $n: $($norm)+ ], [ $($acc_lts),* ], [$($rest)*], $extra  }
    };
}

/// Internal muncher that uses __jsig_normalize_type_then per argument type and accumulates a normalized list.
///
/// The accumulator + $rest are initialized with a dummy `@HEAD` entry to help avoid special-casing the first/last argument.
///
/// When we reach the end, we strip off the `@HEAD` and call the final callback.
///
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_normalize_args_then__munch {

    // Finished
    ( $cb:ident, $lifetimes:tt, [ @HEAD, $($acc:tt)* ], [$(,)? $($acc_lts:lifetime),*], [@HEAD $(,)?], ( $cb2:ident $(, $($extra:tt)* )? ) ) => {
        // TODO: accumulate a list of lifetimes for Reference-type arguments
        $crate::$cb!{ ( $($acc)* ), ($($acc_lts),*), $lifetimes, $cb2 $(, $($extra)* )? }
    };

    // Java primitive or object or Rust path with optional as clause
    ( $cb:ident, $lifetimes:tt, $acc:tt, $acc_lts:tt,
      [ @HEAD, $n:ident : $(&)? $first:ident $(. $seg:ident )* $( :: $inner:ident )* $(as $as_ty:ty)? $(, $($rest:tt)*)? ],
      $extra:tt
    ) => {
        $crate::__jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, $lifetimes, ( $first $( . $seg )* $( :: $inner )* $(as $as_ty)? ),
            $cb, $acc, $acc_lts, [@HEAD $(, $($rest)*)? ], $n, $extra
        }
    };
    // Default-package Java object with optional as clause
    ( $cb:ident, $lifetimes:tt, $acc:tt, $acc_lts:tt,
      [ @HEAD, $n:ident : . $first:ident $(. $seg:ident )* $( :: $inner:ident )* $(as $as_ty:ty)? $(, $($rest:tt)*)? ],
      $extra:tt
    ) => {
        $crate::__jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, $lifetimes, ( . $first $( . $seg )* $( :: $inner )* $(as $as_ty)? ),
            $cb, $acc, $acc_lts, [@HEAD $(, $($rest)*)? ], $n, $extra
        }
    };

    // Array form, with optional as clause
    ( $cb:ident, $lifetimes:tt, $acc:tt, $acc_lts:tt,
      [ @HEAD, $n:ident : $(&)? [ $($inner:tt)+ ] $( as $as_ty:ty )? $(, $($rest:tt)*)?],
      $extra:tt
    ) => {
        $crate::__jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, $lifetimes, ( [ $($inner)+ ] $( as $as_ty:ty )? ),
            $cb, $acc, $acc_lts, [@HEAD $(, $($rest)*)? ], $n, $extra
        }
    };

    // Suffix array with Java primitive or object or Rust path with optional as clause
    // Note: we take the opportunity to normalize from foo.bar[][] to [[foo.bar]]
    ( $cb:ident, $lifetimes:tt, $acc:tt, $acc_lts:tt,
      [ @HEAD, $n:ident : $(&)? $first:ident $(. $seg:ident )* $( :: $inner:ident )* [] $($rest:tt)* ],
      $extra:tt
    ) => {
        //compile_error!(concat!("DEBUG: Suffix array hit: n = '", stringify!($n), "' first = '", stringify!($first), "' seg = '", stringify!($($seg)*), "' inner = '", stringify!($($inner)*), "' rest = '", stringify!($($rest)*), "' extra = '", stringify!($extra), "' acc = '", stringify!($acc), "'"));
        $crate::__jsig_normalize_args_then__munch! { @arr_suffix $cb, $lifetimes, $acc, $acc_lts,
            $n ( [ $first $(. $seg )* $( :: $inner )* ] ),
            [ $($rest)* ],
            $extra
        }
    };
    // Suffix array with default-package Java object with optional as clause
    // Note: we take the opportunity to normalize from .foo[][] to [[.foo]]
    ( $cb:ident, $lifetimes:tt, $acc:tt, $acc_lts:tt,
      [ @HEAD, $n:ident : . $first:ident $( :: $inner:ident )* [] $($rest:tt)* ],
      $extra:tt
    ) => {
        $crate::__jsig_normalize_args_then__munch! { @arr_suffix $cb, $lifetimes, $acc, $acc_lts,
            $n ( [ . $first $( :: $inner )* ] ),
            [ $($rest)* ],
            $extra
        }
    };
    ( @arr_suffix $cb:ident, $lifetimes:tt, $acc:tt, $acc_lts:tt, $n:ident ( $($base:tt)+ ), [ [] $($rest:tt)* ], $extra:tt ) => {
        $crate::__jsig_normalize_args_then__munch! { @arr_suffix $cb, $lifetimes, $acc, $acc_lts, $n ( [ $($base)+ ]), [ $($rest)* ], $extra }
    };
    ( @arr_suffix $cb:ident, $lifetimes:tt, $acc:tt, $acc_lts:tt, $n:ident ( $($base:tt)+ ), [ as $as_ty:ty $(, $rest:tt)* ], $extra:tt ) => {
        $crate::__jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, $lifetimes,
            ( $($base)+ as $as_ty ),
            $cb, $acc, $acc_lts, [@HEAD $(, $rest)* ], $n, $extra
        }
    };
    ( @arr_suffix $cb:ident, $lifetimes:tt, $acc:tt, $acc_lts:tt, $n:ident ( $($base:tt)+ ), [ $(, $( $rest:tt)*)? ], $extra:tt ) => {
        //compile_error!(concat!("DEBUG: Finalizing suffix array for arg ", stringify!($n), " base = '", stringify!($($base)+), "' rest = '", stringify!($(, $($rest)*)?), "' extra = '", stringify!($extra), "' acc = '", stringify!($acc), "'"));
        $crate::__jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, $lifetimes,
            ( $($base)+ ),
            $cb, $acc, $acc_lts, [@HEAD $(, $($rest)*)? ], $n, $extra
        }
    };
    ( @arr_suffix $($catch:tt)*) => { compile_error!(concat!("DEBUG: Internal error: malformed suffix array muncher args: ", stringify!($($catch)*))); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_normalize_args_then__shim {
    ( $args:tt, $arg_lts:tt, $lifetimes:tt, $real_cb:ident $(, $($rest:tt)* )? ) => {
        $crate::$real_cb!{ $args, $arg_lts, $lifetimes $(, $($rest)* )? }
    };
}

/// Normalize an args list, then invoke a callback macro that expects normalized args
///
/// # Usage
///
/// ```
/// __jsig_normalize_args_then!(
///     CB,
///     {'lifetime0, 'lifetime1, ...},  // bag of lifetimes
///     ( a: TyA, b: TyB, ... ) [, extra tokens...])
/// ```
///
/// Note: the bag of unique lifetimes are required because Rust macro_rules can not concatonate
/// unique lifetimes, and individual method arguments may have different lifetimes.
///
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_normalize_args_then {
    // Empty args specializations to avoid trailing-comma edge cases
    ( $cb:ident, $lifetimes:tt, ( $(,)? ) $(, $($extra:tt)* )? ) => {
        $crate::$cb!{ (), (), $lifetimes $(, $($extra)* )? }
    };
    ( $cb:ident, $lifetimes:tt, ( $($args:tt)* ) $(, $($extra:tt)*)? ) => {
        $crate::__jsig_normalize_args_then__munch!{ __jsig_normalize_args_then__shim, $lifetimes, [@HEAD], [], [@HEAD, $($args)*], ($cb $(, $($extra)* )?) }
    };
}

/// Shim that routes from return type normalization to the real callback, accepting both normalized args and ret
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_normalize_args_ret_then__finish {
    ( $ret:tt, $lifetimes:tt, $real_cb:ident, $args:tt, $arg_lts:tt $(, $($rest:tt)* )? ) => {
        $crate::$real_cb!{ $args, $arg_lts, $ret $(, $($rest)* )? }
    };
}

/// Shim that forwards normalized args to return type normalization
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_normalize_args_ret_then__then_ret_shim {
    ( $args:tt, $arg_lts:tt, $lifetimes:tt, $real_cb:ident, ( $($raw_ret:tt)+ ) $(, $($extra:tt)* )? ) => {
        $crate::__jsig_normalize_ret_then!{__jsig_normalize_args_ret_then__finish, $lifetimes, ( $($raw_ret)+ ), $real_cb, $args, $arg_lts $(, $($extra)* )? }
    };
}

/// Normalize both arguments and return type, then invoke a callback macro that expects both
///
/// # Usage
///
/// ```
///  __jsig_normalize_args_ret_then!(
///     CB,
///     ( a: TyA, b: TyB, ... ), ( RetTy ) [, extra tokens...])
/// ```
///
/// Note: the bag of unique lifetimes are required because Rust macro_rules can not concatonate
/// unique lifetimes, and individual method arguments may have different lifetimes.
///
/// # Callback signature
///
/// The callback will be invoked as: `CB!( ( normalized_args... ), (arg_lifetimes...), ( normalized_ret ) [, extra tokens...] )`
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_normalize_args_ret_then {
    ( $cb:ident, ( $($args:tt)* ), ( $($ret:tt)+ ) $(, $($extra:tt)* )? ) => {
        $crate::__jsig_normalize_args_then!{
            __jsig_normalize_args_ret_then__then_ret_shim,
            {'any0, 'any1, 'any2, 'any3, 'any4, 'any5, 'any6, 'any7, 'any8, 'any9, 'any10, 'any11, 'any12, 'any13, 'any14, 'any15, 'any16, 'any17, 'any18, 'any19, 'any20},
            ( $($args)* ), $cb, ( $($ret)+ ) $(, $($extra)* )?}
    };
}

/// Compose a JNI signature as a string literal
///
/// Returns a `Cow::Borrowed`
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_emit_sig_literal {
    ( ( $( $n:ident : $ak:ident $arg:tt as $arg_as:tt ),* )
      -> ( $rk:ident $ret:tt as $ret_as:tt )
    ) => {
        const {
            let s = concat!(
                "(",
                $( $crate::__java_type_to_internal_literal!( $ak $arg as $arg_as ), )*
                ")",
                $crate::__java_type_to_internal_literal!( $rk $ret as $ret_as ),
                "\0"
            );
            // Safety: the concat! above guarantees a null terminator with no interior nulls
            let cs = unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(s.as_bytes()) };
            let js = $crate::strings::JNIStr::from_cstr(cs);
            std::borrow::Cow::Borrowed(js)
        }
    };
}

/// Compose a JNI signature dynamically into a `String`
///
/// Returns a `Cow::Owned`
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_emit_sig_dynamic_owned {
    ( ( $( $n:ident : $ak:ident $arg:tt as $arg_as:tt ),* )
      -> ( $rk:ident $ret:tt as $ret_as:tt )
    ) => {
        {
            let mut buf = String::new();
            fn extend_with_internal(buf: &mut String, s: &str) {
                // If it's not a primitive (len > 1) or array descriptor it's a class name
                if s.len() > 1 && !s.starts_with('[') {
                    buf.push('L');
                    buf.extend(s.chars().map(|c| if c == '.' { '/' } else { c }));
                    buf.push(';');
                } else {
                    buf.extend(s.chars().map(|c| if c == '.' { '/' } else { c }));
                }
            }
            buf.push('(');
            $( extend_with_internal(&mut buf, &$crate::__any_type_to_internal_owned!( $ak $arg as $arg_as )); )*
            buf.push(')');
            extend_with_internal(&mut buf, &$crate::__any_type_to_internal_owned!( $rk $ret as $ret_as ));
            //buf.push('\0');
            let cs = std::ffi::CString::new(buf).expect("internal error: signature contained null byte");
            // Safety: the CString above guarantees a null terminator with no interior nulls
            // The string itself is composed from ascii characters or JNIStr class names that were already
            // validated.
            let js = unsafe { $crate::strings::JNIString::from_cstring(cs) };
            std::borrow::Cow::Owned(js)
        }
    };
}

/// Assuming the args have already been checked for rust(...) types, check the return type
/// and determine whether to emit a literal (Cow::Borrowed) or dynamic (Cow::Owned) signature
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_emit_sig_cow__check_ret_then {
    // rust(...) return type, use dynamic callback
    ( ( rust $rust:tt as $as:tt ), $literal_cb:ident, $dynamic_cb:ident, ( $($orig_args:tt)* ) ) => {
        $crate::$dynamic_cb!( ( $($orig_args)* ) -> ( rust $rust as $as ) )
    };
    // rust-based array return type, use dynamic callback
    ( ( array (rust, $elem:tt, $dims:tt) as $as:tt ), $literal_cb:ident, $dynamic_cb:ident, ( $($orig_args:tt)* ) ) => {
        $crate::$dynamic_cb!( ( $($orig_args)* ) -> ( array (rust, $elem, $dims) as $as ) )
    };
    // Non-rust return type, use literal callback
    ( ( $ret_kind:ident $ret:tt as $as:tt ), $literal_cb:ident, $dynamic_cb:ident, ( $($orig_args:tt)* ) ) => {
        $crate::$literal_cb!( ( $($orig_args)* ) -> ( $ret_kind $ret as $as ) )
    };
}

/// Helper to recursively check if any type is rust(...) before chaining to return type check
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_emit_sig_cow__check_args_then {
    // Entrypoint
    ( ( $($args:tt)* ), ( $($nret:tt)* ), $literal_cb:ident, $dynamic_cb:ident, ( $($orig_args:tt)* ) ) => {
        //compile_error!(concat!("DEBUG: Checking args = ", stringify!( $($args)* ), " ret = ", stringify!( $($nret)* )));
        $crate::__jsig_emit_sig_cow__check_args_then!( @CHECK (@HEAD, $($args)* ), ( $($nret)* ), $literal_cb, $dynamic_cb, ( $($orig_args)* ) )
    };

    // Base case: no more args, check return type with original args preserved
    ( @CHECK (@HEAD $(,)?), ( $($nret:tt)* ), $literal_cb:ident, $dynamic_cb:ident, ( $($orig_args:tt)* ) ) => {
        $crate::__jsig_emit_sig_cow__check_ret_then!( ( $($nret)* ), $literal_cb, $dynamic_cb, ( $($orig_args)* ) )
    };
    // Recursive case: check first arg, continue with rest
    ( @CHECK (@HEAD, $first_name:ident : rust $rust:tt as $as:tt $(, $($rest_args:tt)*)? ), ( $($nret:tt)* ), $literal_cb:ident, $dynamic_cb:ident, ( $($orig_args:tt)* ) ) => {
        // Found rust(...) type, use dynamic callback
        $crate::$dynamic_cb!( ( $($orig_args)* ) -> ( $($nret)* ) )
    };
    ( @CHECK (@HEAD, $first_name:ident : array (rust, $($rest:tt)* ) as $as:tt $(, $($rest_args:tt)*)? ), ( $($nret:tt)* ), $literal_cb:ident, $dynamic_cb:ident, ( $($orig_args:tt)* ) ) => {
        // Found array(rust, ...) type, use dynamic callback
        $crate::$dynamic_cb!( ( $($orig_args)* ) -> ( $($nret)* ) )
    };
    // Recursive case: non-rust arg, continue checking
    ( @CHECK (@HEAD, $first_name:ident : $first_kind:ident $first_arg:tt as $as:tt $(, $($rest_args:tt)*)? ), ( $($nret:tt)* ), $literal_cb:ident, $dynamic_cb:ident, ( $($orig_args:tt)* ) ) => {
        $crate::__jsig_emit_sig_cow__check_args_then!( @CHECK ( @HEAD $(, $($rest_args)* )? ), ( $($nret)* ), $literal_cb, $dynamic_cb, ( $($orig_args)* ) )
    };
}

/// Given normalized args and ret, emit a `Cow<str>` signature either based on a `Cow::Borrowed` string
/// literal or a `Cow::Owned String` built at runtime.
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_emit_sig_cow {
    ( $args:tt, $_arg_lts:tt, $ret:tt ) => {
        //compile_error!(concat!("DEBUG: args = ", stringify!( $($norm_args)* ), " ret = ", stringify!( $($norm_ret)+ ) ));
        $crate::__jsig_emit_sig_cow__check_args_then!( $args, $ret, __jsig_emit_sig_literal, __jsig_emit_sig_dynamic_owned, $args )
    };
}

/// Get a raw `jobject` handle from a `Reference` Rust wrapper
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_obj_reference_as_raw {
    ( $ty:ty, $expr:expr ) => {{ <$ty as $crate::refs::Reference>::as_raw(($expr).as_ref()) }};
}

/// Maps a normalized argument type to a `[$crate::sys::jvalue]`
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_arg_to_jvalue {
    ( $name:ident : prim ( jboolean ) as $as:tt ) => {
        $crate::sys::jvalue {
            z: ($name as $crate::sys::jboolean),
        }
    };
    ( $name:ident : prim ( jbyte    ) as $as:tt ) => {
        $crate::sys::jvalue {
            b: ($name as $crate::sys::jbyte),
        }
    };
    ( $name:ident : prim ( jchar    ) as $as:tt ) => {
        $crate::sys::jvalue {
            c: ($name as $crate::sys::jchar),
        }
    };
    ( $name:ident : prim ( jshort   ) as $as:tt ) => {
        $crate::sys::jvalue {
            s: ($name as $crate::sys::jshort),
        }
    };
    ( $name:ident : prim ( jint     ) as $as:tt ) => {
        $crate::sys::jvalue {
            i: ($name as $crate::sys::jint),
        }
    };
    ( $name:ident : prim ( jlong    ) as $as:tt ) => {
        $crate::sys::jvalue {
            j: ($name as $crate::sys::jlong),
        }
    };
    ( $name:ident : prim ( jfloat   ) as $as:tt ) => {
        $crate::sys::jvalue {
            f: ($name as $crate::sys::jfloat),
        }
    };
    ( $name:ident : prim ( jdouble  ) as $as:tt ) => {
        $crate::sys::jvalue {
            d: ($name as $crate::sys::jdouble),
        }
    };

    // Java objects: get the raw `jobject` from the `as rust(...)` `Reference` type
    ( $name:ident : obj $obj:tt as ( $as_ty:ty ) ) => {
        $crate::sys::jvalue {
            l: $crate::__jsig_obj_reference_as_raw!($as_ty, $name),
        }
    };

    // Rust `Reference` types
    ( $name:ident : rust $rust:tt as ( $as_ty:ty ) ) => {
        $crate::sys::jvalue {
            l: $crate::__jsig_obj_reference_as_raw!($as_ty, $name),
        }
    };

    // Array types
    ( $name:ident : array ( $ak:ident, $elem:tt, $dims:tt ) as ( $as_ty:ty ) ) => {
        $crate::sys::jvalue {
            l: $crate::__jsig_obj_reference_as_raw!($as_ty, $name),
        }
    };
}

/// Build `[jvalue; N]` from a normalized arg list
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_args_to_jvalue_array {
    ( ( $( $an:ident : $ak:ident $arg:tt as $as:tt ),* $(,)? ) ) => {{
        [ $( $crate::__jsig_arg_to_jvalue!( $an : $ak $arg as $as ) ),* ]
    }};
}
