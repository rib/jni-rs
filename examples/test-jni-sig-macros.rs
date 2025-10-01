// Import the `paste!` macro
use paste::paste;

// ---- BEGIN: copy of JNI call macros from src/macros.rs ----

/// Directly calls a Env FFI function, nothing else
///
/// # Safety
///
/// When calling any function added after JNI 1.1 you must know that it's valid
/// for the current JNI version.
macro_rules! jni_call_unchecked {
    ( $jnienv:expr, $version:tt, $name:ident $(, $args:expr )*) => {{
        // Safety: we know that the Env pointer can't be null, since that's
        // checked in `from_raw()`
        let env: *mut jni_sys::JNIEnv = $jnienv.get_raw();
        let interface: *const jni_sys::JNINativeInterface_ = *env;
        ((*interface).$version.$name)(env $(, $args)*)
    }};
}

/// Calls a Env function, then checks for a pending exception
///
/// This only checks for an exception, it doesn't clear the exception and so the
/// exception will be thrown if the native code returns to the JVM.
///
/// Returns `Err` if there is a pending exception after the call.
macro_rules! jni_call_check_ex {
    ( $jnienv:expr, $version:tt, $name:ident $(, $args:expr )* ) => ({
        let ret = jni_call_unchecked!($jnienv, $version, $name $(, $args)*);
        if $jnienv.exception_check() {
            Err(()) // Simplified error for mock
        } else {
            Ok(ret)
        }
    })
}

// ---- END: copy of JNI call macros from src/macros.rs ----

