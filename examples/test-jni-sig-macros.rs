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
    ( $cb:tt, ( () )     $(, $($pass:tt)* )? ) => { $cb!{ ( prim( void )     ) $(, $($pass)* )? } };
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
    ( ( $($norm:tt)* ), $real_cb:tt $(, $($rest:tt)* )? ) => {
        $real_cb!( ( $($norm)* ) $(, $($rest)* )? );
    };
}
/// Shim that routes from normalization macros to the real callback, for expressions without a
/// trailing semicolon
macro_rules! __then_expr {
    ( ( $($norm:tt)* ), $real_cb:tt $(, $($rest:tt)* )? ) => {
        $real_cb!( ( $($norm)* ) $(, $($rest)* )? )
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
    // End of list (with pass) — tolerate a trailing comma when caller passed an empty args list
    ( @ with_pass $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)+ ), ) => {
        $cb!{ ( $($acc)* ), $($pass)+ }
    };
    // End of list (no pass)
    ( @ no_pass $cb:tt, ( $($acc:tt)* ), () ) => {
        $cb!( ( $($acc)* ) )
    };
    // End of list (no pass) — tolerate a trailing comma when caller passed an empty args list
    ( @ no_pass $cb:tt, ( $($acc:tt)* ), (), ) => {
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
    // Empty args specializations to avoid trailing-comma edge cases
    ( @ expr $cb:tt, ( ), $($pass:tt)+ ) => {
        $cb!( (), $($pass)+ )
    };
    ( @ item $cb:tt, ( ), $($pass:tt)+ ) => {
        $cb!( (), $($pass)+ );
    };
    ( @ expr $cb:tt, ( ) ) => {
        $cb!( () )
    };
    ( @ item $cb:tt, ( ) ) => {
        $cb!( () );
    };
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
    ( ( $($norm_ret:tt)+ ), $real_cb:tt, ( $($norm_args:tt)* ) $(, $($rest:tt)* )? ) => {
        $real_cb!( ( $($norm_args)* ), ( $($norm_ret)+ ) $(, $($rest)* )? )
    };
}
/// Shim that routes from return type normalization to the real callback, accepting both normalized args and ret
macro_rules! __then_args_ret_item {
    ( ( $($norm_ret:tt)+ ), $real_cb:tt, ( $($norm_args:tt)* ) $(, $($rest:tt)* )? ) => {
        $real_cb!( ( $($norm_args)* ), ( $($norm_ret)+ ) $(, $($rest)* )? );
    };
}

/// Shim that forwards normalized args to return type normalization
macro_rules! __args_then_ret_expr {
    ( ( $($norm_args:tt)* ), $real_cb:tt, ( $($raw_ret:tt)+ ) $(, $($rest:tt)* )? ) => {
        __jsig_normalize_ret_then!( @ expr __then_args_ret_expr, ( $($raw_ret)+ ), $real_cb, ( $($norm_args)* ) $(, $($rest)* )? )
    };
}
macro_rules! __args_then_ret_item {
    ( ( $($norm_args:tt)* ), $real_cb:tt, ( $($raw_ret:tt)+ ) $(, $($rest:tt)* )? ) => {
        __jsig_normalize_ret_then!( @ item __then_args_ret_item, ( $($raw_ret)+ ), $real_cb, ( $($norm_args)* ) $(, $($rest)* )? );
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
            $( __java_type_to_internal_literal!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ), )*
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
    ( $ty:ty, $expr:expr ) => {{ <$ty as $crate::refs::Reference>::as_raw(($expr).as_ref()) }};
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
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( void ) )
    ) => {
        paste::paste! {
            unsafe {
                use $crate::refs::Reference as _;
                jni_call_check_ex!(
                    $env,
                    v1_1,
                    [<$kind VoidMethodA>],
                    ($this).as_ref().as_raw(),
                    $method_id,
                    $jni_args.as_ptr()
                )
            }
        }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jboolean ) )
    ) => {
        paste::paste! {
            unsafe {
                use $crate::refs::Reference as _;
                jni_call_check_ex!(
                    $env,
                    v1_1,
                    [<$kind BooleanMethodA>],
                    ($this).as_ref().as_raw(),
                    $method_id,
                    $jni_args.as_ptr()
                )
            }
        }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jbyte ) )
    ) => {
        paste::paste! {
            unsafe {
                use $crate::refs::Reference as _;
                jni_call_check_ex!(
                    $env,
                    v1_1,
                    [<$kind ByteMethodA>],
                    ($this).as_ref().as_raw(),
                    $method_id,
                    $jni_args.as_ptr()
                )
            }
        }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jchar ) )
    ) => {
        paste::paste! {
            unsafe {
                use $crate::refs::Reference as _;
                jni_call_check_ex!(
                    $env,
                    v1_1,
                    [<$kind CharMethodA>],
                    ($this).as_ref().as_raw(),
                    $method_id,
                    $jni_args.as_ptr()
                )
            }
        }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jshort ) )
    ) => {
        paste::paste! {
            unsafe {
                use $crate::refs::Reference as _;
                jni_call_check_ex!(
                    $env,
                    v1_1,
                    [<$kind ShortMethodA>],
                    ($this).as_ref().as_raw(),
                    $method_id,
                    $jni_args.as_ptr()
                )
            }
        }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jint ) )
    ) => {
        paste::paste! {
            unsafe {
                use $crate::refs::Reference as _;
                jni_call_check_ex!(
                    $env,
                    v1_1,
                    [<$kind IntMethodA>],
                    ($this).as_ref().as_raw(),
                    $method_id,
                    $jni_args.as_ptr()
                )
            }
        }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jlong ) )
    ) => {
        paste::paste! {
            unsafe {
                use $crate::refs::Reference as _;
                jni_call_check_ex!(
                    $env,
                    v1_1,
                    [<$kind LongMethodA>],
                    ($this).as_ref().as_raw(),
                    $method_id,
                    $jni_args.as_ptr()
                )
            }
        }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jfloat ) )
    ) => {
        paste::paste! {
            unsafe {
                use $crate::refs::Reference as _;
                jni_call_check_ex!(
                    $env,
                    v1_1,
                    [<$kind FloatMethodA>],
                    ($this).as_ref().as_raw(),
                    $method_id,
                    $jni_args.as_ptr()
                )
            }
        }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jdouble ) )
    ) => {
        paste::paste! {
            unsafe {
                use $crate::refs::Reference as _;
                jni_call_check_ex!(
                    $env,
                    v1_1,
                    [<$kind DoubleMethodA>],
                    ($this).as_ref().as_raw(),
                    $method_id,
                    $jni_args.as_ptr()
                )
            }
        }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( obj  ( $($rt:tt)+ ) as rust ( $as_ty:ty ) )
    ) => {
        paste::paste! {
            unsafe {
                use $crate::refs::Reference as _;
                let ret_obj: jni::sys::jobject = jni_call_check_ex!(
                    $env,
                    v1_1,
                    [<$kind ObjectMethodA>],
                    ($this).as_ref().as_raw(),
                    $method_id,
                    $jni_args.as_ptr()
                )?;
                <$as_ty>::from_raw($env, ret_obj)
            }
        }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( rust ( $as_ty:ty ) )
    ) => {
        paste::paste! {
            unsafe {
                use $crate::refs::Reference as _;
                let ret_obj: jni::sys::jobject = jni_call_check_ex!(
                    $env,
                    v1_1,
                    [<$kind ObjectMethodA>],
                    ($this).as_ref().as_raw(),
                    $method_id,
                    $jni_args.as_ptr()
                )?;
                <$as_ty>::from_raw($env, ret_obj)
            }
        }
    };
}

/// Emit a method ID lookup function for the given method name and signature
macro_rules! __jgen_emit_method_id_lookup {
    (
        $this:path,
        $jname:ident,
        $rname:ident,
        $lookup_api:ident,
        ( $($args:tt)* ),
        ( $($ret:tt)+ )
    ) => {
        paste! {
            fn [<_ $rname _lookup>](env: &mut Env, class: &JClass) -> $crate::errors::Result<JMethodID> {
                let sig = __jsig_normalize_args_ret_then!( @expr __jsig_emit_sig_cow, ( $($args)* ), ( $($ret)+ ) );
                env.$lookup_api(class, stringify!($jname), &sig)
            }
        }
    };
}

// Final emitter: we now have normalized args and ret; generate the whole fn
macro_rules! __jgen_emit_method_impl__with_norm_args_ret {
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
            ) -> $crate::errors::Result< __jgen_emit_rust_return_type!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? ) > {
                let api = [<$this API>]::get(env, &$crate::refs::LoaderContext::None)?;
                let jni_args = __jsig_args_to_jvalue_array!( ( $( $an : $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ),* ) );

                __jgen_emit_jni_sys_call!{
                    Call,
                    env,
                    self,
                    api.[<$rname _method_id>],
                    jni_args,
                    ( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? )
                }
            }
        }
    };
}

/// A shim before __jgen_emit_method_impl__with_norm_args_ret that determines whether &Env or &mut Env is needed
macro_rules! __jgen_emit_method_impl__with_norm_args_ret__shim {
    // Primitive return types (including void) use &Env
    (
        ( $( $nargs:tt )* ),
        ( prim ( $($p:tt)+ ) ),
        $this:path,
        $rname:ident
    ) => {
        __jgen_emit_method_impl__with_norm_args_ret!( &Env, $this, $rname, ( $( $nargs )* ), ( prim( $($p)+ ) ) );
    };
    // Object return types use &mut Env because they create new local references
    (
        ( $( $nargs:tt )* ),
        ( obj ( $($rt:tt)+ ) as rust ( $($ret_as:tt)+ ) ),
        $this:path,
        $rname:ident
    ) => {
        __jgen_emit_method_impl__with_norm_args_ret!( &mut Env, $this, $rname, ( $( $nargs )* ), ( obj( $($rt)+ ) as rust( $($ret_as)+ ) ) );
    };
    // Rust return types use &mut Env because they create new local references
    (
        ( $( $nargs:tt )* ),
        ( rust ( $($ret_as:tt)+ ) ),
        $this:path,
        $rname:ident
    ) => {
        __jgen_emit_method_impl__with_norm_args_ret!( &mut Env, $this, $rname, ( $( $nargs )* ), ( rust( $($ret_as)+ ) ) );
    };
}

macro_rules! __jgen_emit_method_impl {
    (
        $this:path,
        $rname:ident,
        ( $($args:tt)* ),
        ( $($ret:tt)+ )
    ) => {
        __jsig_normalize_args_ret_then!( @ item __jgen_emit_method_impl__with_norm_args_ret__shim, ( $($args)* ), ( $($ret)+ ), $this, $rname );
    };
}

// Final emitter: we now have normalized args and ret; generate the whole fn
macro_rules! __jgen_emit_static_method_impl__with_norm_args_ret {
    (
        $envType:ty,
        $this:path,
        $rname:ident,
        ( $( $an:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* $(,)? ),
        ( $rk:ident ( $($rt:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? )
    ) => {
        paste! {
            pub fn $rname (
                env: $envType,
                $( $an: __jgen_emit_rust_method_arg_type!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ) ),*
            ) -> $crate::errors::Result< __jgen_emit_rust_return_type!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? ) > {
                let api = [<$this API>]::get(env, &$crate::refs::LoaderContext::None)?;
                let jni_args = __jsig_args_to_jvalue_array!( ( $( $an : $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ),* ) );

                __jgen_emit_jni_sys_call!{
                    CallStatic,
                    env,
                    api.class.as_ref(),
                    api.[<$rname _method_id>],
                    jni_args,
                    ( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? )
                }
            }
        }
    };
}

/// A shim before __jgen_emit_static_method_impl__with_norm_args_ret that determines whether &Env or &mut Env is needed
macro_rules! __jgen_emit_static_method_impl__with_norm_args_ret__shim {
    // Primitive return types (including void) use &Env
    (
        ( $( $nargs:tt )* ),
        ( prim ( $($p:tt)+ ) ),
        $this:path,
        $rname:ident
    ) => {
        __jgen_emit_static_method_impl__with_norm_args_ret!( &Env, $this, $rname, ( $( $nargs )* ), ( prim( $($p)+ ) ) );
    };
    // Object return types use &mut Env because they create new local references
    (
        ( $( $nargs:tt )* ),
        ( obj ( $($rt:tt)+ ) as rust ( $($ret_as:tt)+ ) ),
        $this:path,
        $rname:ident
    ) => {
        __jgen_emit_static_method_impl__with_norm_args_ret!( &mut Env, $this, $rname, ( $( $nargs )* ), ( obj( $($rt)+ ) as rust( $($ret_as)+ ) ) );
    };
    // Rust return types use &mut Env because they create new local references
    (
        ( $( $nargs:tt )* ),
        ( rust ( $($ret_as:tt)+ ) ),
        $this:path,
        $rname:ident
    ) => {
        __jgen_emit_static_method_impl__with_norm_args_ret!( &mut Env, $this, $rname, ( $( $nargs )* ), ( rust( $($ret_as)+ ) ) );
    };
}

macro_rules! __jgen_emit_static_method_impl {
    (
        $this:path,
        $rname:ident,
        ( $($args:tt)* ),
        ( $($ret:tt)+ )
    ) => {
        __jsig_normalize_args_ret_then!( @ item __jgen_emit_static_method_impl__with_norm_args_ret__shim, ( $($args)* ), ( $($ret)+ ), $this, $rname );
    };
}

/// Binds a static Java method to Rust by emitting a method ID lookup function and a call function.
///
/// This assumes the existence of an `API` struct named `${this}API` with a `get` method that
/// returns a cached instance of the struct, which contains a field named `${rname}_method_id`
/// of type `JMethodID`.
///
/// The `this` type must implement `Reference`.
///
/// # Usage
///
/// ```
///  jgen_bind_static_method!(
///      this: JTest,
///      javaStaticMethodName as rust_static_method_name,
///      sig: (a: &JString, b: java.lang.String as JString, c: jint) -> void
///  );
/// ```
///
///  # Outputs:
///
/// ```
/// impl JTestAPI {
///     fn _rust_method_name_lookup(env: &mut Env, class: &JClass) -> Result<JMethodID> {
///         let sig = std::borrow::Cow::Borrowed("(Ljava/lang/String;Ljava/lang/String;I)V\u{0}");
///         env.get_method_id(class, "javaMethodName", &sig)
///     }
/// }
/// impl JTest {
///     pub fn rust_method_name(&self, env: Env, a: impl AsRef<JString>, b: impl AsRef<JString>, c: jint) -> Result<()> {
///         let api = JTestAPI::get(env, &LoaderContext::None)?;
///         let jni_args = {
///             [
///                 jni::sys::jvalue {
///                     l: { <crate::objects::JObject as crate::refs::Reference>::as_raw(a.as_ref()) },
///                 },
///                 jni::sys::jvalue {
///                     l: { <crate::objects::JObject as crate::refs::Reference>::as_raw(b.as_ref()) },
///                 },
///                 jni::sys::jvalue {
///                     i: (c as jni::sys::jint),
///                 },
///             ]
///         };
///         jni_call_check_ex!(env, v1_1, CallVoidMethodA, self, api.rust_method_name, jni_args)
///     }
/// }
/// ```
macro_rules! jgen_bind_method {
    (
        this: $this:path,
        $jname:ident as $rname:ident,
        sig: ( $($args:tt)* ) -> ($($rty:tt)+)
    ) => {

        paste::paste!{
            impl [<$this API>] {
                __jgen_emit_method_id_lookup!(
                    $this,
                    $jname,
                    $rname,
                    get_method_id,
                    ( $($args)* ),
                    ( $($rty)+ )
                );
            }
            impl $this {
                __jgen_emit_method_impl!(
                    $this,
                    $rname,
                    ( $($args)* ),
                    ( $($rty)+ )
                );
            }
        }
    };
    (
        this: $this:path,
        $jname:ident as $rname:ident,
        sig: ( $($args:tt)* ) -> $rty:path
    ) => {
        jgen_bind_method!(
            this: $this,
            $jname as $rname,
            sig: ( $($args)* ) -> ( $rty )
        );
    };
    (
        this: $this:path,
        $jname:ident as $rname:ident,
        sig: ( $($args:tt)* )
    ) => {
        jgen_bind_method!(
            this: $this,
            $jname as $rname,
            sig: ( $($args)* ) -> ( void )
        );
    };
    (
        this: $this:path,
        $jname:ident as $rname:ident,
        sig: ( $($args:tt)* ) -> ()
    ) => {
        jgen_bind_method!(
            this: $this,
            $jname as $rname,
            sig: ( $($args)* ) -> ( void )
        );
    };
}

macro_rules! jgen_bind_static_method {
    (
        this: $this:path,
        $jname:ident as $rname:ident,
        sig: ( $($args:tt)* ) -> $($rty:tt)+
    ) => {

        paste::paste!{
            impl [<$this API>] {
                __jgen_emit_method_id_lookup!(
                    $this,
                    $jname,
                    $rname,
                    get_static_method_id,
                    ( $($args)* ),
                    ( $($rty)+ )
                );
            }
            impl $this {
                __jgen_emit_static_method_impl!(
                    $this,
                    $rname,
                    ( $($args)* ),
                    ( $($rty)+ )
                );
            }
        }
    };
    (
        this: $this:path,
        $jname:ident as $rname:ident,
        sig: ( $($args:tt)* ) -> $rty:path
    ) => {
        jgen_bind_static_method!(
            this: $this,
            $jname as $rname,
            sig: ( $($args)* ) -> ( $rty )
        );
    };
    (
        this: $this:path,
        $jname:ident as $rname:ident,
        sig: ( $($args:tt)* )
    ) => {
        jgen_bind_static_method!(
            this: $this,
            $jname as $rname,
            sig: ( $($args)* ) -> ( void )
        );
    };
    (
        this: $this:path,
        $jname:ident as $rname:ident,
        sig: ( $($args:tt)* ) -> ()
    ) => {
        jgen_bind_static_method!(
            this: $this,
            $jname as $rname,
            sig: ( $($args)* ) -> ( void )
        );
    };
}

/// Compose a field JNI signature as a Cow<str> (literal or dynamic)
macro_rules! __jsig_emit_field_sig_literal {
    ( ( prim ( $($p:tt)+ ) ) ) => {
        std::borrow::Cow::Borrowed(concat!( __prim_to_internal!($($p)+), "\0" ))
    };
    ( ( obj ( $($j:tt)+ ) as rust ( $($as:tt)+ ) ) ) => {
        std::borrow::Cow::Borrowed(concat!( __java_type_to_internal_literal!( obj( $($j)+ ) as rust( $($as)+ ) ), "\0" ))
    };
}
macro_rules! __jsig_emit_field_sig_dynamic_owned {
    ( ( rust ( $($r:tt)+ ) ) ) => {{
        let mut buf = String::new();
        fn extend_with_internal(buf: &mut String, s: &str) {
            if s.len() > 1 && !s.starts_with('[') {
                buf.push('L');
                buf.extend(s.chars().map(|c| if c == '.' { '/' } else { c }));
                buf.push(';');
            } else {
                buf.extend(s.chars().map(|c| if c == '.' { '/' } else { c }));
            }
        }
        let cls = < $($r)+ as $crate::refs::Reference >::class_name();
        extend_with_internal(&mut buf, &cls);
        buf.push('\0');
        std::borrow::Cow::Owned(buf)
    }};
}
macro_rules! __jsig_emit_field_sig_cow {
    ( ( rust ( $($r:tt)+ ) ) ) => { __jsig_emit_field_sig_dynamic_owned!( ( rust( $($r)+ ) ) ) };
    ( ( prim ( $($p:tt)+ ) ) ) => { __jsig_emit_field_sig_literal!( ( prim( $($p)+ ) ) ) };
    ( ( obj ( $($j:tt)+ ) as rust ( $($as:tt)+ ) ) ) => { __jsig_emit_field_sig_literal!( ( obj( $($j)+ ) as rust( $($as)+ ) ) ) };
}

/// Emit a FieldID lookup helper
macro_rules! __jgen_emit_field_id_lookup {
    (
        $this:path,
        $jname:ident,
        $rname:ident,
        $lookup_api:ident,
        $ret_id_ty:ty,
        ( $($fty:tt)+ )
    ) => {
        paste! {
            fn [<_ $rname _lookup>](env: &mut Env, class: &JClass) -> $crate::errors::Result<$ret_id_ty> {
                let sig = __jsig_normalize_type_then!( __then_expr, ( $($fty)+ ), __jsig_emit_field_sig_cow );
                env.$lookup_api(class, stringify!($jname), &sig)
            }
        }
    };
}

/// Emit JNI get field call based on normalized type
macro_rules! __jgen_emit_jni_sys_get_field {
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jboolean ) ) ) => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind BooleanField>], ($this).as_ref().as_raw(), $field_id) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jbyte ) ) )    => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind ByteField>],    ($this).as_ref().as_raw(), $field_id) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jchar ) ) )    => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind CharField>],    ($this).as_ref().as_raw(), $field_id) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jshort ) ) )   => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind ShortField>],   ($this).as_ref().as_raw(), $field_id) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jint ) ) )     => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind IntField>],     ($this).as_ref().as_raw(), $field_id) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jlong ) ) )    => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind LongField>],    ($this).as_ref().as_raw(), $field_id) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jfloat ) ) )   => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind FloatField>],   ($this).as_ref().as_raw(), $field_id) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jdouble ) ) )  => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind DoubleField>],  ($this).as_ref().as_raw(), $field_id) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( obj  ( $($jt:tt)+ ) as rust ( $as_ty:ty ) ) ) => {{ paste! { unsafe { use $crate::refs::Reference as _; let ret_obj: jni::sys::jobject = jni_call_check_ex!($env, v1_1, [<$kind ObjectField>], ($this).as_ref().as_raw(), $field_id)?; <$as_ty>::from_raw($env, ret_obj) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( rust ( $as_ty:ty ) ) ) => {{ paste! { unsafe { use $crate::refs::Reference as _; let ret_obj: jni::sys::jobject = jni_call_check_ex!($env, v1_1, [<$kind ObjectField>], ($this).as_ref().as_raw(), $field_id)?; <$as_ty>::from_raw($env, ret_obj) } } }};
}

/// Emit JNI set field call based on normalized type
macro_rules! __jgen_emit_jni_sys_set_field {
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jboolean ) ) ) => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind BooleanField>], ($this).as_ref().as_raw(), $field_id, $val as jni::sys::jboolean) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jbyte ) ) )    => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind ByteField>],    ($this).as_ref().as_raw(), $field_id, $val as jni::sys::jbyte) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jchar ) ) )    => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind CharField>],    ($this).as_ref().as_raw(), $field_id, $val as jni::sys::jchar) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jshort ) ) )   => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind ShortField>],   ($this).as_ref().as_raw(), $field_id, $val as jni::sys::jshort) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jint ) ) )     => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind IntField>],     ($this).as_ref().as_raw(), $field_id, $val as jni::sys::jint) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jlong ) ) )    => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind LongField>],    ($this).as_ref().as_raw(), $field_id, $val as jni::sys::jlong) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jfloat ) ) )   => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind FloatField>],   ($this).as_ref().as_raw(), $field_id, $val as jni::sys::jfloat) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jdouble ) ) )  => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind DoubleField>],  ($this).as_ref().as_raw(), $field_id, $val as jni::sys::jdouble) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( obj  ( $($jt:tt)+ ) as rust ( $as_ty:ty ) ) ) => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind ObjectField>], ($this).as_ref().as_raw(), $field_id, __jsig_obj_reference_as_raw!($as_ty, $val)) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( rust ( $as_ty:ty ) ) ) => {{ paste! { unsafe { use $crate::refs::Reference as _; jni_call_check_ex!($env, v1_1, [<$kind ObjectField>], ($this).as_ref().as_raw(), $field_id, __jsig_obj_reference_as_raw!($as_ty, $val)) } } }};
}