/// Map primitive type to internal descriptor code (e.g., jint -> "I")
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
macro_rules! __java_name_to_internal {
    (. $outer:ident $( :: $inner:ident )* ) => {
        concat!( stringify!($outer) $(, "$", stringify!($inner) )* )
    };
    ($first:ident . $($rest:tt)+) => { __java_name_to_internal!(@accum [$first] $($rest)+) };
    (@accum [$($pkg:ident)+] $next:ident . $($more:tt)+) => {
        __java_name_to_internal!(@accum [$($pkg)+ $next] $($more)+)
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
macro_rules! __java_obj_to_internal {
    ( $first:ident . $($rest:tt)+ ) => { concat!("L", __java_name_to_internal!($first . $($rest)+), ";") };
    ( . $outer:ident $( :: $inner:ident )* ) => { concat!("L", __java_name_to_internal!(. $outer $( :: $inner )*), ";") };
}

/// Maps a Java array element type to its internal descriptor form.
macro_rules! __java_elem_to_internal {
    // Java object element
    ( $first:ident . $($rest:tt)+ ) => { __java_obj_to_internal!($first . $($rest)+) };
    ( . $outer:ident $( :: $inner:ident )* ) => { __java_obj_to_internal!(. $outer $( :: $inner )*) };
    // Primitive element (canonical)
    ( jboolean ) => { __prim_to_internal!(jboolean) };
    ( jbyte    ) => { __prim_to_internal!(jbyte)    };
    ( jchar    ) => { __prim_to_internal!(jchar)    };
    ( jshort   ) => { __prim_to_internal!(jshort)   };
    ( jint     ) => { __prim_to_internal!(jint)     };
    ( jlong    ) => { __prim_to_internal!(jlong)    };
    ( jfloat   ) => { __prim_to_internal!(jfloat)   };
    ( jdouble  ) => { __prim_to_internal!(jdouble)  };
    // Primitive aliases
    ( boolean  ) => { __prim_to_internal!(jboolean) };
    ( byte     ) => { __prim_to_internal!(jbyte)    };
    ( char     ) => { __prim_to_internal!(jchar)    };
    ( short    ) => { __prim_to_internal!(jshort)   };
    ( int      ) => { __prim_to_internal!(jint)     };
    ( long     ) => { __prim_to_internal!(jlong)    };
    ( float    ) => { __prim_to_internal!(jfloat)   };
    ( double   ) => { __prim_to_internal!(jdouble)  };
}

/// Maps a normalized Java array type (such as `[jint]` or `[java.lang.String]`) to its internal
/// descriptor form `[I` or `[Ljava/lang/String;`
macro_rules! __java_array_to_internal {
    ( [ [ $($inner:tt)+ ] ] ) => { concat!("[", __java_array_to_internal!([ $($inner)+ ]) ) };
    ( [ $($inner:tt)+ ] ) => { concat!("[", __java_elem_to_internal!( $($inner)+ )) };
}

/// Maps normalized Java types to their internal descriptor forms.
///
/// Rust types trigger a compile-time error since they are not supported in literal signatures.
macro_rules! __java_type_to_internal_literal {
    // prim(...)
    ( prim ( $p:ident ) ) => { __prim_to_internal!($p) };

    // obj(...), including array element syntax: obj([ ... ])
    ( obj ( [ $($inner:tt)+ ] ) as rust ( $as_ty:ty ) ) => { __java_array_to_internal!([ $($inner)+ ]) };
    ( obj ( $first:ident . $($rest:tt)+ ) as rust ( $as_ty:ty ) ) => { __java_obj_to_internal!($first . $($rest)+) };
    ( obj ( . $outer:ident $( :: $inner:ident )* ) as rust ( $as_ty:ty ) ) => { __java_obj_to_internal!(. $outer $( :: $inner )*) };

    // The macros should never try to expand a literal signature with a Rust type
    ( rust ( $($r:tt)+ ) ) => { compile_error!("BUG: Rust types are not supported in literal signatures") };
}

/// Maps normalized Java or Rust types to their internal descriptor forms.
///
/// Rust types are dynamically mapped to their class names via the `Reference::class_name()` method.
macro_rules! __any_type_to_internal_owned {
    // rust(...) is only used in formatted signatures
    ( rust ( $($r:tt)+ ) ) => { &<$($r)+ as $crate::refs::Reference>::class_name() };

    // prim | obj
    ( $($raw:tt)+ ) => {
        __java_type_to_internal_literal!( $($raw)+ )
    };
}

// Build the default Rust wrapper type for a Java array of arbitrary dimension.
// Base cases map a 1D array; recursion wraps further dimensions in JObjectArray<...>.
macro_rules! __jsig_default_rust_type_for_java_array_elem {
    // recursion: more dimensions
    ( [ [ $($inner:tt)+ ] ] ) => {
        JObjectArray< __jsig_default_rust_type_for_java_array_elem!( [ $($inner)+ ] ) >
    };

    // base: 1D primitive arrays
    ( [ jboolean ] ) => { JPrimitiveArray<$crate::sys::jboolean> };
    ( [ jbyte    ] ) => { JPrimitiveArray<$crate::sys::jbyte>    };
    ( [ jchar    ] ) => { JPrimitiveArray<$crate::sys::jchar>    };
    ( [ jshort   ] ) => { JPrimitiveArray<$crate::sys::jshort>   };
    ( [ jint     ] ) => { JPrimitiveArray<$crate::sys::jint>     };
    ( [ jlong    ] ) => { JPrimitiveArray<$crate::sys::jlong>    };
    ( [ jfloat   ] ) => { JPrimitiveArray<$crate::sys::jfloat>   };
    ( [ jdouble  ] ) => { JPrimitiveArray<$crate::sys::jdouble>  };
    ( [ boolean  ] ) => { JPrimitiveArray<$crate::sys::jboolean> };
    ( [ byte     ] ) => { JPrimitiveArray<$crate::sys::jbyte>    };
    ( [ char     ] ) => { JPrimitiveArray<$crate::sys::jchar>    };
    ( [ short    ] ) => { JPrimitiveArray<$crate::sys::jshort>   };
    ( [ int      ] ) => { JPrimitiveArray<$crate::sys::jint>     };
    ( [ long     ] ) => { JPrimitiveArray<$crate::sys::jlong>    };
    ( [ float    ] ) => { JPrimitiveArray<$crate::sys::jfloat>   };
    ( [ double   ] ) => { JPrimitiveArray<$crate::sys::jdouble>  };

    // base: 1D object arrays (named package or default package)
    ( [ $first:ident . $($rest:tt)+ ] ) => { $crate::objects::JObjectArray<$crate::objects::JObject> };
    ( [ . $outer:ident $( :: $inner:ident )* ] ) => { $crate::objects::JObjectArray<$crate::objects::JObject> };
}

/// Normalize one raw type and immediately invoke a callback macro with the normalized form.
///
/// # Normalized shapes:
///   - Primitive types: `prim( jint )`
///   - Java types, with cast: `obj( <java.name or .Default::Inner> )` as `rust( <Ty> )` (default Ty = JObject/JObjectArray if no `as`)
///   - Rust `Reference` types and `&[]` arrays: `rust( <Ty> )`
///
/// # Usage:
///
///   __jsig_normalize_type_then!(CB, ( <raw-type> ) [, extra tokens... ])
///
/// # Calls:
///
///   CB!( <normalized-type> [, extra tokens...] )
///
/// # Normalization examples:
///
///   - `jint` -> `prim(jint)`
///   - `int`  -> `prim(jint)`
///   - `java.lang.String` -> `obj(java.lang.String)` as `rust(JObject)`
///   - `[java.lang.String]` -> `obj([java.lang.String])` as `rust(JObjectArray<JObject>)`
///   - `java.lang.String[]` -> `obj([java.lang.String])` as `rust(JObjectArray<JObject>)`
///   - `jint[]` -> `obj([jint])` as `rust(JPrimitiveArray<jint>)`
///   - `[jint]` -> `obj([jint])` as `rust(JPrimitiveArray<jint>)`
///   - `[[jint]]` -> `obj([[jint]])` as `rust(JObjectArray<JPrimitiveArray<jint>>)`
///   - `jint[][]` -> `obj([[jint]])` as `rust(JObjectArray<JPrimitiveArray<jint>>)`
///   - `.NoPackage` -> `obj(NoPackage)` as `rust(JObject)`
///   - `.NoPackage` as `JString` -> `obj(NoPackage)` as `rust(JString)`
///   - `java.lang.String` as `JString` -> `obj(java.lang.String)` as `rust(JString)`
///   - `&JString` -> `rust(JString)`
///   - `&[JString]` -> `rust(JObjectArray<JString>)`
///   - `&[[JString]]` -> `rust(JObjectArray<JObjectArray<JString>>)`
///   - `&[jint]` -> `rust(JPrimitiveArray<jint>)`
macro_rules! __jsig_normalize_type_then {
    // ----- Rust reference arrays -----
    ( $cb:tt, ( & [ [ $($inner:tt)+ ] ] ) $(, $($pass:tt)* )? ) => {
        __jsig_normalize_type_then!{@objarr $cb, () (), [ [ $($inner)+ ] ] $(, $($pass)* )? }
    };
    ( $cb:tt, ( & [ $ty:path ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( JObjectArray<$ty> ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( & [ & $ty:path ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( JObjectArray<$ty> ) ) $(, $($pass)* )? }
    };
    // & [primitive]
    ( $cb:tt, ( & [ jboolean ] ) $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jboolean> ) ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ boolean ] )  $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jboolean> ) ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ jbyte ] )    $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jbyte> )    ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ byte ] )     $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jbyte> )    ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ jchar ] )    $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jchar> )    ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ char ] )     $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jchar> )    ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ jshort ] )   $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jshort> )   ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ short ] )    $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jshort> )   ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ jint ] )     $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jint> )     ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ int ] )      $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jint> )     ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ jlong ] )    $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jlong> )    ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ long ] )     $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jlong> )    ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ jfloat ] )   $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jfloat> )   ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ float ] )    $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jfloat> )   ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ jdouble ] )  $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jdouble> )  ) $(, $($pass)* )? } };
    ( $cb:tt, ( & [ double ] )   $(, $($pass:tt)* )? ) => { $cb!{ ( rust( JPrimitiveArray<jdouble> )  ) $(, $($pass)* )? } };

    // Handle multi-dimensional primitive arrays by the &-array path
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ jboolean ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jboolean> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ boolean ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jboolean> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ jbyte ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jbyte> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ byte ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jbyte> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ jchar ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jchar> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ char ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jchar> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ jshort ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jshort> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ short ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jshort> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ jint ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jint> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ int ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jint> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ jlong ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jlong> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ long ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jlong> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ jfloat ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jfloat> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ float ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jfloat> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ jdouble ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jdouble> $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ double ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* JPrimitiveArray<$crate::sys::jdouble> $($close)* ) ) $(, $($pass)* )? }
    };
    // Object array recursion for &[[...]]
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ [ $($inner:tt)+ ] ] $(, $($pass:tt)* )? ) => {
        __jsig_normalize_type_then!{@objarr $cb, ( $($open)* JObjectArray< ) ( > $($close)* ), [ $($inner)+ ] $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ $ty:path ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* $ty $($close)* ) ) $(, $($pass)* )? }
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ & $ty:path ] $(, $($pass:tt)* )? ) => {
        $cb!{ ( rust( $($open)* $ty $($close)* ) ) $(, $($pass)* )? }
    };

    // ----- Java-style arrays (non-Rust), N-dimensional -----

    // Bracket form with explicit `as` (any dimension)
    ( $cb:tt, ( [ $($arr:tt)+ ] as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ $($arr)+ ] ) as rust( $as_ty ) ) $(, $($pass)* )? }
    };
    // Bracket form without `as` (any dimension)
    ( $cb:tt, ( [ $($arr:tt)+ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ $($arr)+ ] ) as rust( __jsig_default_rust_type_for_java_array_elem!([ $($arr)+ ]) ) ) $(, $($pass)* )? }
    };

    // Multi-dimensional suffix forms with explicit as clause
    ( $cb:tt, ( $first:ident $( . $seg:ident )+ $( :: $inner:ident )* [ ] [ ] [ ] as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ [ $first $( . $seg )+ $( :: $inner )* ] ] ] ) as rust( $as_ty ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $first:ident $( . $seg:ident )+ $( :: $inner:ident )* [ ] [ ] as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ $first $( . $seg )+ $( :: $inner )* ] ] ) as rust( $as_ty ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $first:ident $( . $seg:ident )+ $( :: $inner:ident )* [ ] as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ $first $( . $seg )+ $( :: $inner )* ] ) as rust( $as_ty ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* [ ] [ ] [ ] as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ [ . $outer $( :: $inner )* ] ] ] ) as rust( $as_ty ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* [ ] [ ] as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ . $outer $( :: $inner )* ] ] ) as rust( $as_ty ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* [ ] as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ . $outer $( :: $inner )* ] ) as rust( $as_ty ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $p:ident [ ] [ ] [ ] as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ [ $p ] ] ] ) as rust( $as_ty ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $p:ident [ ] [ ] as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ $p ] ] ) as rust( $as_ty ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $p:ident [ ] as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ $p ] ) as rust( $as_ty ) ) $(, $($pass)* )? }
    };
    // Multi-dimensional suffix forms (without explicit as clause)
    ( $cb:tt, ( $first:ident $( . $seg:ident )+ $( :: $inner:ident )* [ ] [ ] [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ [ $first $( . $seg )+ $( :: $inner )* ] ] ] ) as rust( __jsig_default_rust_type_for_java_array_elem!([ [ [ $first $( . $seg )+ $( :: $inner )* ] ] ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $first:ident $( . $seg:ident )+ $( :: $inner:ident )* [ ] [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ $first $( . $seg )+ $( :: $inner )* ] ] ) as rust( __jsig_default_rust_type_for_java_array_elem!([ [ $first $( . $seg )+ $( :: $inner )* ] ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $first:ident $( . $seg:ident )+ $( :: $inner:ident )* [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ $first $( . $seg )+ $( :: $inner )* ] ) as rust( __jsig_default_rust_type_for_java_array_elem!([ $first $( . $seg )+ $( :: $inner )* ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* [ ] [ ] [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ [ . $outer $( :: $inner )* ] ] ] ) as rust( __jsig_default_rust_type_for_java_array_elem!([ [ [ . $outer $( :: $inner )* ] ] ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* [ ] [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ . $outer $( :: $inner )* ] ] ) as rust( __jsig_default_rust_type_for_java_array_elem!([ [ . $outer $( :: $inner )* ] ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ . $outer $( :: $inner )* ] ) as rust( __jsig_default_rust_type_for_java_array_elem!([ . $outer $( :: $inner )* ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $p:ident [ ] [ ] [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ [ $p ] ] ] ) as rust( __jsig_default_rust_type_for_java_array_elem!([ [ [ $p ] ] ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $p:ident [ ] [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ $p ] ] ) as rust( __jsig_default_rust_type_for_java_array_elem!([ [ $p ] ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $p:ident [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ $p ] ) as rust( __jsig_default_rust_type_for_java_array_elem!([ $p ]) ) ) $(, $($pass)* )? }
    };


    // ----- Rust reference type like &JObject, &JString etc -----
    ( $cb:tt, ( & $rust:ty ) $(, $($pass:tt)* )? ) => { $cb!{ ( rust( $rust ) ) $(, $($pass)* )? } };

    // ----- Java object type -----
    ( $cb:tt, ( $first:ident $( . $seg:ident )+ $( :: $inner:ident )* as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
       $cb!{ ( obj( $first $( . $seg )+ $( :: $inner )* ) as rust( $as_ty ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $first:ident $( . $seg:ident )+ $( :: $inner:ident )* ) $(, $($pass:tt)* )? ) => {
      $cb!{ ( obj( $first $( . $seg )+ $( :: $inner )* ) as rust( $crate::objects::JObject ) ) $(, $($pass)* )? }
    };

    // ----- Java object type (default package) -----
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( . $outer $( :: $inner )* ) as rust( $as_ty ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( . $outer $( :: $inner )* ) as rust( $crate::objects::JObject ) ) $(, $($pass)* )? }
    };

    // ----- Primitives (canonicalize) -----
    ( $cb:tt, ( void )     $(, $($pass:tt)* )? ) => { $cb!{ ( prim( void )     ) $(, $($pass)* )? } };
    ( $cb:tt, ( jboolean ) $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jboolean ) ) $(, $($pass)* )? } };
    ( $cb:tt, ( boolean )  $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jboolean ) ) $(, $($pass)* )? } };
    ( $cb:tt, ( jbyte )    $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jbyte )    ) $(, $($pass)* )? } };
    ( $cb:tt, ( byte )     $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jbyte )    ) $(, $($pass)* )? } };
    ( $cb:tt, ( jchar )    $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jchar )    ) $(, $($pass)* )? } };
    ( $cb:tt, ( char )     $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jchar )    ) $(, $($pass)* )? } };
    ( $cb:tt, ( jshort )   $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jshort )   ) $(, $($pass)* )? } };
    ( $cb:tt, ( short )    $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jshort )   ) $(, $($pass)* )? } };
    ( $cb:tt, ( jint )     $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jint )     ) $(, $($pass)* )? } };
    ( $cb:tt, ( int )      $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jint )     ) $(, $($pass)* )? } };
    ( $cb:tt, ( jlong )    $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jlong )    ) $(, $($pass)* )? } };
    ( $cb:tt, ( long )     $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jlong )    ) $(, $($pass)* )? } };
    ( $cb:tt, ( jfloat )   $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jfloat )   ) $(, $($pass)* )? } };
    ( $cb:tt, ( float )    $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jfloat )   ) $(, $($pass)* )? } };
    ( $cb:tt, ( jdouble )  $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jdouble )  ) $(, $($pass)* )? } };
    ( $cb:tt, ( double )   $(, $($pass:tt)* )? ) => { $cb!{ ( prim( jdouble )  ) $(, $($pass)* )? } };

    // ----- Rust type: JString, JObject, etc. (must come after primitives) -----
    ( $cb:tt, ( $rust:ty ) $(, $($pass:tt)* )? ) => { $cb!{ ( rust( $rust ) ) $(, $($pass)* )? } };

    // ----- Already-normalized (idempotent) -----
    ( $cb:tt, ( prim ( $($p:tt)+ ) ) $(, $($pass:tt)* )? ) => { $cb!( ( prim( $($p)+ ) ) $(, $($pass)* )? ) };
    ( $cb:tt, ( obj ( $($j:tt)+ ) as rust ( $as:ty ) ) $(, $($pass:tt)* )? ) => { $cb!( ( obj( $($j)+ ) as rust( $as ) ) $(, $($pass)* )? ) };
    ( $cb:tt, ( rust ( $as:ty ) ) $(, $($pass:tt)* )? ) => { $cb!( ( rust( $as ) ) $(, $($pass)* )? ) };
}