// Instance field get/set implementations
macro_rules! __jgen_emit_field_impl__with_norm_type {
    ( $envGet:ty, $this:path, $rname:ident, $get:ident, $set:ident, ( $fk:ident ( $($ft:tt)* ) $( as rust ( $($fas:tt)+ ) )? ) ) => {
        paste! {
            pub fn $get(&self, env: $envGet) -> $crate::errors::Result< __jgen_emit_rust_return_type!( $fk( $($ft)* ) $( as rust( $($fas)+ ) )? ) > {
                let api = [<$this API>]::get(env, &$crate::refs::LoaderContext::None)?;
                __jgen_emit_jni_sys_get_field!( Get, env, self, api.[<$rname _field_id>], ( $fk( $($ft)* ) $( as rust( $($fas)+ ) )? ) )
            }
            pub fn $set(&self, env: &Env, val: __jgen_emit_rust_method_arg_type!( $fk( $($ft)* ) $( as rust( $($fas)+ ) )? )) -> $crate::errors::Result<()> {
                let api = [<$this API>]::get(env, &$crate::refs::LoaderContext::None)?;
                __jgen_emit_jni_sys_set_field!( Set, env, self, api.[<$rname _field_id>], val, ( $fk( $($ft)* ) $( as rust( $($fas)+ ) )? ) )
            }
        }
    };
}
macro_rules! __jgen_emit_field_impl__shim {
    ( ( prim ( $($p:tt)+ ) ), $this:path, $rname:ident, $get_rname:ident, $set_rname:ident ) => { __jgen_emit_field_impl__with_norm_type!( &Env, $this, $rname, $get_rname, $set_rname, ( prim( $($p)+ ) ) ); };
    ( ( obj ( $($jt:tt)+ ) as rust ( $($fas:tt)+ ) ), $this:path, $rname:ident, $get_rname:ident, $set_rname:ident ) => { __jgen_emit_field_impl__with_norm_type!( &mut Env, $this, $rname, $get_rname, $set_rname, ( obj( $($jt)+ ) as rust( $($fas)+ ) ) ); };
    ( ( rust ( $($fas:tt)+ ) ), $this:path, $rname:ident, $get_rname:ident, $set_rname:ident ) => { __jgen_emit_field_impl__with_norm_type!( &mut Env, $this, $rname, $get_rname, $set_rname, ( rust( $($fas)+ ) ) ); };
}
macro_rules! __jgen_emit_field_impl {
    ( $this:path, $rname:ident, $get:ident, $set:ident, ( $($fty:tt)+ ) ) => {
        __jsig_normalize_type_then!( __then_item, ( $($fty)+ ), __jgen_emit_field_impl__shim, $this, $rname, $get, $set );
    };
}

// Static field get/set implementations
macro_rules! __jgen_emit_static_field_impl__with_norm_type {
    ( $envGet:ty, $this:path, $rname:ident, ( $fk:ident ( $($ft:tt)* ) $( as rust ( $($fas:tt)+ ) )? ) ) => {
        paste! {
            pub fn $rname(env: $envGet) -> $crate::errors::Result< __jgen_emit_rust_return_type!( $fk( $($ft)* ) $( as rust( $($fas)+ ) )? ) > {
                let api = [<$this API>]::get(env, &$crate::refs::LoaderContext::None)?;
                __jgen_emit_jni_sys_get_field!( GetStatic, env, api.class.as_ref(), api.[<$rname _field_id>], ( $fk( $($ft)* ) $( as rust( $($fas)+ ) )? ) )
            }
            pub fn [<set_ $rname>](env: &Env, val: __jgen_emit_rust_method_arg_type!( $fk( $($ft)* ) $( as rust( $($fas)+ ) )? )) -> $crate::errors::Result<()> {
                let api = [<$this API>]::get(env, &$crate::refs::LoaderContext::None)?;
                __jgen_emit_jni_sys_set_field!( SetStatic, env, api.class.as_ref(), api.[<$rname _field_id>], val, ( $fk( $($ft)* ) $( as rust( $($fas)+ ) )? ) )
            }
        }
    };
}
macro_rules! __jgen_emit_static_field_impl__shim {
    ( ( prim ( $($p:tt)+ ) ), $this:path, $rname:ident ) => { __jgen_emit_static_field_impl__with_norm_type!( &Env, $this, $rname, ( prim( $($p)+ ) ) ); };
    ( ( obj ( $($jt:tt)+ ) as rust ( $($fas:tt)+ ) ), $this:path, $rname:ident ) => { __jgen_emit_static_field_impl__with_norm_type!( &mut Env, $this, $rname, ( obj( $($jt)+ ) as rust( $($fas)+ ) ) ); };
    ( ( rust ( $($fas:tt)+ ) ), $this:path, $rname:ident ) => { __jgen_emit_static_field_impl__with_norm_type!( &mut Env, $this, $rname, ( rust( $($fas)+ ) ) ); };
}
macro_rules! __jgen_emit_static_field_impl {
    ( $this:path, $rname:ident, ( $($fty:tt)+ ) ) => {
        __jsig_normalize_type_then!( __then_item, ( $($fty)+ ), __jgen_emit_static_field_impl__shim, $this, $rname );
    };
}

/// Public macro to bind an instance field
macro_rules! jgen_bind_field {
    (
        this: $this:path,
        $jname:ident as $rname:ident,
        sig: ($($fty:tt)+)
    ) => {
        paste::paste!{
            impl [<$this API>] {
                __jgen_emit_field_id_lookup!(
                    $this,
                    $jname,
                    $rname,
                    get_field_id,
                    JFieldID,
                    ( $($fty)+ )
                );
            }
            impl $this {
                __jgen_emit_field_impl!(
                    $this,
                    $rname,
                    $rname,
                    [< set_ $rname >],
                    ( $($fty)+ )
                );
            }
        }
    };
    (
        this: $this:path,
        $jname:ident as $rname:ident,
        sig: $fty:path
    ) => {
        jgen_bind_field!(
            this: $this,
            $jname as $rname,
            sig: ( $fty )
        );
    };
}

/// Public macro to bind a static field
macro_rules! jgen_bind_static_field {
    (
        this: $this:path,
        $jname:ident as $rname:ident,
        sig: ($($fty:tt)+)
    ) => {
        paste::paste!{
            impl [<$this API>] {
                __jgen_emit_field_id_lookup!(
                    $this,
                    $jname,
                    $rname,
                    get_static_field_id,
                    JStaticFieldID,
                    ( $($fty)+ )
                );
            }
            impl $this {
                __jgen_emit_static_field_impl!(
                    $this,
                    $rname,
                    ( $($fty)+ )
                );
            }
        }
    };
    (
        this: $this:path,
        $jname:ident as $rname:ident,
        sig: $fty:path
    ) => {
        jgen_bind_static_field!(
            this: $this,
            $jname as $rname,
            sig: ( $fty )
        );
    };
}

/// Invoke a callback with parsed method components after reordering arguments
macro_rules! __drt__parse_method_body__finish {
    ( ($($args:tt)*), ($($ret:tt)+), $rname:ident, $jname:literal, $callback:tt, $($extra:tt)* ) => {
        $callback!{ $rname, $jname, ( $($args)* ), ( $($ret)+ ), $($extra)* }
    };
    ( ($($args:tt)*), ($($ret:tt)+), $rname:ident, $jname:literal, $callback:tt ) => {
        $callback!{ $rname, $jname, ( $($args)* ), ( $($ret)+ ) }
    };
}

/// Parse one method body `{ name = "javaName", sig = (args) -> ret }` in any key order.
///
/// Keys can be repeated and the last one wins.
#[macro_export]
macro_rules! __drt__parse_method_body {
    // entrypoints
    ( @$kind:ident $callback:tt, $rname:ident = { $($method_desc:tt)* }, $($extra:tt)*) => {
        //compile_error!(concat!("parsing: ", stringify!( $($tt)+ ) ));
        __drt__parse_method_body!{@$kind @parse $callback $rname ( (stringify!($rname)) ) ( ) ( ) [ $($method_desc)* ] ($($extra)*) }
    };
    ( @$kind:ident $callback:tt, $rname:ident = { $($method_desc:tt)* }) => {
        //compile_error!(concat!("parsing: ", stringify!( $($tt)+ ) ));
        __drt__parse_method_body!{@$kind @parse $callback $rname ( (stringify!($rname)) ) ( ) ( ) [ $($method_desc)* ] () }
    };

    // name = literal
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($_jname:tt)* ) ( $($args:tt)* ) ( $($ret:tt)* ) [ name = $jname:literal , $($more:tt)* ] ($($extra:tt)*) ) => {
        //compile_error!(concat!("parsing: ", stringify!( $($more)+ ) ));
        __drt__parse_method_body!{@$kind @parse $cb $rname ( ($jname) ) ( $($args)* ) ( $($ret)* ) [ $($more)* ] ($($extra)*) }
    };
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($_jname:tt)* ) ( $($args:tt)* ) ( $($ret:tt)* ) [ name = $jname:literal ] ($($extra:tt)*) ) => {
        __drt__parse_method_body!{@$kind @finish $cb $rname ( ($jname) ) ( $($args)* ) ( $($ret)* ) [] ($($extra)*) }
    };

    // sig = (args) -> (ret)
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_args:tt)* ) ( $($_ret:tt)* ) [ sig = ( $($args:tt)* ) -> ( $($ret:tt)+ ) , $($more:tt)* ] ($($extra:tt)*) ) => {
        __drt__parse_method_body!{@$kind @parse $cb $rname ( $($jname)* ) ( ( $($args)* ) ) ( ( $($ret)+ ) ) [ $($more)* ] ($($extra)*) }
    };
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_args:tt)* ) ( $($_ret:tt)* ) [ sig = ( $($args:tt)* ) -> ( $($ret:tt)+ ) ] ($($extra:tt)*) ) => {
        __drt__parse_method_body!{@$kind @finish $cb $rname ( $($jname)* ) ( ( $($args)* ) ) ( ( $($ret)+ ) ) [] ($($extra)*) }
    };

    // sig = (args) -> ret:path
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_args:tt)* ) ( $($_ret:tt)* ) [ sig = ( $($args:tt)* ) -> $ret:path , $($more:tt)* ] ($($extra:tt)*) ) => {
        __drt__parse_method_body!{@$kind @parse $cb $rname ( $($jname)* ) ( ( $($args)* ) ) ( ( $ret ) ) [ $($more)* ] ($($extra)*) }
    };
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_args:tt)* ) ( $($_ret:tt)* ) [ sig = ( $($args:tt)* ) -> $ret:path ] ($($extra:tt)*) ) => {
        __drt__parse_method_body!{@$kind @finish $cb $rname ( $($jname)* ) ( ( $($args)* ) ) ( ( $ret ) ) [] ($($extra)*) }
    };

    // sig = (args) -> ()
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_args:tt)* ) ( $($_ret:tt)* ) [ sig = ( $($args:tt)* ) -> () , $($more:tt)* ] ($($extra:tt)*) ) => {
        __drt__parse_method_body!{@$kind @parse $cb $rname ( $($jname)* ) ( ( $($args)* ) ) ( ( void ) ) [ $($more)* ] ($($extra)*) }
    };
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_args:tt)* ) ( $($_ret:tt)* ) [ sig = ( $($args:tt)* ) -> () ] ($($extra:tt)*) ) => {
        __drt__parse_method_body!{@$kind @finish $cb $rname ( $($jname)* ) ( ( $($args)* ) ) ( ( void ) ) [] ($($extra)*) }
    };

    // sig = (args)
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_args:tt)* ) ( $($_ret:tt)* ) [ sig = ( $($args:tt)* ) , $($more:tt)* ] ($($extra:tt)*) ) => {
        __drt__parse_method_body!{@$kind @parse $cb $rname ( $($jname)* ) ( ( $($args)* ) ) ( ( void ) ) [ $($more)* ] ($($extra)*) }
    };
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_args:tt)* ) ( $($_ret:tt)* ) [ sig = ( $($args:tt)* ) ] ($($extra:tt)*) ) => {
        __drt__parse_method_body!{@$kind @finish $cb $rname ( $($jname)* ) ( ( $($args)* ) ) ( ( void ) ) [] ($($extra)*) }
    };

    // unexpected
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_args:tt)* ) ( $($_ret:tt)+ ) [ $bad:tt $($more:tt)* ] ($($extra:tt)*) ) => {
        compile_error!(concat!("define_reference_type!: unexpected token in method body: ", stringify!($bad)));
    };

    // missing required keys
    ( @$kind:ident @finish $cb:tt $rname:ident ( ) ( $($args:tt)* ) ( $($ret:tt)* ) [] ($($extra:tt)*) ) => {
        compile_error!("define_reference_type!: method missing required `name` key")
    };
    ( @$kind:ident @finish $cb:tt $rname:ident ( $jname:tt ) ( $($args:tt)* ) ( ) [] ($($extra:tt)*) ) => {
        compile_error!("define_reference_type!: method missing required `sig` key")
    };

    // finalize callback
    ( @$kind:ident @finish $cb:tt $rname:ident ( ($jname:literal) ) ( ( $($args:tt)* ) ) ( ( $($ret:tt)+ ) ) [] ($($extra:tt)+) ) => {
        //compile_error!(concat!("DEBUG: parsed method ", stringify!($rname), " = { name = ", stringify!($jname), ", sig = (", stringify!($($args)*), ") -> (", stringify!($($ret)+), ") }, extra = ", stringify!($($extra)+) ));
        __jsig_normalize_args_ret_then!{ @$kind __drt__parse_method_body__finish, ( $($args)* ), ( $($ret)+ ), $rname, $jname, $cb, $($extra)* }
    };
    ( @$kind:ident @finish $cb:tt $rname:ident ( ($jname:literal) ) ( ( $($args:tt)* ) ) ( ( $($ret:tt)+ ) ) [] () ) => {
        __jsig_normalize_args_ret_then!{ @$kind __drt__parse_method_body__finish, ( $($args)* ), ( $($ret)+ ), $rname, $jname, $cb }
    };
}

/// Emit Rust instance method bindings into `impl $Type { ... }`
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_method_binding {
    ( $rname:ident, $jname:literal, ( $($args:tt)* ), ( $($ret:tt)+ ), $Type:path ) => {
        __jgen_emit_method_impl__with_norm_args_ret__shim!{ ( $($args)* ), ( $($ret)+ ), $Type, $rname }
    };
}

/// Emit Rust static method bindings into `impl $Type { ... }`
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_static_method_binding {
    ( $rname:ident, $jname:literal, ( $($args:tt)* ), ( $($ret:tt)+ ), $Type:path ) => {
        __jgen_emit_static_method_impl__with_norm_args_ret__shim!{ ( $($args)* ), ( $($ret)+ ), $Type, $rname }
    };
}