/// Shim that routes from normalization macros to the real callback, for items with a trailing
/// semicolon
macro_rules! __then_item {
    ( ( $($norm:tt)+ ), $real_cb:tt $(, $rest:tt )* ) => {
        $real_cb!( ( $($norm)+ ) $(, $rest )* );
    };
}
/// Shim that routes from normalization macros to the real callback, for expressions without a
/// trailing semicolon
macro_rules! __then_expr {
    ( ( $($norm:tt)+ ), $real_cb:tt $(, $rest:tt )* ) => {
        $real_cb!( ( $($norm)+ ) $(, $rest )* )
    };
}

/// Normalize a return type, then invoke a callback macro that expects a normalized type
///
/// # Usage
///
///  - `__jsig_normalize_ret_then!(@expr CB, ( RetTy ) [, extra tokens...])`
///  - `__jsig_normalize_ret_then!(@item CB, ( RetTy ) [, extra tokens...])`
///
/// Add @expr | @item to control whether the callback emits an expression or items.
macro_rules! __jsig_normalize_ret_then {
    ( @ expr $cb:tt, ( $($ret:tt)+ ) $(, $($pass:tt)* )? ) => {
        __jsig_normalize_type_then!( __then_expr, ( $($ret)+ ) $(, $cb, $($pass)* )? )
    };
    ( @ item $cb:tt, ( $($ret:tt)+ ) $(, $($pass:tt)* )? ) => {
        __jsig_normalize_type_then!( __then_item, ( $($ret)+ ) $(, $cb, $($pass)* )? );
    };
}