macro_rules! __drt_emit_api_method_id_init {
    ( $rname:ident, $jname:literal, ( $($args:tt)* ), ( $($ret:tt)+ ), $env:expr, $class:expr ) => {
        {
            let sig = __jsig_emit_sig_cow!( ( $($args)* ), ( $($ret)+ ) );
            $env.get_method_id($class, $jname, &sig)?
        }
    };
}

macro_rules! __drt_emit_api_static_method_id_init {
    ( $rname:ident, $jname:literal, ( $($args:tt)* ), ( $($ret:tt)+ ), $env:expr, $class:expr ) => {
        {
            let sig = __jsig_emit_sig_cow!( ( $($args)* ), ( $($ret)+ ) );
            $env.get_static_method_id($class, $jname, &sig)?
        }
    };
}

/// Invoke a callback with parsed method components after reordering arguments
macro_rules! __drt__parse_field_body__finish {
    ( ($($type:tt)+), $rname:ident, $get_rname:ident, $set_rname:ident, $jname:literal, $callback:tt, $($extra:tt)* ) => {
        $callback!{ $rname, $get_rname, $set_rname, $jname, ( $($type)* ), $($extra)* }
    };
    ( ($($type:tt)+), $rname:ident, $get_rname:ident, $set_rname:ident, $jname:literal, $callback:tt ) => {
        $callback!{ $rname, $get_rname, $set_rname, $jname, ( $($type)* ) }
    };
}

/// Parse one field body `{ name = "javaName", sig = (type), get = <ident>, set = <ident> }` in any key order.
///
/// Keys can be repeated and the last one wins.
///
/// `name` defaults to the Rust name if not specified.
/// `get` defaults to the Rust name if not specified.
/// `set` defaults to `set_<rust name>` if not specified.
/// `sig` is required.
#[macro_export]
macro_rules! __drt__parse_field_body {
    // entrypoints
    ( @$kind:ident $callback:tt, $rname:ident = { $($field_desc:tt)* }, $($extra:tt)*) => {
        //compile_error!(concat!("parsing: ", stringify!( $($field_desc)+ ) ));
        paste::paste! {
            __drt__parse_field_body!{@$kind @parse $callback $rname ( (stringify!($rname)) ) ( ) ( $rname ) ( [<set_ $rname>] ) [ $($field_desc)* ] ($($extra)*) }
        }
    };
    ( @$kind:ident $callback:tt, $rname:ident = { $($field_desc:tt)* }) => {
        //compile_error!(concat!("parsing: ", stringify!( $($field_desc)+ ) ));
        paste::paste! {
            __drt__parse_field_body!{@$kind @parse $callback $rname ( (stringify!($rname)) ) ( ) ( $rname ) ( [<set_ $rname>] ) [ $($field_desc)* ] () }
        }
    };

    // name = literal
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($_jname:tt)* ) ( $($type:tt)* ) ( $get:ident ) ( $set:ident ) [ name = $jname:literal , $($more:tt)* ] ($($extra:tt)*) ) => {
        __drt__parse_field_body!{@$kind @parse $cb $rname ( ($jname) ) ( $($type)* ) ( $get ) ( $set ) [ $($more)* ] ($($extra)*) }
    };
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($_jname:tt)* ) ( $($type:tt)* ) ( $get:ident ) ( $set:ident ) [ name = $jname:literal ] ($($extra:tt)*) ) => {
        __drt__parse_field_body!{@$kind @finish $cb $rname ( ($jname) ) ( $($type)* ) ( $get ) ( $set ) [] ($($extra)*) }
    };

    // sig = (type)
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_type:tt)* ) ( $get:ident ) ( $set:ident ) [ sig = ( $($new_type:tt)* ) , $($more:tt)* ] ($($extra:tt)*) ) => {
        __drt__parse_field_body!{@$kind @parse $cb $rname ( $($jname)* ) ( ( $($new_type)* ) ) ( $get ) ( $set ) [ $($more)* ] ($($extra)*) }
    };
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_type:tt)* ) ( $get:ident ) ( $set:ident ) [ sig = ( $($new_type:tt)* ) ] ($($extra:tt)*) ) => {
        __drt__parse_field_body!{@$kind @finish $cb $rname ( $($jname)* ) ( ( $($new_type)* ) ) ( $get ) ( $set ) [] ($($extra)*) }
    };

    // sig = type:path
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_type:tt)* ) ( $get:ident ) ( $set:ident ) [ sig = $new_type:path , $($more:tt)* ] ($($extra:tt)*) ) => {
        __drt__parse_field_body!{@$kind @parse $cb $rname ( $($jname)* ) ( ( $new_type ) ) ( $get ) ( $set ) [ $($more)* ] ($($extra)*) }
    };
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_type:tt)* ) ( $get:ident ) ( $set:ident ) [ sig = $new_type:path ] ($($extra:tt)*) ) => {
        __drt__parse_field_body!{@$kind @finish $cb $rname ( $($jname)* ) ( ( $new_type ) ) ( $get ) ( $set ) [] ($($extra)*) }
    };

    // sig = ()
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_type:tt)* ) ( $get:ident ) ( $set:ident ) [ sig = () , $($more:tt)* ] ($($extra:tt)*) ) => {
        compile_error!("define_reference_type!: field `sig` cannot be void `()`");
    };
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($_type:tt)* ) ( $get:ident ) ( $set:ident ) [ sig = () ] ($($extra:tt)*) ) => {
        compile_error!("define_reference_type!: field `sig` cannot be void `()`");
    };

    // unexpected
    ( @$kind:ident @parse $cb:tt $rname:ident ( $($jname:tt)* ) ( $($type:tt)* ) ( $get:ident ) ( $set:ident ) [ $bad:tt $($more:tt)* ] ($($extra:tt)*) ) => {
        compile_error!(concat!("define_reference_type!: unexpected token in method body: ", stringify!($bad)));
    };

    // missing required keys
    ( @$kind:ident @finish $cb:tt $rname:ident ( ) ( $($type:tt)* ) ( $get:ident ) ( $set:ident ) [] ($($extra:tt)*) ) => {
        compile_error!("define_reference_type!: field missing required `name` key")
    };
    ( @$kind:ident @finish $cb:tt $rname:ident ( $jname:tt ) ( ) ( $get:ident ) ( $set:ident ) [] ($($extra:tt)*) ) => {
        compile_error!("define_reference_type!: field missing required `sig` key")
    };

    // finalize callback
    ( @$kind:ident @finish $cb:tt $rname:ident ( ($jname:literal) ) ( ( $($type:tt)+ ) ) ( $get:ident ) ( $set:ident ) [] ($($extra:tt)+) ) => {
        __jsig_normalize_ret_then!{ @$kind __drt__parse_field_body__finish, ( $($type)* ), $rname, $get, $set, $jname, $cb, $($extra)* }
    };
    ( @$kind:ident @finish $cb:tt $rname:ident ( ($jname:literal) ) ( ( $($type:tt)+ ) ) ( $get:ident ) ( $set:ident ) [] () ) => {
        __jsig_normalize_ret_then!{ @$kind __drt__parse_field_body__finish, ( $($type)* ), $rname, $get, $set, $jname, $cb }
    };
}

/// Emit Rust instance field bindings into `impl $Type { ... }`
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_field_binding {
    ( $rname:ident, $get:ident, $set:ident, $jname:literal, ( $($type:tt)+ ), $Type:path ) => {
        __jgen_emit_field_impl__shim!{ ( $($type)* ), $Type, $rname, $get, $set}
    };
}

/// Emit Rust static field bindings into `impl $Type { ... }`
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_static_field_binding {
    ( $rname:ident, $get:ident, $set:ident, $jname:literal, ( $($type:tt)+ ), $Type:path ) => {
        __jgen_emit_static_field_impl__shim!{ ( $($type)* ), $Type, $rname }
    };
}

macro_rules! __drt_emit_api_field_id_init {
    ( $rname:ident, $get:ident, $set:ident, $jname:literal, ( $($type:tt)+ ), $env:expr, $class:expr ) => {
        {
            let sig = __jsig_emit_field_sig_cow!( ( $($type)* ) );
            $env.get_field_id($class, $jname, &sig)?
        }
    };
}

macro_rules! __drt_emit_api_static_field_id_init {
    ( $rname:ident, $get:ident, $set:ident, $jname:literal, ( $($type:tt)+ ), $env:expr, $class:expr ) => {
        {
            let sig = __jsig_emit_field_sig_cow!( ( $($type)* ) );
            $env.get_static_field_id($class, $jname, &sig)?
        }
    };
}

/// Emit the API struct and its get() implementation from raw maps for instance and static methods
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_api_struct_and_get {
    (
        { $( $method_rname:ident = { $($method_desc:tt)* } ),* $(,)? },
        { $( $static_method_rname:ident = { $($static_method_desc:tt)* } ),* $(,)? },
        { $( $field_rname:ident = { $($field_desc:tt)* } ),* $(,)? },
        { $( $static_field_rname:ident = { $($static_field_desc:tt)* } ),* $(,)? },
        $ApiTy:ident, $Type:ident, $LoadClass:expr
    ) => {
        paste::paste!{
            struct $ApiTy {
                class: $crate::refs::Global<$crate::objects::JClass>,
                $( [<$method_rname _method_id>]: JMethodID, )*
                $( [<$static_method_rname _method_id>]: JStaticMethodID, )*
                $( [<$field_rname _field_id>]: JFieldID, )*
                $( [<$static_field_rname _field_id>]: JStaticFieldID, )*
            }

            impl $ApiTy {
                #[allow(unused)]
                fn _load_class_wrapper<F, R>(env: &mut $crate::Env, loader: &$crate::refs::LoaderContext, initialize: bool, load_class: F) -> $crate::errors::Result<R>
                where
                    F: FnOnce(&mut $crate::Env, &$crate::refs::LoaderContext, bool) -> $crate::errors::Result<R>,
                {
                    load_class(env, loader, initialize)
                }

                pub fn get(env: &Env, loader: &$crate::refs::LoaderContext) -> $crate::errors::Result<&'static Self> {
                    static CELL: once_cell::sync::OnceCell<$ApiTy> = once_cell::sync::OnceCell::new();
                    CELL.get_or_try_init(|| {
                        env.with_local_frame(4, |env| {
                            let class = Self::_load_class_wrapper(env, &loader, false, $LoadClass)?;
                            let api = Self {
                                class: env.new_global_ref(&class)?,
                                $(
                                    [<$method_rname _method_id>]: __drt__parse_method_body!{@expr __drt_emit_api_method_id_init, $method_rname = { $($method_desc)+ }, env, &class },
                                )*
                                $(
                                    [<$static_method_rname _method_id>]: __drt__parse_method_body!{@expr __drt_emit_api_static_method_id_init, $static_method_rname = { $($static_method_desc)+ }, env, &class },
                                )*
                                $(
                                    [<$field_rname _field_id>]: __drt__parse_field_body!{@expr __drt_emit_api_field_id_init, $field_rname = { $($field_desc)+ }, env, &class },
                                )*
                                $(
                                    [<$static_field_rname _field_id>]: __drt__parse_field_body!{@expr __drt_emit_api_static_field_id_init, $static_field_rname = { $($static_field_desc)+ }, env, &class },
                                )*
                            };
                            Ok(api)
                        })
                    })
                }
            }
        }
    };
}

/// The actual emitter, parameterized by the resolved API ident.
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_with_api {
    // Normalized lists variant: both methods and static methods have normalized args/ret
    (
        $ApiTy:ident,
        $Type:ident,
        $Class:expr,
        $RawTy:ident,
        $LoadClass:expr,
        [ $($Aliases:tt)* ],
        { $( $method_rname:ident = { $($method_desc:tt)* } ),* $(,)? },
        { $( $static_method_rname:ident = { $($static_method_desc:tt)* } ),* $(,)? },
        { $( $field_rname:ident = { $($field_desc:tt)* } ),* $(,)? },
        { $( $static_field_rname:ident = { $($static_field_desc:tt)* } ),* $(,)? }
        $(,)?
    ) => {
        paste::paste!{
            // Define a minimal concrete type for this test harness
            pub struct $Type;
            // Minimal Reference impl so we can call as_raw() in the harness
            impl $crate::refs::Reference for $Type {
                fn class_name() -> std::borrow::Cow<'static, str> { std::borrow::Cow::Borrowed($Class) }
                fn as_raw(&self) -> jni::sys::jobject { core::ptr::null_mut() }
            }
            // Emit API and bindings (raw map forms)
            __drt__emit_api_struct_and_get!(
                { $( $method_rname = { $($method_desc)+ } ),* },
                { $( $static_method_rname = { $($static_method_desc)+ } ),* },
                { $( $field_rname = { $($field_desc)+ } ),* },
                { $( $static_field_rname = { $($static_field_desc)+ } ),* },
                $ApiTy, $Type, $LoadClass );

            impl $Type {
                $(
                    __drt__parse_method_body!{@item __drt__emit_method_binding, $method_rname = { $($method_desc)+ }, $Type }
                )*
                $(
                    __drt__parse_method_body!{@item __drt__emit_static_method_binding, $static_method_rname = { $($static_method_desc)+ }, $Type }
                )*
            }
            impl AsRef<$Type> for $Type {
                fn as_ref(&self) -> &$Type { self }
            }

            // The API struct holds raw JNI IDs (raw pointers) — mark Send+Sync in this harness
            unsafe impl Send for $ApiTy {}
            unsafe impl Sync for $ApiTy {}
        }
    };
}

/// Resolve an API ident (auto => `<Type>API`) and call a callback with it.
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__with_api_ident {
    (__auto_api, $Type:ident, $callback:ident $(, $args:tt)*) => {
        paste::paste! { $crate::$callback!([<$Type API>] $(, $args)*); }
    };
    ($Api:ident, $Type:ident, $callback:ident $(, $args:tt)*) => {
        $crate::$callback!($Api $(, $args)*)
    };
}

// Emit hook that normalizes and hands off to codegen
#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_emit {
    (
        type   = $Type:ident,
        class  = $Class:expr,
        raw    = $RawIdent:ident,
        api    = $Api:ident,
        load_class   = ($LoadClass:expr),
        aliases = [ $($Aliases:tt)* ],
        methods = { $($Methods:tt)* },
        static_methods = { $($StaticMethods:tt)* },
        fields = { $($Fields:tt)* },
        static_fields = { $($StaticFields:tt)* },
    ) => {
        $crate::__drt__with_api_ident!(
            $Api,
            $Type,
            __drt__emit_with_api,
            $Type,
            $Class,
            $RawIdent,
            $LoadClass,
            [ $($Aliases)* ],
            { $($Methods)* },
            { $($StaticMethods)* },
            { $($Fields)* },
            { $($StaticFields)* }
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_parse {
    // Finished parsing tokens: finalize by appending defaults and extracting
    (@parse_tokens { type = ($Type:ident), class = ($Class:expr), pairs = [$($pairs:tt)*] }) => {
        $crate::__def_ref_finalize!([
            (type, $Type)
            (class, $Class)
            $($pairs)*
            (raw, jobject)
            (api, __auto_api)
            (load_class, (|env, loader_context, initialize| { loader_context.load_class_for_type::<$Type>(initialize, env) }))
            (aliases, [])
            (methods, {})
            (static_methods, {})
            (fields, {})
            (static_fields, {})
            (sentinel, ())
        ]);
    };

    // Parse tokens: type = <ident>
    (@parse_tokens { type = (), class = $Class:tt, pairs = [$($acc:tt)*] } type = $value:ident $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = ($value), class = $Class, pairs = [$($acc)*] } $($rest)* }
    };
    // If type already set, updating it is an error to keep semantics simple
    (@parse_tokens { type = ($set:ident), class = $Class:tt, pairs = [$($acc:tt)*] } type = $value:ident $($rest:tt)*) => {
        compile_error!("define_reference_type!: duplicate `type` key")
    };

    // Parse tokens: class = <expr>, with comma
    (@parse_tokens { type = $Type:tt, class = (), pairs = [$($acc:tt)*] } class = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = ($value), pairs = [$($acc)*] } $($rest)* }
    };
    // Parse tokens: class = <expr>, no comma (end)
    (@parse_tokens { type = $Type:tt, class = (), pairs = [$($acc:tt)*] } class = $value:expr) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = ($value), pairs = [$($acc)*] } }
    };
    // If class already set, error on duplicate
    (@parse_tokens { type = $Type:tt, class = ($set:expr), pairs = [$($acc:tt)*] } class = $value:expr, $($rest:tt)*) => {
        compile_error!("define_reference_type!: duplicate `class` key")
    };
    (@parse_tokens { type = $Type:tt, class = ($set:expr), pairs = [$($acc:tt)*] } class = $value:expr) => {
        compile_error!("define_reference_type!: duplicate `class` key")
    };

    // Parse tokens: raw = <ident>
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } raw = $value:ident $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (raw, $value)] } $($rest)* }
    };

    // Parse tokens: api = <ident>
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } api = $value:ident $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (api, $value)] } $($rest)* }
    };

    // Parse tokens: load_class = <expr>, with comma
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } load_class = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (load_class, ($value))] } $($rest)* }
    };
    // Parse tokens: load_class = <expr>, no comma (end)
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } load_class = $value:expr) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (load_class, ($value))] } }
    };

    // Parse tokens: as = [aliases]
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } as = [$($value:tt)*] $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (aliases, [$($value)*])] } $($rest)* }
    };

    // Parse tokens: methods = {methods}
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } methods = {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (methods, {$($value)*})] } $($rest)* }
    };

    // Parse tokens: methods {methods}
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } methods {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (methods, {$($value)*})] } $($rest)* }
    };

    // Parse tokens: static_methods = {static_methods}
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } static_methods = {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (static_methods, {$($value)*})] } $($rest)* }
    };

    // Parse tokens: static_methods {static_methods}
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } static_methods {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (static_methods, {$($value)*})] } $($rest)* }
    };

    // Parse tokens: fields = {fields}
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } fields = {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (fields, {$($value)*})] } $($rest)* }
    };

    // Parse tokens: fields {fields}
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } fields {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (fields, {$($value)*})] } $($rest)* }
    };

    // Parse tokens: static_fields = {static_fields}
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } static_fields = {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (static_fields, {$($value)*})] } $($rest)* }
    };

    // Parse tokens: static_fields {static_fields}
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } static_fields {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (static_fields, {$($value)*})] } $($rest)* }
    };

    // Parse tokens: Skip commas
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } , $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)*] } $($rest)* }
    };

    // Parse tokens: Error on unexpected tokens
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } $bad:tt $($rest:tt)*) => {
        compile_error!(concat!("Unexpected token in define_reference_type: ", stringify!($bad)));
    };
}