/// Helper that consumes a Java-style suffix-array [] without introducing tt/',' ambiguity.
macro_rules! __jsig_normalize_suffix_array_then {
    // More [] → accumulate and continue
    ( $push:tt, $cb:tt, @ $mode:ident,
      ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident,
      base( $($base:tt)+ ), dims( $($dims:tt)* ),
      [ ] $($after:tt)*
    ) => {
        __jsig_normalize_suffix_array_then!(
            $push, $cb, @ $mode,
            ( $($acc)* ), ( $($pass)* ), $n,
            base( $($base)+ ), dims( $($dims)* [ ] ),
            $($after)*
        )
    };

    // as Ty, then trailing comma + rest
    ( $push:tt, $cb:tt, @ $mode:ident,
      ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident,
      base( $($base:tt)+ ), dims( $($dims:tt)* ),
      as $as_ty:ty , $($rest:tt)*
    ) => {
        __jsig_normalize_type_then!{
            $push, ( $($base)+ [ ] $($dims)* as $as_ty ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n, $($rest)*
        }
    };
    // as Ty at end-of-list
    ( $push:tt, $cb:tt, @ $mode:ident,
      ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident,
      base( $($base:tt)+ ), dims( $($dims:tt)* ),
      as $as_ty:ty
    ) => {
        __jsig_normalize_type_then!{
            $push, ( $($base)+ [ ] $($dims)* as $as_ty ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n
        }
    };

    // Comma terminator + rest
    ( $push:tt, $cb:tt, @ $mode:ident,
      ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident,
      base( $($base:tt)+ ), dims( $($dims:tt)* ),
      , $($rest:tt)*
    ) => {
        __jsig_normalize_type_then!{
            $push, ( $($base)+ [ ] $($dims)* ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n, $($rest)*
        }
    };

    // End-of-list terminator (when there are trailing tokens that need to be passed through)
    ( $push:tt, $cb:tt, @ $mode:ident,
      ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident,
      base( $($base:tt)+ ), dims( $($dims:tt)* )
    ) => {
        __jsig_normalize_type_then!{
            $push, ( $($base)+ [ ] $($dims)* ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n
        }
    };

    // End-of-list terminator (when called with empty $($after)* - no trailing tokens)
    ( $push:tt, $cb:tt, @ $mode:ident,
      ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident,
      base( $($base:tt)+ ), dims( $($dims:tt)* ),
    ) => {
        __jsig_normalize_type_then!{
            $push, ( $($base)+ [ ] $($dims)* ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n
        }
    };
}

/// Push one normalized arg into accumulator and continue munching.
macro_rules! __jsig_normalize_args_then__push {
    ( ( $($norm:tt)+ ), $cb:tt, @ $mode:ident, ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident, $($rest:tt)* ) => {
        __jsig_normalize_args_then__munch!{ @ $mode $cb, ( $($acc)* $n: $($norm)+ , ), ( $($pass)* ), $($rest)* }
    };
    ( ( $($norm:tt)+ ), $cb:tt, @ $mode:ident, ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident ) => {
        __jsig_normalize_args_then__munch!{ @ $mode $cb, ( $($acc)* $n: $($norm)+ ), ( $($pass)* ) }
    };
}

/// Internal muncher that uses __jsig_normalize_type_then per argument type and accumulates a normalized list.
///
/// Signature carries a mode (@ with_pass | @ no_pass) and a "( $pass )" group that is forwarded unchanged.
macro_rules! __jsig_normalize_args_then__munch {
    // End of list (with pass)
    ( @ with_pass $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)+ ) ) => {
        $cb!{ ( $($acc)* ), $($pass)+ }
    };
    // End of list (no pass)
    ( @ no_pass $cb:tt, ( $($acc:tt)* ), () ) => {
        $cb!( ( $($acc)* ) )
    };

    // ---------- Recognize specific, non-ambiguous forms ----------

    // &[[...]]
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : & [ [ $($inner:tt)+ ] ]
      $(, $($rest:tt)*)?
    ) => {
        __jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, ( & [ [ $($inner)+ ] ] ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // &[...]
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : & [ $($inner:tt)+ ]
      $(, $($rest:tt)*)?
    ) => {
        __jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, ( & [ $($inner)+ ] ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // &Path
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : & $rust:ty
      $(, $($rest:tt)*)?
    ) => {
        __jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, ( & $rust ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // Bracket arrays: [ ... ] as Ty
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : [ $($arr:tt)+ ] as $as_ty:ty
      $(, $($rest:tt)*)?
    ) => {
        __jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, ( [ $($arr)+ ] as $as_ty ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // Bracket arrays: [ ... ]
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : [ $($arr:tt)+ ]
      $(, $($rest:tt)*)?
    ) => {
        __jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, ( [ $($arr)+ ] ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // Java name with explicit `as`
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : $first:ident $( . $seg:ident )+ $( :: $inner:ident )* as $as_ty:ty
      $(, $($rest:tt)*)?
    ) => {
        __jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, ( $first $( . $seg )+ $( :: $inner )* as $as_ty ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // Java name (no as, no suffix)
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : $first:ident $( . $seg:ident )+ $( :: $inner:ident )*
      $(, $($rest:tt)*)?
    ) => {
        __jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, ( $first $( . $seg )+ $( :: $inner )* ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // Default-package with explicit `as`
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : . $outer:ident $( :: $inner:ident )* as $as_ty:ty
      $(, $($rest:tt)*)?
    ) => {
        __jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, ( . $outer $( :: $inner )* as $as_ty ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // Default-package (no as, no suffix)
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : . $outer:ident $( :: $inner:ident )*
      $(, $($rest:tt)*)?
    ) => {
        __jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, ( . $outer $( :: $inner )* ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // Primitive
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : $p:ident
      $(, $($rest:tt)*)?
    ) => {
        __jsig_normalize_type_then!{
            __jsig_normalize_args_then__push, ( $p ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // ---------- Java-style suffix arrays …[] ----------

    // java.name[]… (suffix form) → delegate to suffix eater
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident :
      $first:ident $( . $seg:ident )+ $( :: $inner:ident )*
      [ ] $($after:tt)*
    ) => {
        __jsig_normalize_suffix_array_then!{
            __jsig_normalize_args_then__push, $cb, @ $mode,
            ( $($acc)* ), ( $($pass)* ), $n,
            base( $first $( . $seg )+ $( :: $inner )* ), dims(),
            $($after)*
        }
    };

    // .Default::Inner[]… (suffix form)
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident :
      . $outer:ident $( :: $inner:ident )*
      [ ] $($after:tt)*
    ) => {
        __jsig_normalize_suffix_array_then!(
            __jsig_normalize_args_then__push, $cb, @ $mode,
            ( $($acc)* ), ( $($pass)* ), $n,
            base( . $outer $( :: $inner )* ), dims(),
            $($after)*
        )
    };

    // primitive[] at end of argument list (no tokens after [])
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident :
      $p:ident
      [ ]
    ) => {
        __jsig_normalize_suffix_array_then!{
            __jsig_normalize_args_then__push, $cb, @ $mode,
            ( $($acc)* ), ( $($pass)* ), $n,
            base( $p ), dims()
        }
    };

    // primitive[]… (suffix form) with more tokens after []
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident :
      $p:ident
      [ ] $($after:tt)+
    ) => {
        __jsig_normalize_suffix_array_then!{
            __jsig_normalize_args_then__push, $cb, @ $mode,
            ( $($acc)* ), ( $($pass)* ), $n,
            base( $p ), dims(),
            $($after)+
        }
    };
}

/// Normalize an args list, then invoke a callback macro that expects normalized args
///
/// # Usage
///
///  - `__jsig_normalize_args_then!(@expr CB, ( a: TyA, b: TyB, ... ) [, extra tokens...])`
///  - `__jsig_normalize_args_then!(@item CB, ( a: TyA, b: TyB, ... ) [, extra tokens...])`
///
/// Add @expr | @item to control whether the callback emits an expression or items.
macro_rules! __jsig_normalize_args_then {
    // Explicit context + pass
    ( @ expr $cb:tt, ( $($args:tt)* ), $($pass:tt)+ ) => {
        __jsig_normalize_args_then__munch!( @ with_pass __then_expr, (), ( $cb, $($pass)+ ), $($args)* )
    };
    ( @ item $cb:tt, ( $($args:tt)* ), $($pass:tt)+ ) => {
        __jsig_normalize_args_then__munch!( @ with_pass __then_item, (), ( $cb, $($pass)+ ), $($args)* );
    };
    // Explicit context, no pass
    ( @ expr $cb:tt, ( $($args:tt)* ) ) => {
        __jsig_normalize_args_then__munch!( @ no_pass __then_expr, (), ( $cb ), $($args)* )
    };
    ( @ item $cb:tt, ( $($args:tt)* ) ) => {
        __jsig_normalize_args_then__munch!( @ no_pass __then_item, (), ( $cb ), $($args)* );
    };
}

/// Shim that routes from return type normalization to the real callback, accepting both normalized args and ret
macro_rules! __then_args_ret_expr {
    ( ( $($norm_ret:tt)+ ), $real_cb:tt, ( $($norm_args:tt)* ) $(, $rest:tt )* ) => {
        $real_cb!( ( $($norm_args)* ), ( $($norm_ret)+ ) $(, $rest )* )
    };
}
/// Shim that routes from return type normalization to the real callback, accepting both normalized args and ret
macro_rules! __then_args_ret_item {
    ( ( $($norm_ret:tt)+ ), $real_cb:tt, ( $($norm_args:tt)* ) $(, $rest:tt )* ) => {
        $real_cb!( ( $($norm_args)* ), ( $($norm_ret)+ ) $(, $rest )* );
    };
}

/// Shim that forwards normalized args to return type normalization
macro_rules! __args_then_ret_expr {
    ( ( $($norm_args:tt)* ), $real_cb:tt, ( $($raw_ret:tt)+ ) $(, $rest:tt )* ) => {
        __jsig_normalize_ret_then!( @ expr __then_args_ret_expr, ( $($raw_ret)+ ), $real_cb, ( $($norm_args)* ) $(, $rest )* )
    };
}
macro_rules! __args_then_ret_item {
    ( ( $($norm_args:tt)* ), $real_cb:tt, ( $($raw_ret:tt)+ ) $(, $rest:tt )* ) => {
        __jsig_normalize_ret_then!( @ item __then_args_ret_item, ( $($raw_ret)+ ), $real_cb, ( $($norm_args)* ) $(, $rest )* );
    };
}

/// Normalize both arguments and return type, then invoke a callback macro that expects both
///
/// # Usage
///
///  - `__jsig_normalize_args_ret_then!(@expr CB, ( a: TyA, b: TyB, ... ), ( RetTy ) [, extra tokens...])`
///  - `__jsig_normalize_args_ret_then!(@item CB, ( a: TyA, b: TyB, ... ), ( RetTy ) [, extra tokens...])`
///
/// Add @expr | @item to control whether the callback emits an expression or items.
///
/// # Callback signature
///
/// The callback will be invoked as: `CB!( ( normalized_args... ), ( normalized_ret ) [, extra tokens...] )`
macro_rules! __jsig_normalize_args_ret_then {
    ( @ expr $cb:tt, ( $($args:tt)* ), ( $($ret:tt)+ ), $($pass:tt)* ) => {
        __jsig_normalize_args_then!( @ expr __args_then_ret_expr, ( $($args)* ), $cb, ( $($ret)+ ), $($pass)* )
    };
    ( @ item $cb:tt, ( $($args:tt)* ), ( $($ret:tt)+ ), $($pass:tt)* ) => {
        __jsig_normalize_args_then!( @ item __args_then_ret_item, ( $($args)* ), $cb, ( $($ret)+ ), $($pass)* );
    };
    ( @ expr $cb:tt, ( $($args:tt)* ), ( $($ret:tt)+ ) ) => {
        __jsig_normalize_args_then!( @ expr __args_then_ret_expr, ( $($args)* ), $cb, ( $($ret)+ ) )
    };
    ( @ item $cb:tt, ( $($args:tt)* ), ( $($ret:tt)+ ) ) => {
        __jsig_normalize_args_then!( @ item __args_then_ret_item, ( $($args)* ), $cb, ( $($ret)+ ) );
    };
}

/// Compose a JNI signature as a string literal
///
/// Returns a `Cow::Borrowed`
macro_rules! __jsig_emit_sig_literal {
    ( ( $( $n:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* )
      -> ( $rk:ident ( $($rt:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? )
    ) => {
        std::borrow::Cow::Borrowed(concat!(
            "(",
            $( __java_type_to_internal_literal!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ) ),*,
            ")",
            __java_type_to_internal_literal!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? ),
            "\0"
        ))
    };
}

/// Compose a JNI signature dynamically into a `String`
///
/// Returns a `Cow::Owned`
macro_rules! __jsig_emit_sig_dynamic_owned {
    ( ( $( $n:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* )
      -> ( $rk:ident ( $($rt:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? )
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
            $( extend_with_internal(&mut buf, __any_type_to_internal_owned!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? )); )*
            buf.push(')');
            extend_with_internal(&mut buf, __any_type_to_internal_owned!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? ));
            buf.push('\0');
            std::borrow::Cow::Owned(buf)
        }
    };
}

/// Assuming the args have already been checked for rust(...) types, check the return type
/// and determine whether to emit a literal (Cow::Borrowed) or dynamic (Cow::Owned) signature
macro_rules! __jsig_emit_sig_cow__check_ret_then {
    // rust(...) return type, use dynamic callback
    ( ( rust ( $($ret_type:tt)* ) ), $literal_cb:tt, $dynamic_cb:tt, ( $($orig_args:tt)* ) ) => {
        $dynamic_cb!( ( $($orig_args)* ) -> ( rust( $($ret_type)* ) ) )
    };
    // Non-rust return type, use literal callback
    ( ( $ret_kind:ident ( $($ret_type:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? ), $literal_cb:tt, $dynamic_cb:tt, ( $($orig_args:tt)* ) ) => {
        $literal_cb!( ( $($orig_args)* ) -> ( $ret_kind( $($ret_type)* ) $( as rust( $($ret_as)+ ) )? ) )
    };
}

/// Helper to recursively check if any type is rust(...) before chaining to return type check
macro_rules! __jsig_emit_sig_cow__check_args_then {
    // Base case: no more args, check return type with original args preserved
    ( (), ( $($nret:tt)* ), $literal_cb:tt, $dynamic_cb:tt, ( $($orig_args:tt)* ) ) => {
        __jsig_emit_sig_cow__check_ret_then!( ( $($nret)* ), $literal_cb, $dynamic_cb, ( $($orig_args)* ) )
    };
    // Recursive case: check first arg, continue with rest
    ( ( $first_name:ident : rust ( $($first_type:tt)* ) , $($rest_args:tt)* ), ( $($nret:tt)* ), $literal_cb:tt, $dynamic_cb:tt, ( $($orig_args:tt)* ) ) => {
        // Found rust(...) type, use dynamic callback
        $dynamic_cb!( ( $($orig_args)* ) -> ( $($nret)* ) )
    };
    // Recursive case: non-rust arg, continue checking
    ( ( $first_name:ident : $first_kind:ident ( $($first_type:tt)* ) $( as rust ( $($first_as:tt)+ ) )? , $($rest_args:tt)* ), ( $($nret:tt)* ), $literal_cb:tt, $dynamic_cb:tt, ( $($orig_args:tt)* ) ) => {
        __jsig_emit_sig_cow__check_args_then!( ( $($rest_args)* ), ( $($nret)* ), $literal_cb, $dynamic_cb, ( $($orig_args)* ) )
    };
    // Handle single arg without trailing comma
    ( ( $first_name:ident : rust ( $($first_type:tt)* ) ), ( $($nret:tt)* ), $literal_cb:tt, $dynamic_cb:tt, ( $($orig_args:tt)* ) ) => {
        // Found rust(...) type, use dynamic callback
        $dynamic_cb!( ( $($orig_args)* ) -> ( $($nret)* ) )
    };
    ( ( $first_name:ident : $first_kind:ident ( $($first_type:tt)* ) $( as rust ( $($first_as:tt)+ ) )? ), ( $($nret:tt)* ), $literal_cb:tt, $dynamic_cb:tt, ( $($orig_args:tt)* ) ) => {
        __jsig_emit_sig_cow__check_args_then!( ( ), ( $($nret)* ), $literal_cb, $dynamic_cb, ( $($orig_args)* ) )
    };
}

/// Given normalized args and ret, emit a `Cow<str>` signature either based on a `Cow::Borrowed` string
/// literal or a `Cow::Owned String` built at runtime.
macro_rules! __jsig_emit_sig_cow {
    ( ( $($norm_args:tt)* ), ( $($norm_ret:tt)+ ) ) => {
        __jsig_emit_sig_cow__check_args_then!( ( $($norm_args)* ), ( $($norm_ret)* ), __jsig_emit_sig_literal, __jsig_emit_sig_dynamic_owned, ( $($norm_args)* ) )
    };
}

/// Get a raw `jobject` handle from a `Reference` Rust wrapper
macro_rules! __jsig_obj_reference_as_raw {
    ( $ty:ty, $expr:expr ) => {{
        <$ty as $crate::refs::Reference>::as_raw(($expr).as_ref())
    }};
}

/// Maps a normalized argument type to a `[jni::sys::jvalue]`
macro_rules! __jsig_arg_to_jvalue {
    ( $name:ident : prim ( jboolean ) ) => {
        jni::sys::jvalue {
            z: ($name as jni::sys::jboolean),
        }
    };
    ( $name:ident : prim ( jbyte    ) ) => {
        jni::sys::jvalue {
            b: ($name as jni::sys::jbyte),
        }
    };
    ( $name:ident : prim ( jchar    ) ) => {
        jni::sys::jvalue {
            c: ($name as jni::sys::jchar),
        }
    };
    ( $name:ident : prim ( jshort   ) ) => {
        jni::sys::jvalue {
            s: ($name as jni::sys::jshort),
        }
    };
    ( $name:ident : prim ( jint     ) ) => {
        jni::sys::jvalue {
            i: ($name as jni::sys::jint),
        }
    };
    ( $name:ident : prim ( jlong    ) ) => {
        jni::sys::jvalue {
            j: ($name as jni::sys::jlong),
        }
    };
    ( $name:ident : prim ( jfloat   ) ) => {
        jni::sys::jvalue {
            f: ($name as jni::sys::jfloat),
        }
    };
    ( $name:ident : prim ( jdouble  ) ) => {
        jni::sys::jvalue {
            d: ($name as jni::sys::jdouble),
        }
    };

    // Java objects: get the raw `jobject` from the `as rust(...)` `Reference` type
    ( $name:ident : obj  ( $($j:tt)+ ) as rust ( $as_ty:ty ) ) => {
        jni::sys::jvalue {
            l: __jsig_obj_reference_as_raw!($as_ty, $name),
        }
    };

    // Rust `Reference` types
    ( $name:ident : rust ( $as_ty:ty ) ) => {
        jni::sys::jvalue {
            l: __jsig_obj_reference_as_raw!($as_ty, $name),
        }
    };
}

/// Build `[jvalue; N]` from a normalized arg list
macro_rules! __jsig_args_to_jvalue_array {
    ( ( $( $an:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* $(,)? ) ) => {{
        [ $( __jsig_arg_to_jvalue!( $an : $ak($($at)*) $( as rust ( $($aas)+ ) )? ) ),* ]
    }};
}

/// Derive a Rust method argument type from a normalized type
macro_rules! __jgen_emit_rust_method_arg_type {
    ( prim ( void ) ) => {
        compile_error!("'void' is not a valid parameter type");
    };
    ( prim ( $p:ident ) ) => {
        $crate::sys::$p
    };
    ( obj ( $( $j:tt )+ ) as rust ( $as_ty:ty ) ) => {
        impl AsRef<$as_ty>
    };
    ( rust ( $as_ty:ty ) ) => {
        impl AsRef<$as_ty>
    };
}

/// Derive a Rust method return type from a normalized type
macro_rules! __jgen_emit_rust_return_type {
    ( prim ( void ) ) => {
        ()
    };
    ( prim ( $p:ident ) ) => {
        $crate::sys::$p
    };
    ( obj ( $( $j:tt )+ ) as rust ( $as_ty:ty ) ) => {
        $as_ty
    };
    ( rust ( $as_ty:ty ) ) => {
        $as_ty
    };
}

/// Emit a JNI call to a method with the given normalized return type
macro_rules! __jgen_emit_jni_sys_call {
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( void ) )
    ) => {
        unsafe {
            use $crate::refs::Reference as _;
            jni_call_check_ex!(
                $env,
                v1_1,
                CallVoidMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jboolean ) )
    ) => {
        unsafe {
            use $crate::refs::Reference as _;
            jni_call_check_ex!(
                $env,
                v1_1,
                CallBooleanMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jbyte ) )
    ) => {
        unsafe {
            use $crate::refs::Reference as _;
            jni_call_check_ex!(
                $env,
                v1_1,
                CallByteMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jchar ) )
    ) => {
        unsafe {
            use $crate::refs::Reference as _;
            jni_call_check_ex!(
                $env,
                v1_1,
                CallCharMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jshort ) )
    ) => {
        unsafe {
            use $crate::refs::Reference as _;
            jni_call_check_ex!(
                $env,
                v1_1,
                CallShortMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jint ) )
    ) => {
        unsafe {
            use $crate::refs::Reference as _;
            jni_call_check_ex!(
                $env,
                v1_1,
                CallIntMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jlong ) )
    ) => {
        unsafe {
            use $crate::refs::Reference as _;
            jni_call_check_ex!(
                $env,
                v1_1,
                CallLongMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jfloat ) )
    ) => {
        unsafe {
            use $crate::refs::Reference as _;
            jni_call_check_ex!(
                $env,
                v1_1,
                CallFloatMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jdouble ) )
    ) => {
        unsafe {
            use $crate::refs::Reference as _;
            jni_call_check_ex!(
                $env,
                v1_1,
                CallDoubleMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( obj  ( $($rt:tt)+ ) as rust ( $as_ty:ty ) )
    ) => {
        unsafe {
            use $crate::refs::Reference as _;
            let ret_obj: jni::sys::jobject = jni_call_check_ex!(
                $env,
                v1_1,
                CallObjectMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )?;
            <$as_ty>::from_raw($env, ret_obj)
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( rust ( $as_ty:ty ) )
    ) => {
        unsafe {
            use $crate::refs::Reference as _;
            let ret_obj: jni::sys::jobject = jni_call_check_ex!(
                $env,
                v1_1,
                CallObjectMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )?;
            <$as_ty>::from_raw($env, ret_obj)
        }
    };
}

/// Emit a method ID lookup function for the given method name and signature
macro_rules! __jgen_emit_lookup_method_fn {
    (
        $this:path,
        $jname:ident,
        $rname:ident,
        ( $($args:tt)* ),
        ( $($ret:tt)+ )
    ) => {
        paste! {
            fn [<_ $rname _lookup>](env: &mut Env, class: &JClass) -> Result<JMethodID> {
                let sig = __jsig_normalize_args_ret_then!( @expr __jsig_emit_sig_cow, ( $($args)* ), ( $($ret)+ ) );
                env.get_method_id(class, stringify!($jname), &sig)
            }
        }
    };
}

// Final emitter: we now have normalized args and ret; generate the whole fn
macro_rules! __jgen_emit_call_method_fn__with_norm_args_ret {
    (
        $envType:ty,
        $this:path,
        $rname:ident,
        ( $( $an:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* $(,)? ),
        ( $rk:ident ( $($rt:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? )
    ) => {
        paste! {
            pub fn $rname (
                &self,
                env: $envType,
                $( $an: __jgen_emit_rust_method_arg_type!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ) ),*
            ) -> Result< __jgen_emit_rust_return_type!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? ) > {
                let api = [<$this API>]::get(env, &$crate::refs::LoaderContext::None)?;
                let jni_args = __jsig_args_to_jvalue_array!( ( $( $an : $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ),* ) );

                __jgen_emit_jni_sys_call!{
                    env,
                    self,
                    api.[<$rname _method_id>],
                    &jni_args,
                    ( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? )
                }
            }
        }
    };
}

/// A shim before __jgen_emit_call_method_fn__with_norm_args_ret that determines whether &Env or &mut Env is needed
macro_rules! __jgen_emit_call_method_fn__with_norm_args_ret__shim {
    // Primitive return types (including void) use &Env
    (
        ( $( $nargs:tt )* ),
        ( prim ( $($p:tt)+ ) ),
        $this:path,
        $rname:ident
    ) => {
        __jgen_emit_call_method_fn__with_norm_args_ret!( &Env, $this, $rname, ( $( $nargs )* ), ( prim( $($p)+ ) ) );
    };
    // Object return types use &mut Env
    (
        ( $( $nargs:tt )* ),
        ( obj ( $($rt:tt)+ ) as rust ( $($ret_as:tt)+ ) ),
        $this:path,
        $rname:ident
    ) => {
        __jgen_emit_call_method_fn__with_norm_args_ret!( &mut Env, $this, $rname, ( $( $nargs )* ), ( obj( $($rt)+ ) as rust( $($ret_as)+ ) ) );
    };
    // Rust return types use &mut Env
    (
        ( $( $nargs:tt )* ),
        ( rust ( $($ret_as:tt)+ ) ),
        $this:path,
        $rname:ident
    ) => {
        __jgen_emit_call_method_fn__with_norm_args_ret!( &mut Env, $this, $rname, ( $( $nargs )* ), ( rust( $($ret_as)+ ) ) );
    };
}

macro_rules! __jgen_emit_call_method_fn {
    (
        $this:path,
        $rname:ident,
        ( $($args:tt)* ),
        ( $($ret:tt)+ )
    ) => {
        __jsig_normalize_args_ret_then!( @ item __jgen_emit_call_method_fn__with_norm_args_ret__shim, ( $($args)* ), ( $($ret)+ ), $this, $rname );
    };
}

/// Binds a Java method to Rust by emitting a method ID lookup function and a call function.
///
/// # Usage
///
///  jgen_bind_method!(
///      this: JTest,
///      javaMethodName as rust_method_name,
///      sig: (a: &JString, b: java.lang.String as JString, c: jint) -> void
///  );
///
///  # Outputs:
///
///  fn _rust_method_name_lookup(env: &mut Env) -> Result<JMethodID> {
///      let class: &JClass = JTest::lookup_class()?;
///      let sig = std::borrow::Cow::Borrowed("(Ljava/lang/String;Ljava/lang/String;I)V\u{0}");
///      env.get_method_id(class, "javaMethodName", &sig)
///  }
///
///  fn _rust_method_name_call(env: Env, this: &JFoo, method_id: JMethodID, a: impl AsRef<JString>, b: impl AsRef<JString>, c: jint) -> Result<void> {
///      let jni_args = {
///          [
///              jni::sys::jvalue {
///                  l: { <crate::objects::JObject as crate::refs::Reference>::as_raw(a.as_ref()) },
///              },
///              jni::sys::jvalue {
///                  l: { <crate::objects::JObject as crate::refs::Reference>::as_raw(b.as_ref()) },
///              },
///              jni::sys::jvalue {
///                  i: (c as jni::sys::jint),
///              },
///          ]
///      };
///      jni_call_check_ex!(env, v1_1, CallVoidMethodA, this, method_id, jni_args)
///  }
macro_rules! jgen_bind_method {
    (
        this: $this:path,
        $jname:ident as $rname:ident,
        sig: ( $($args:tt)* ) -> $($rty:tt)+
    ) => {

        paste::paste!{
            impl [<$this API>] {
                __jgen_emit_lookup_method_fn!(
                    $this,
                    $jname,
                    $rname,
                    ( $($args)* ),
                    ( $($rty)+ )
                );
            }
            impl $this {
                __jgen_emit_call_method_fn!(
                    $this,
                    $rname,
                    ( $($args)* ),
                    ( $($rty)+ )
                );
            }
        }
    };
}

/// Emits a cacheable method IDs API struct, lookup APIs and call APIs for a given set of methods
///
/// # Usage
///
/// jgen_bind_api!{
///     JTestAPI,
///     JTest,
///     methods = {
///         get_message = { name = "getMessage", sig = () -> java.lang.String,
///         },
///         set_message = {
///             name = "setMessage",
///             sig = (java.lang.String as JString) -> void,
///         }
///     },
///     static_methods = {
///         example_static = {
///             name = "exampleStatic",
///             sig = (jint, java.lang.String as JString) -> jboolean,
///         }
///     },
///     native_methods = {
///         example_native = {
///             name = "exampleNative",
///             sig = (jint, java.lang.String as JString) -> jboolean,
///             fn = example_native_impl,
///         }
///     }
/// }
///
/// # Outputs
///
/// pub struct JTestAPI {
///     class: Global<JClass>,
///     get_message: JMethodID,
///     set_message: JMethodID,
///     example_static: JStaticMethodID,
/// }
/// impl JTestAPI {
///    pub fn get(env: &mut Env) -> Result<&'static Self> {
///         let static API: OnceCell<JTestAPI> = OnceCell::new();
///         API.get_or_try_init(|| {
///            let class = env.find_class("com/example/Test")?.into_global(env);
///            let native_methods: &[JNativeMethod] = &[
///               JNativeMethod {
///                   name: "exampleNative",
///                   sig: "(ILjava/lang/String;)Z",
///                   fn: example_native_impl,
///               }
///            ];
///            env.register_native_methods(&class, native_methods)?;
///            Ok(JTestAPI {
///                class: env.new_global_ref(class.as_ref())?,
///                get_message: env.get_method_id(&class, "getMessage", "()Ljava/lang/String;")?,
///                set_message: env.get_method_id(&class, "setMessage", "(Ljava/lang/String;)V")?,
///                 example_static: env.get_static_method_id(&class, "exampleStatic", "(ILjava/lang/String;)Z")?,
///            })
///         })
///    }
/// }
///
/// impl JTest {
///     pub fn get_message(&self, env: &mut Env) -> Result<JString> {
///          let api = JTestAPI::get(env)?;
///          _get_message_call(env, self, api.get_message)
/// }
macro_rules! jgen_bind_methods {
    () => {
        todo!()
    };
}

// ------------------
// EXAMPLES / TESTS *
// ------------------

// ----------- Minimal placeholders so this compiles in isolation -----------
use std::{borrow::Cow, ops::Deref};

mod sys {
    pub use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort, jvalue};
}

type Result<T> = core::result::Result<T, ()>;
type JMethodID = *mut jni::sys::_jmethodID;

pub struct Global<T>(T);
impl Deref for Global<JClass> {
    type Target = JClass;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct Env;

impl Env {
    fn new_global_ref<T>(&self, _obj: &T) -> Result<Global<T>>
    where
        T: crate::refs::Reference,
    {
        Ok(Global(T::null()))
    }
    fn get_method_id(&mut self, _class: &JClass, _name: &str, _sig: &str) -> Result<JMethodID> {
        Ok(std::ptr::null_mut())
    }

    fn get_raw(&self) -> *mut jni::sys::JNIEnv {
        std::ptr::null_mut()
    }

    fn exception_check(&self) -> bool {
        false
    }

    fn with_local_frame<F, R>(&self, _capacity: i32, f: F) -> Result<R>
    where
        F: FnOnce(&mut Env) -> Result<R>,
    {
        let mut env = Env;
        f(&mut env)
    }
}

static GLOBAL_JCLASS: JClass = JClass;
impl JTest {
    fn lookup_class() -> Result<&'static JClass> {
        Ok(&GLOBAL_JCLASS)
    }
}

mod refs {
    use crate::objects::JClass;
    use crate::Env;
    use std::borrow::Cow;

    pub trait Reference {
        /// The fully qualified class name of the Java class represented by this
        /// reference.
        ///
        /// The class name is expected to be dot-separated, in the same format as
        /// `Class.getName()` and suitable for passing to `Class.forName()`
        ///
        /// For example: `"com.example.MyClass"`
        ///
        /// Note: this format is very similar to the FindClass naming conventions,
        /// except for the use of dots instead of slashes.
        ///
        /// An array of objects would look like: "[Ljava.lang.Object;" An array of
        /// integers would look like: "[I"
        fn class_name() -> Cow<'static, str>;
        fn as_raw(&self) -> jni::sys::jobject;
        fn null() -> Self
        where
            Self: Sized,
        {
            unsafe { std::mem::zeroed() }
        }
    }

    #[derive(Default)]
    pub enum LoaderContext {
        #[default]
        None,
    }
    impl LoaderContext {
        pub fn load_class_for_type<T>(
            &self,
            _initialize: bool,
            _env: &mut Env,
        ) -> Result<JClass, String> {
            Ok(JClass)
        }
    }
}

mod objects {
    use std::borrow::Cow;

    #[derive(Default)]
    pub struct JObject;
    impl AsRef<JObject> for JObject {
        fn as_ref(&self) -> &JObject {
            self
        }
    }
    impl crate::refs::Reference for JObject {
        fn class_name() -> Cow<'static, str> {
            Cow::Borrowed("java.lang.Object")
        }
        fn as_raw(&self) -> jni::sys::jobject {
            core::ptr::null_mut()
        }
    }

    impl JObject {
        pub fn from_raw(_env: &mut crate::Env, _obj: jni::sys::jobject) -> crate::Result<Self> {
            Ok(JObject)
        }
    }
    #[derive(Default)]
    pub struct JClass;
    impl AsRef<JClass> for JClass {
        fn as_ref(&self) -> &JClass {
            self
        }
    }
    impl crate::refs::Reference for JClass {
        fn class_name() -> Cow<'static, str> {
            Cow::Borrowed("java.lang.Class")
        }
        fn as_raw(&self) -> jni::sys::jobject {
            core::ptr::null_mut()
        }
    }
    #[derive(Default)]
    pub struct JString;
    impl AsRef<JString> for JString {
        fn as_ref(&self) -> &JString {
            self
        }
    }
    impl crate::refs::Reference for JString {
        fn class_name() -> Cow<'static, str> {
            Cow::Borrowed("java.lang.String")
        }
        fn as_raw(&self) -> jni::sys::jobject {
            core::ptr::null_mut()
        }
    }

    impl JString {
        pub fn from_raw(_env: &mut crate::Env, _obj: jni::sys::jobject) -> crate::Result<Self> {
            Ok(JString)
        }
    }
    pub struct JObjectArray<T>(std::marker::PhantomData<T>);
    impl<T> AsRef<JObjectArray<T>> for JObjectArray<T> {
        fn as_ref(&self) -> &JObjectArray<T> {
            self
        }
    }
    impl<T: crate::refs::Reference> crate::refs::Reference for JObjectArray<T> {
        fn class_name() -> Cow<'static, str> {
            let inner = T::class_name();
            let name = if inner.len() == 1 || inner.starts_with("[") {
                // inner = primitive OR array
                format!("[{inner}")
            } else {
                // inner = object
                format!("[L{inner};")
            };
            Cow::Owned(name)
        }
        fn as_raw(&self) -> jni::sys::jobject {
            core::ptr::null_mut()
        }
    }

    impl<T> JObjectArray<T> {
        pub fn from_raw(_env: &mut crate::Env, _obj: jni::sys::jobject) -> crate::Result<Self> {
            Ok(JObjectArray(std::marker::PhantomData))
        }
    }
    pub struct JPrimitiveArray<T>(std::marker::PhantomData<T>);
    impl<T> AsRef<JPrimitiveArray<T>> for JPrimitiveArray<T> {
        fn as_ref(&self) -> &JPrimitiveArray<T> {
            self
        }
    }
    impl<T> crate::refs::Reference for JPrimitiveArray<T> {
        fn class_name() -> Cow<'static, str> {
            Cow::Borrowed("[I") // Simplified - normally would depend on T
        }
        fn as_raw(&self) -> jni::sys::jobject {
            core::ptr::null_mut()
        }
    }
}

pub use objects::{JClass, JObject, JObjectArray, JPrimitiveArray, JString};

#[derive(Default)]
struct JTest;

impl AsRef<JTest> for JTest {
    fn as_ref(&self) -> &JTest {
        self
    }
}

impl crate::refs::Reference for JTest {
    fn class_name() -> Cow<'static, str> {
        Cow::Borrowed("com.example.Test")
    }

    fn as_raw(&self) -> jni::sys::jobject {
        std::ptr::null_mut()
    }
}

macro_rules! jni_internal_of_emit {
    ( ( $($norm:tt)+ ) ) => { __java_type_to_internal_literal!( $($norm)+ ) };
}
/// Test utility to get the JNI type descriptor of a raw type
macro_rules! jni_internal_of {
    ( $($raw:tt)+ ) => {
        __jsig_normalize_type_then!( jni_internal_of_emit, ( $($raw)+ ) )
    };
}

const _: &str = jni_internal_of!(jint); // "I"
const _: &str = jni_internal_of!(int); // "I"
const _: &str = jni_internal_of!(void); // "V"

const _: &str = jni_internal_of!(java.lang.String); // "Ljava/lang/String;"
const _: &str = jni_internal_of!([java.lang.String]); // "[Ljava/lang/String;"
const _: &str = jni_internal_of!([[java.lang.String]]); // "[[Ljava/lang/String;"
const _: &str = jni_internal_of!(java.util.Map::Entry); // "Ljava/util/Map$Entry;"
const _: &str = jni_internal_of!([java.util.Map::Entry]); // "[Ljava/util/Map$Entry;"
const _: &str = jni_internal_of!(.Hello); // "LHello;"
const _: &str = jni_internal_of!([.Hello]); // "[LHello;"
const _: &str = jni_internal_of!(char); // "C"
const _: &str = jni_internal_of!([char]); // "[C"

const _: &str = jni_internal_of!(java.lang.String); // "Ljava/lang/String;"
const _: &str = jni_internal_of!([[java.lang.String]]); // "[Ljava/lang/String;"

pub struct JTestAPI {
    class: Global<JClass>,
    rust_function_0_method_id: JMethodID,
    rust_function_1_method_id: JMethodID,
    rust_function_2_method_id: JMethodID,
    rust_function_3_method_id: JMethodID,
    rust_function_4_method_id: JMethodID,
    rust_function_5_method_id: JMethodID,
    rust_function_6_method_id: JMethodID,
    rust_function_7_method_id: JMethodID,
    rust_function_8_method_id: JMethodID,
    rust_function_10_method_id: JMethodID,
}
unsafe impl Send for JTestAPI {}
unsafe impl Sync for JTestAPI {}
impl JTestAPI {
    pub fn get(env: &Env, loader_context: &crate::refs::LoaderContext) -> Result<&'static Self> {
        static API: once_cell::sync::OnceCell<JTestAPI> = once_cell::sync::OnceCell::new();
        API.get_or_try_init(|| {
            env.with_local_frame(4, |env| {
                let class = loader_context
                    .load_class_for_type::<JTest>(false, env)
                    .unwrap();
                Ok(JTestAPI {
                    class: env.new_global_ref(class.as_ref())?,
                    rust_function_0_method_id: JTestAPI::_rust_function_0_lookup(env, &class)?,
                    rust_function_1_method_id: JTestAPI::_rust_function_1_lookup(env, &class)?,
                    rust_function_2_method_id: JTestAPI::_rust_function_2_lookup(env, &class)?,
                    rust_function_3_method_id: JTestAPI::_rust_function_3_lookup(env, &class)?,
                    rust_function_4_method_id: JTestAPI::_rust_function_4_lookup(env, &class)?,
                    rust_function_5_method_id: JTestAPI::_rust_function_5_lookup(env, &class)?,
                    rust_function_6_method_id: JTestAPI::_rust_function_6_lookup(env, &class)?,
                    rust_function_7_method_id: JTestAPI::_rust_function_7_lookup(env, &class)?,
                    rust_function_8_method_id: JTestAPI::_rust_function_8_lookup(env, &class)?,
                    rust_function_10_method_id: JTestAPI::_rust_function_10_lookup(env, &class)?,
                })
            })
        })
    }
}

jgen_bind_method!(
    this: JTest,
    javaFunction as rust_function_0,
    sig: (a: java.lang.String, c: jint) -> void
);

jgen_bind_method!(
    this: JTest,
    javaFunction as rust_function_1,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> jint
);

jgen_bind_method!(
    this: JTest,
    javaFunction as rust_function_2,
    sig: (a: &JString, b: java.lang.String[], c: jint) -> void
);

jgen_bind_method!(
    this: JTest,
    javaFunction as rust_function_3,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> JString
);

jgen_bind_method!(
    this: JTest,
    javaFunction as rust_function_4,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> crate::objects::JString
);

jgen_bind_method!(
    this: JTest,
    javaFunction as rust_function_5,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> java.lang.String
);

jgen_bind_method!(
    this: JTest,
    javaFunction as rust_function_6,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> java.lang.String[]
);
jgen_bind_method!(
    this: JTest,
    javaFunction as rust_function_7,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> java.lang.String[] as JObjectArray<JString>
);

jgen_bind_method!(
    this: JTest,
    javaFunction as rust_function_8,
    sig: (a: &JString, b: java.lang.String[], c: jint[]) -> void
);

// Test multi-dimensional primitive arrays with rust-like syntax
jgen_bind_method!(
    this: JTest,
    javaFunction as rust_function_10,
    sig: (a: &[[jint]], b: &[[[jchar]]], c: &[[jdouble]]) -> &[[jbyte]]
);

// Test macro that actually calls the real normalization functions
macro_rules! print_jni_sig_for {
    ( ( $($args:tt)* ) -> $($ret:tt)+ ) => {
        {
            let sig = __jsig_normalize_args_ret_then!( @expr __jsig_emit_sig_cow, ( $($args)* ), ( $($ret)+ ) );
            println!("Signature: ({}) -> {} = {}",
                stringify!($($args)*),
                stringify!($($ret)+),
                sig
            );
            sig
        }
    };
}

fn main() {
    /*
    TODO:
    - Allow jvoid or () as aliases for void
    - make sure we get a compiler error if void is used as a param type
     */

    // Test the new test_lookup_sig macro with simple primitive signature
    println!("=== Testing JNI Signature Generation ===");
    println!("Testing with simple primitive signature:\n");

    let sig = print_jni_sig_for!((a: jint, b: jlong) -> void);
    assert_eq!(sig, "(IJ)V\0");

    let sig = print_jni_sig_for!((a: jint, b: &JString) -> void);
    assert_eq!(sig, "(ILjava/lang/String;)V\0");

    let sig = print_jni_sig_for!((a: jint, b: java.lang.String) -> java.lang.String);
    assert_eq!(sig, "(ILjava/lang/String;)Ljava/lang/String;\0");

    let sig = print_jni_sig_for!((a: jint, b: [java.lang.String]) -> [java.lang.String]);
    assert_eq!(sig, "(I[Ljava/lang/String;)[Ljava/lang/String;\0");

    let sig = print_jni_sig_for!((a: jint, b: [java.lang.String]) -> java.lang.String[]);
    assert_eq!(sig, "(I[Ljava/lang/String;)[Ljava/lang/String;\0");

    let sig = print_jni_sig_for!((a: jint, b: [java.lang.String]) -> java.lang.String[][]);
    assert_eq!(sig, "(I[Ljava/lang/String;)[[Ljava/lang/String;\0");

    let sig = print_jni_sig_for!((a: jint, b: java.lang.String[]) -> java.lang.String);
    assert_eq!(sig, "(I[Ljava/lang/String;)Ljava/lang/String;\0");

    let sig = print_jni_sig_for!((a: jint, b: java.lang.String[][]) -> java.lang.String);
    assert_eq!(sig, "(I[[Ljava/lang/String;)Ljava/lang/String;\0");

    println!("\n=== Testing Suffix Array Arguments ===");
    println!("The following demonstrates that suffix array arguments (java.lang.String[]) now parse correctly:");

    let sig =
        print_jni_sig_for!((name: java.lang.String, items: java.util.List[], count: jint) -> void);
    assert_eq!(sig, "(Ljava/lang/String;[Ljava/util/List;I)V\0");
    let sig = print_jni_sig_for!((name: java.lang.String, items: java.util.List[][], count: jint) -> void);
    assert_eq!(sig, "(Ljava/lang/String;[[Ljava/util/List;I)V\0");

    println!("\n=== Testing Individual Descriptors ===");
    // Test that primitive arrays now parse correctly
    println!("1D Primitive array descriptors:");
    println!("jint[] -> {}", jni_internal_of!(jint[]));
    println!("jbyte[] -> {}", jni_internal_of!(jbyte[]));
    println!("jchar[] -> {}", jni_internal_of!(jchar[]));

    println!("\n2D Primitive array descriptors:");
    println!("jint[][] -> {}", jni_internal_of!(jint[][]));
    println!("jbyte[][] -> {}", jni_internal_of!(jbyte[][]));
    println!("jchar[][] -> {}", jni_internal_of!(jchar[][]));

    println!("\n3D Primitive array descriptors:");
    println!("jint[][][] -> {}", jni_internal_of!(jint[][][]));
    println!("jchar[][][] -> {}", jni_internal_of!(jchar[][][]));

    println!("\nBracket syntax descriptors:");
    println!("[[jint]] -> {}", jni_internal_of!([[jint]]));
    println!("[[[jchar]]] -> {}", jni_internal_of!([[[jchar]]]));

    // Don't call these, just make sure they compile
    let _ = || {
        let mut env = Env;
        let test = JTest;

        test.rust_function_0(&env, &JObject, 42).unwrap();
        let _a: JObjectArray<JObject> = test
            .rust_function_6(&mut env, &JString, &JString, 42)
            .unwrap();
    };
}