// Lookups for pairs (order-agnostic)
#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_type {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!(type, $($args)*); };
    ([(type, $Type:ident) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!($Type $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_type!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_class {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!(class, $($args)*); };
    ([(class, $Class:expr) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!($Class $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_class!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_raw {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!(raw, $($args)*); };
    ([(raw, $Raw:ident) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!($Raw $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_raw!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_api {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!(api, $($args)*); };
    ([(api, $Api:ident) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!($Api $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_api!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_load_class {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!(load_class, $($args)*); };
    ([(load_class, $LoadClass:tt) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!($LoadClass $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_load_class!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_aliases {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!(aliases, $($args)*); };
    ([(aliases, [$($Aliases:tt)*]) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!([$($Aliases)*] $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_aliases!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_methods {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!(methods, $($args)*); };
    ([(methods, {$($Methods:tt)*}) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!({$($Methods)*} $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_methods!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_static_methods {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!($($args)*); };
    ([(static_methods, {$($StaticMethods:tt)*}) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!({$($StaticMethods)*} $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_static_methods!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_fields {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!(fields, $($args)*); };
    ([(fields, {$($Fields:tt)*}) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!({$($Fields)*} $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_fields!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_static_fields {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!(static_fields, $($args)*); };
    ([(static_fields, {$($StaticFields:tt)*}) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!({$($StaticFields)*} $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_static_fields!([$($rest)*], $found, $not $(, $args)*); };
}

// Finalizer chain: extract required, then optional, then emit
#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_finalize {
    ([$($pairs:tt)*]) => {
        //compile_error!(concat!("DEBUG: pairs = ", stringify!([$($pairs)*])));
        $crate::__def_ref_lookup_type!([$($pairs)*], $crate::__def_ref_found_type, $crate::__def_ref_missing_type, [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_missing_type {
    ([$($pairs:tt)*]) => {
        compile_error!("define_reference_type!: missing required `type` field")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_type {
    ($Type:ident, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_class!([$($pairs)*], $crate::__def_ref_found_class, $crate::__def_ref_missing_class, $Type, [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_missing_class {
    ($Type:ident, [$(_pairs:tt)*]) => {
        compile_error!("define_reference_type!: missing required `class` field")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_class {
    ($Class:expr, $Type:ident, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_raw!([$($pairs)*], $crate::__def_ref_found_raw, $crate::__def_ref_missing_prop__unreachable, $Type, $Class, [$($pairs)*]);
    };
}

/// This should be unreachable because the it's used with properties that have default values
#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_missing_prop__unreachable {
    ($field:ident $($any:tt)*) => {
        compile_error!(concat!(
            "internal error: ",
            stringify!($field),
            " property missing but it should have a default"
        ))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_raw {
    ($Raw:ident, $Type:ident, $Class:expr, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_api!([$($pairs)*], $crate::__def_ref_found_api, $crate::__def_ref_missing_prop__unreachable, $Type, $Class, $Raw, [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_api {
    ($Api:ident, $Type:ident, $Class:expr, $Raw:ident, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_load_class!([$($pairs)*], $crate::__def_ref_found_load_class, $crate::__def_ref_missing_prop__unreachable, $Type, $Class, $Raw, $Api, [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_load_class {
    ($LoadClass:tt, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_aliases!([$($pairs)*], $crate::__def_ref_found_aliases, $crate::__def_ref_missing_prop__unreachable, $Type, $Class, $Raw, $Api, $LoadClass, [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_aliases {
    ([$($Aliases:tt)*], $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $LoadClass:tt, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_methods!([$($pairs)*], $crate::__def_ref_found_methods, $crate::__def_ref_missing_prop__unreachable, $Type, $Class, $Raw, $Api, $LoadClass, [$($Aliases)*], [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_methods {
    ({$($Methods:tt)*}, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $LoadClass:tt, [$($Aliases:tt)*], [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_static_methods!([$($pairs)*], $crate::__def_ref_found_static_methods, $crate::__def_ref_missing_prop__unreachable, $Type, $Class, $Raw, $Api, $LoadClass, [$($Aliases)*], {$($Methods)*}, [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_static_methods {
    ({$($StaticMethods:tt)*}, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $LoadClass:tt, [$($Aliases:tt)*], {$($Methods:tt)*}, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_fields!([$($pairs)*], $crate::__def_ref_found_fields, $crate::__def_ref_missing_prop__unreachable, $Type, $Class, $Raw, $Api, $LoadClass, [$($Aliases)*], {$($Methods)*}, {$($StaticMethods)*}, [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_fields {
    ({$($Fields:tt)*}, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $LoadClass:tt, [$($Aliases:tt)*], {$($Methods:tt)*}, {$($StaticMethods:tt)*}, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_static_fields!([$($pairs)*], $crate::__def_ref_found_static_fields, $crate::__def_ref_missing_prop__unreachable, $Type, $Class, $Raw, $Api, $LoadClass, [$($Aliases)*], {$($Methods)*}, {$($StaticMethods)*}, {$($Fields)*});
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_static_fields {
    ({$($StaticFields:tt)*}, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $LoadClass:tt, [$($Aliases:tt)*], {$($Methods:tt)*}, {$($StaticMethods:tt)*}, {$($Fields:tt)*}) => {
        $crate::__def_ref_emit! {
            type   = $Type,
            class  = $Class,
            raw    = $Raw,
            api    = $Api,
            load_class   = ($LoadClass),
            aliases = [$($Aliases)*],
            methods = {$($Methods)*},
            static_methods = {$($StaticMethods)*},
            fields = {$($Fields)*},
            static_fields = {$($StaticFields)*},
        }
    };
}

/// Emits a cacheable method IDs API struct, lookup APIs and call APIs for a given set of methods
///
/// # Usage
/// ```
/// fn example_native_impl<'local>(env: &mut Env<'local>, this: JTest<'local>, arg0: jint, arg1: impl AsRef<JString>) -> Result<jboolean> {
///    // ...
///    Ok(true)
/// }
///
/// declare_reference_type!{
///     type = JTest,
///     class = "com.example.Test",
///     error_policy = ThrowRuntimeExAndDefault,
///     constructors = {
///        new = { sig = () },
///        new_from_value = { sig = (value: java.lang.String as JString) },
///        new_flat(jint),
///     }
///     methods = {
///         get_message = { name = "getMessage", sig = () -> (java.lang.String as JString) },
///         set_message = { name = "setMessage", sig = (msg: java.lang.String as JString) -> void },
///         exampleFlat as example_flat(val: jint, name: java.lang.String as JString) -> jboolean,
///     },
///     static_methods = {
///         example_static = { name = "exampleStatic", sig = (val: jint, name: java.lang.String as JString) -> jboolean },
///     },
///     native_methods = {
///         example_native = {
///             name = "exampleNative",
///             sig = (val: jint, name: java.lang.String as JString) -> jboolean,
///             fn = example_native_impl,
///         }
///     },
///     fields = {
///         short = { sig = (java.lang.String as JString) },
///         long_field = { name = "longField", sig = jint },
///         flat(jint),
///         longFlat as long_flat(jint),
///     },
///     static_fields = {
///         static_field0 = { name = "staticField0", sig = (java.lang.String as JString) },
///     }
/// }
/// ```
///
/// # Outputs
///
/// ```
/// pub struct JTestAPI {
///     class: Global<JClass>,
///     get_message_method_id: JMethodID,
///     set_message_method_id: JMethodID,
///     example_static_method_id: JStaticMethodID,
///     field0_field_id: JFieldID,
///     field1_field_id: JFieldID,
///     static_field0_field_id: JStaticFieldID,
/// }
/// impl JTestAPI {
///     pub fn get(env: &Env, loader_context: &LoaderContext) -> Result<&'static Self> {
///         static API: once_cell::sync::OnceCell<JTestAPI> = once_cell::sync::OnceCell::new();
///         API.get_or_try_init(|| {
///            let mut env = env.with_local_frame(8, |env| {
///                let class = env.find_class("com/example/Test")?.into_global(env);
///                let native_methods: &[JNativeMethod] = &[
///                    JNativeMethod {
///                        name: "exampleNative",
///                        sig: "(ILjava/lang/String;)Z",
///                        fn: Self::example_native__shim,
///                    }
///                ];
///                env.register_native_methods(&class, native_methods)?;
///                Ok(JTestAPI {
///                    class: env.new_global_ref(class.as_ref())?,
///                    get_message_method_id: env.get_method_id(&class, "getMessage", "()Ljava/lang/String;")?,
///                    set_message_method_id: env.get_method_id(&class, "setMessage", "(Ljava/lang/String;)V")?,
///                    example_static_method_id: env.get_static_method_id(&class, "exampleStatic", "(ILjava/lang/String;)Z")?,
///                    field0_field_id: env.get_field_id(&class, "field0", "Ljava/lang/String;")?,
///                    field1_field_id: env.get_field_id(&class, "field1", "I")?,
///                    static_field0_field_id: env.get_static_field_id(&class, "staticField0", "Ljava/lang/String;")?,
///                })
///            })
///         })
///     }
///     fn example_native__shim<'local>(unowned_env: UnownedEnv, this: JTest<'local>, arg0: jint, arg1: JObject<'local>) -> jboolean {
///         unowned_env.with_env(|env| {
///            let arg1 = JString::from_raw(env, arg1)?;
///            example_native_impl(env, this, arg0, arg1)
///         }).resolve::<ThrowRuntimeExAndDefault>()
///     }
/// }
///
/// impl JTest {
///     pub fn new(env: &mut Env) -> Result<Self> {
///          let api = JTestAPI::get(env, &LoaderContext::None)?;
///          let jni_args = [ ];
///          __jgen_emit_jni_sys_call!(NewObject, ...)
///     }
///     pub fn new_from_value(env: &mut Env, value: impl AsRef<JString>) -> Result<Self> {
///          let api = JTestAPI::get(env, &LoaderContext::None)?;
///          let jni_args = [value.as_ref().as_raw()];
///          __jgen_emit_jni_sys_call!(NewObject, ...)
///     }
///     pub fn get_message(&self, env: &mut Env) -> Result<JString> {
///          let api = JTestAPI::get(env, &LoaderContext::None)?;
///          let jni_args = [ ];
///          __jgen_emit_jni_sys_call!(Call, ...)
///     }
///     pub fn set_message(&self, env: &Env, msg: impl AsRef<JString>) -> Result<()> {
///          let api = JTestAPI::get(env, &LoaderContext::None)?;
///          let jni_args = [msg.as_ref().as_raw()];
///          __jgen_emit_jni_sys_call!(Call, ...)
///     }
///     pub fn example_static(env: &mut Env, val: jint, msg: impl AsRef<JString>) -> Result<jboolean> {
///          let api = JTestAPI::get(env, &LoaderContext::None)?;
///          let jni_args = [val, msg.as_ref().as_raw()];
///          __jgen_emit_jni_sys_call!(CallStatic, ...)
///     }
///     pub fn short(&self, env: &mut Env) -> Result<JString> {
///          let api = JTestAPI::get(env, &LoaderContext::None)?;
///          __jgen_emit_jni_sys_get_field!(Get, ...)
///     }
///     pub fn set_short(&self, env: &Env, val: impl AsRef<JString>) -> Result<()> {
///          let api = JTestAPI::get(env, &LoaderContext::None)?;
///          __jgen_emit_jni_sys_set_field!(Set, ...)
///     }
///     // ... long_field(), flat() and long_flat() accessors omitted for brevity
///     pub fn static_field0(env: &mut Env) -> Result<JString> {
///         let api = JTestAPI::get(env, &LoaderContext::None)?;
///          __jgen_emit_jni_sys_get_field!(GetStatic, ...)
///     }
///     pub fn set_static_field0(env: &Env, val: impl AsRef<JString>) -> Result<()> {
///         let api = JTestAPI::get(env, &LoaderContext::None)?;
///          __jgen_emit_jni_sys_set_field!(SetStatic, ...)
///     }
/// }
/// ```
/// Define a new reference type that wraps a `JObject` and implements `Reference`.
#[macro_export]
macro_rules! define_reference_type {
    (
        type = $Type:ident,
        class = $Class:expr
        $(, $($rest:tt)*)?
    ) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = ($Type), class = ($Class), pairs = [] } $( $($rest)* )?
        }
    };
}

// ------------------
// EXAMPLES / TESTS *
// ------------------

// ----------- Minimal placeholders so this compiles in isolation -----------
use std::borrow::Cow;

mod sys {
    pub use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort, jvalue};
}

type JMethodID = *mut jni::sys::_jmethodID;
type JStaticMethodID = *mut jni::sys::_jmethodID;
type JFieldID = *mut jni::sys::_jfieldID;
type JStaticFieldID = *mut jni::sys::_jfieldID;

#[derive(Default)]
pub struct Env;

impl Env {
    fn new_global_ref<T: Default>(&self, _obj: &T) -> crate::errors::Result<crate::refs::Global<T>>
    where
        T: crate::refs::Reference,
    {
        Ok(crate::refs::Global::default())
    }
    fn get_method_id(
        &mut self,
        _class: &JClass,
        _name: &str,
        _sig: &str,
    ) -> crate::errors::Result<JMethodID> {
        Ok(std::ptr::null_mut())
    }
    fn get_static_method_id(
        &mut self,
        _class: &JClass,
        _name: &str,
        _sig: &str,
    ) -> crate::errors::Result<JMethodID> {
        Ok(std::ptr::null_mut())
    }
    fn get_field_id(
        &mut self,
        _class: &JClass,
        _name: &str,
        _sig: &str,
    ) -> crate::errors::Result<JFieldID> {
        Ok(std::ptr::null_mut())
    }
    fn get_static_field_id(
        &mut self,
        _class: &JClass,
        _name: &str,
        _sig: &str,
    ) -> crate::errors::Result<JStaticFieldID> {
        Ok(std::ptr::null_mut())
    }

    fn get_raw(&self) -> *mut jni::sys::JNIEnv {
        std::ptr::null_mut()
    }

    fn exception_check(&self) -> bool {
        false
    }

    fn find_class(&mut self, _name: &str) -> crate::errors::Result<JClass> {
        Ok(JClass)
    }

    fn with_local_frame<F, R>(&self, _capacity: i32, f: F) -> crate::errors::Result<R>
    where
        F: FnOnce(&mut Env) -> crate::errors::Result<R>,
    {
        let mut env = Env;
        f(&mut env)
    }
}

mod errors {
    pub type Result<T> = core::result::Result<T, ()>;
}

mod refs {
    use crate::Env;
    use crate::objects::JClass;
    use std::borrow::Cow;
    use std::ops::Deref;

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
        ) -> crate::errors::Result<JClass> {
            Ok(JClass)
        }
    }

    #[derive(Default)]
    pub struct Global<T>(T);
    impl Deref for Global<JClass> {
        type Target = JClass;
        fn deref(&self) -> &Self::Target {
            &self.0
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
        pub fn from_raw(
            _env: &mut crate::Env,
            _obj: jni::sys::jobject,
        ) -> crate::errors::Result<Self> {
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
        pub fn from_raw(
            _env: &mut crate::Env,
            _obj: jni::sys::jobject,
        ) -> crate::errors::Result<Self> {
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
        pub fn from_raw(
            _env: &mut crate::Env,
            _obj: jni::sys::jobject,
        ) -> crate::errors::Result<Self> {
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

use crate::objects::{JClass, JObject, JObjectArray, JPrimitiveArray, JString};
use crate::refs::Global;

#[derive(Default)]
struct JTest;

// Types JFoo and JBar will be generated by define_reference_type! invocations below.

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
    rust_static_function_0_method_id: JMethodID,
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
    // Field IDs (for testing field-binding macros)
    field0_field_id: JFieldID,
    field1_field_id: JFieldID,
    field_arr_field_id: JFieldID,
    static_field0_field_id: JStaticFieldID,
    static_count_field_id: JStaticFieldID,
}
unsafe impl Send for JTestAPI {}
unsafe impl Sync for JTestAPI {}
impl JTestAPI {
    pub fn get(
        env: &Env,
        loader_context: &crate::refs::LoaderContext,
    ) -> crate::errors::Result<&'static Self> {
        static API: once_cell::sync::OnceCell<JTestAPI> = once_cell::sync::OnceCell::new();
        API.get_or_try_init(|| {
            env.with_local_frame(4, |env| {
                let class = loader_context
                    .load_class_for_type::<JTest>(false, env)
                    .unwrap();
                Ok(JTestAPI {
                    class: env.new_global_ref(class.as_ref())?,
                    rust_static_function_0_method_id: JTestAPI::_rust_static_function_0_lookup(
                        env, &class,
                    )?,
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
                    field0_field_id: JTestAPI::_field0_lookup(env, &class)?,
                    field1_field_id: JTestAPI::_field1_lookup(env, &class)?,
                    field_arr_field_id: JTestAPI::_field_arr_lookup(env, &class)?,
                    static_field0_field_id: JTestAPI::_static_field0_lookup(env, &class)?,
                    static_count_field_id: JTestAPI::_static_count_lookup(env, &class)?,
                })
            })
        })
    }
}

jgen_bind_static_method!(
    this: JTest,
    javaStaticFunction as rust_static_function_0,
    sig: (a: java.lang.String, c: jint) -> void
);

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
    sig: (a: &JString, b: java.lang.String[], c: jint)
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
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> (java.lang.String)
);

jgen_bind_method!(
    this: JTest,
    javaFunction as rust_function_6,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> (java.lang.String[])
);
jgen_bind_method!(
    this: JTest,
    javaFunction as rust_function_7,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> (java.lang.String[] as JObjectArray<JString>)
);

jgen_bind_method!(
    this: JTest,
    javaFunction as rust_function_8,
    sig: (a: &JString, b: java.lang.String[], c: jint[]) -> ()
);

// Test multi-dimensional primitive arrays with rust-like syntax
jgen_bind_method!(
    this: JTest,
    javaFunction as rust_function_10,
    sig: (a: &[[jint]], b: &[[[jchar]]], c: &[[jdouble]]) -> (&[[jbyte]])
);

// -------- Field binding macro tests (parsing/expansion) --------

// Instance field: object
jgen_bind_field!(
    this: JTest,
    field0 as field0,
    sig: (java.lang.String as JString)
);

// Instance field: primitive
jgen_bind_field!(
    this: JTest,
    field1 as field1,
    sig: jint
);

// Instance field: object array (explicit rust type)
jgen_bind_field!(
    this: JTest,
    field_arr as field_arr,
    sig: (java.lang.String[] as JObjectArray<JString>)
);

// Static field: object
jgen_bind_static_field!(
    this: JTest,
    staticField0 as static_field0,
    sig: (java.lang.String as JString)
);

// Static field: primitive
jgen_bind_static_field!(
    this: JTest,
    staticCount as static_count,
    sig: jint
);

// -------- declare_reference_type! compile checks (method parsing/emission) --------

define_reference_type! {
    type = JFoo,
    class = "com.example.Foo",
    methods = {
        // object return
        get_msg = { name = "getMsg", sig = () -> (java.lang.String) },
        // void return and object arg with explicit Rust type
        set_msg = { name = "setMsg", sig = (msg: java.lang.String as JString) -> () },
        // primitive return, primitive args
        add = { name = "add", sig = (a: jint, b: jint) -> (jint) },
    },
    static_methods = {
        // simple primitive-returning static method
        version = { name = "version", sig = () -> (jint) },
        // object-returning static method with an object argument
        make = { name = "make", sig = (name: java.lang.String as JString) -> (java.lang.String) },
    },
}

define_reference_type! {
    type = JBar,
    class = "com.example.Bar",
    methods = {
        // array return (object[]) captured as a single token tree
        items = { name = "items", sig = () -> (java.lang.String[]) },
        // array argument (object[])
        put = { name = "put", sig = (name: java.lang.String as JString, arr: java.lang.String[]) -> () },
    },
    static_methods = {
        // static method returning object array
        empty_items = { name = "emptyItems", sig = () -> (java.lang.String[]) },
    },
    fields = {
        // object array field
        item_arr = { name = "itemArr", sig = (java.lang.String[]) },
    },
}

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
    println!(
        "The following demonstrates that suffix array arguments (java.lang.String[]) now parse correctly:"
    );

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

        // declare_reference_type! invocations above should parse and expand; no runtime calls here.
    };
}
