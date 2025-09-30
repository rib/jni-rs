// Import the `paste!` macro
use paste::paste;

//pub use crate::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort, jvalue};

macro_rules! jprim_canon {
    (void) => {
        void
    };
    (jboolean) => {
        jboolean
    };
    (boolean) => {
        jboolean
    };
    (jbyte) => {
        jbyte
    };
    (byte) => {
        jbyte
    };
    (jchar) => {
        jchar
    };
    (char) => {
        jchar
    };
    (jshort) => {
        jshort
    };
    (short) => {
        jshort
    };
    (jint) => {
        jint
    };
    (int) => {
        jint
    };
    (jlong) => {
        jlong
    };
    (long) => {
        jlong
    };
    (jfloat) => {
        jfloat
    };
    (float) => {
        jfloat
    };
    (jdouble) => {
        jdouble
    };
    (double) => {
        jdouble
    };
}
macro_rules! jprim {
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

// ---------- internal names for Java types ----------
macro_rules! jname_internal {
    (. $outer:ident $( :: $inner:ident )* ) => {
        concat!( stringify!($outer) $(, "$", stringify!($inner) )* )
    };
    ($first:ident . $($rest:tt)+) => { jname_internal!(@accum [$first] $($rest)+) };
    (@accum [$($pkg:ident)+] $next:ident . $($more:tt)+) => {
        jname_internal!(@accum [$($pkg)+ $next] $($more)+)
    };
    (@accum [$($pkg:ident)+] $outer:ident $( :: $inner:ident )* ) => {
        concat!(
            $( stringify!($pkg), "/", )*
            stringify!($outer)
            $(, "$", stringify!($inner) )*
        )
    };
}

// ---------- Type → internal descriptor (normalized only) ----------
// Helpers to render Java names and arrays.
macro_rules! jdesc_java_name {
    ( $first:ident . $($rest:tt)+ ) => { concat!("L", jname_internal!($first . $($rest)+), ";") };
    ( . $outer:ident $( :: $inner:ident )* ) => { concat!("L", jname_internal!(. $outer $( :: $inner )*), ";") };
}

macro_rules! jdesc_java_elem {
    // Java reference element
    ( $first:ident . $($rest:tt)+ ) => { jdesc_java_name!($first . $($rest)+) };
    ( . $outer:ident $( :: $inner:ident )* ) => { jdesc_java_name!(. $outer $( :: $inner )*) };
    // Primitive element (canonical)
    ( jboolean ) => { jprim!(jboolean) };
    ( jbyte    ) => { jprim!(jbyte)    };
    ( jchar    ) => { jprim!(jchar)    };
    ( jshort   ) => { jprim!(jshort)   };
    ( jint     ) => { jprim!(jint)     };
    ( jlong    ) => { jprim!(jlong)    };
    ( jfloat   ) => { jprim!(jfloat)   };
    ( jdouble  ) => { jprim!(jdouble)  };
    // Primitive aliases
    ( boolean  ) => { jprim!(jboolean) };
    ( byte     ) => { jprim!(jbyte)    };
    ( char     ) => { jprim!(jchar)    };
    ( short    ) => { jprim!(jshort)   };
    ( int      ) => { jprim!(jint)     };
    ( long     ) => { jprim!(jlong)    };
    ( float    ) => { jprim!(jfloat)   };
    ( double   ) => { jprim!(jdouble)  };
}

macro_rules! jdesc_java_array {
    ( [ [ $($inner:tt)+ ] ] ) => { concat!("[", jdesc_java_array!([ $($inner)+ ]) ) };
    ( [ $($inner:tt)+ ] ) => { concat!("[", jdesc_java_elem!( $($inner)+ )) };
}

// Normalized-only jdesc
macro_rules! jdesc {
    // prim(...)
    ( prim ( $p:ident ) ) => { jprim!($p) };

    // obj(...), including array element syntax: obj([ ... ])
    ( obj ( [ $($inner:tt)+ ] ) as rust ( $as_ty:ty ) ) => { jdesc_java_array!([ $($inner)+ ]) };
    ( obj ( $first:ident . $($rest:tt)+ ) as rust ( $as_ty:ty ) ) => { jdesc_java_name!($first . $($rest)+) };
    ( obj ( . $outer:ident $( :: $inner:ident )* ) as rust ( $as_ty:ty ) ) => { jdesc_java_name!(. $outer $( :: $inner )*) };

    // rust(...) is only used in formatted signatures
    ( rust ( $($r:tt)+ ) ) => { "{}" };
}

// Helper: normalize a raw type and emit its descriptor with jdesc
macro_rules! jdesc_of_emit {
    // Accept grouped normalized type
    ( ( $($norm:tt)+ ) ) => { jdesc!( $($norm)+ ) };
}
macro_rules! jdesc_of {
    ( $($raw:tt)+ ) => {
        jnorm_type_then!( jdesc_of_emit, ( $($raw)+ ) )
    };
}

macro_rules! rust_jdesc {
    // rust(...) is only used in formatted signatures
    ( rust ( $($r:tt)+ ) ) => { &<$($r)+ as $crate::refs::Reference>::class_name() };

    // prim | obj
    ( $($raw:tt)+ ) => {
        jdesc!( $($raw)+ )
    };
}

// ---------- NORMALIZATION ----------

// Build the default Rust wrapper type for a Java array of arbitrary dimension.
// Base cases map a 1D array; recursion wraps further dimensions in JObjectArray<...>.
macro_rules! rust_type_for_java_array_default {
    // recursion: more dimensions
    ( [ [ $($inner:tt)+ ] ] ) => {
        JObjectArray< rust_type_for_java_array_default!( [ $($inner)+ ] ) >
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

// Normalize one raw type and immediately invoke a callback macro with the normalized form.
// Normalized shapes:
//   - prim( jprim_canon!(...) )
//   - obj( <java.name or .Default::Inner> ) as rust( <Ty> )   // default Ty = JObject if no `as`
//   - rust( <Ty> )                                           // Rust refs, including &[...]
// Usage:
//   jnorm_type_then!(CB, ( <raw-type> ) [, extra tokens... ])
// Calls:
//   CB!( <normalized-type> [, extra tokens...] )
//
// Normalization examples:
//   - jint -> prim(jint)
//   - int  -> prim(jint)
//   - java.lang.String -> obj(java.lang.String) as rust(JObject)
//   - [java.lang.String] -> obj([java.lang.String]) as rust(JObjectArray<JObject>)
//   - java.lang.String[] -> obj([java.lang.String]) as rust(JObjectArray<JObject>)
//   - jint[] -> obj([jint]) as rust(JPrimitiveArray<jint>)
//   - [jint] -> obj([jint]) as rust(JPrimitiveArray<jint>)
//   - [[jint]] -> obj([[jint]]) as rust(JObjectArray<JPrimitiveArray<jint>>)
//   - jint[][] -> obj([[jint]]) as rust(JObjectArray<JPrimitiveArray<jint>>)
//   - .NoPackage -> obj(NoPackage) as rust(JObject)
//   - .NoPackage as JString -> obj(NoPackage) as rust(JString)
//   - java.lang.String as JString -> obj(java.lang.String) as rust(JString)
//   - &JString -> rust(JString)
//   - &[JString] -> rust(JObjectArray<JString>)
//   - &[[JString]] -> rust(JObjectArray<JObjectArray<JString>>)
//   - &[jint] -> rust(JPrimitiveArray<jint>)
macro_rules! jnorm_type_then {
    // ----- Rust reference arrays -----
    ( $cb:tt, ( & [ [ $($inner:tt)+ ] ] ) $(, $($pass:tt)* )? ) => {
        jnorm_type_then!{@objarr $cb, () (), [ [ $($inner)+ ] ] $(, $($pass)* )? }
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
        jnorm_type_then!{@objarr $cb, ( $($open)* JObjectArray< ) ( > $($close)* ), [ $($inner)+ ] $(, $($pass)* )? }
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
        $cb!{ ( obj( [ $($arr)+ ] ) as rust( rust_type_for_java_array_default!([ $($arr)+ ]) ) ) $(, $($pass)* )? }
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
        $cb!{ ( obj( [ [ [ $first $( . $seg )+ $( :: $inner )* ] ] ] ) as rust( rust_type_for_java_array_default!([ [ [ $first $( . $seg )+ $( :: $inner )* ] ] ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $first:ident $( . $seg:ident )+ $( :: $inner:ident )* [ ] [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ $first $( . $seg )+ $( :: $inner )* ] ] ) as rust( rust_type_for_java_array_default!([ [ $first $( . $seg )+ $( :: $inner )* ] ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $first:ident $( . $seg:ident )+ $( :: $inner:ident )* [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ $first $( . $seg )+ $( :: $inner )* ] ) as rust( rust_type_for_java_array_default!([ $first $( . $seg )+ $( :: $inner )* ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* [ ] [ ] [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ [ . $outer $( :: $inner )* ] ] ] ) as rust( rust_type_for_java_array_default!([ [ [ . $outer $( :: $inner )* ] ] ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* [ ] [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ . $outer $( :: $inner )* ] ] ) as rust( rust_type_for_java_array_default!([ [ . $outer $( :: $inner )* ] ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ . $outer $( :: $inner )* ] ) as rust( rust_type_for_java_array_default!([ . $outer $( :: $inner )* ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $p:ident [ ] [ ] [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ [ $p ] ] ] ) as rust( rust_type_for_java_array_default!([ [ [ $p ] ] ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $p:ident [ ] [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ [ $p ] ] ) as rust( rust_type_for_java_array_default!([ [ $p ] ]) ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $p:ident [ ] ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ $p ] ) as rust( rust_type_for_java_array_default!([ $p ]) ) ) $(, $($pass)* )? }
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

// Shims: route from normalization macros to the real callback,
// adding a trailing semicolon only when generating items.
macro_rules! __then_item {
    ( ( $($norm:tt)+ ), $real_cb:tt $(, $rest:tt )* ) => {
        $real_cb!( ( $($norm)+ ) $(, $rest )* );
    };
}
macro_rules! __then_expr {
    ( ( $($norm:tt)+ ), $real_cb:tt $(, $rest:tt )* ) => {
        $real_cb!( ( $($norm)+ ) $(, $rest )* )
    };
}

// Rework jnorm_ret_then to use jnorm_type_then (variadic; forwards extra tokens)
macro_rules! jnorm_ret_then {
    ( @ expr $cb:tt, ( $($ret:tt)+ ) $(, $($pass:tt)* )? ) => {
        jnorm_type_then!( __then_expr, ( $($ret)+ ) $(, $cb, $($pass)* )? )
    };
    ( @ item $cb:tt, ( $($ret:tt)+ ) $(, $($pass:tt)* )? ) => {
        jnorm_type_then!( __then_item, ( $($ret)+ ) $(, $cb, $($pass)* )? );
    };
    // Back-compat: default to expr
    ( $cb:tt, ( $($ret:tt)+ ) $(, $($pass:tt)* )? ) => {
        jnorm_ret_then!( @ expr $cb, ( $($ret)+ ) $(, $($pass)* )? )
    };
}

// Normalize an args list, then invoke a callback macro that expects normalized args:
//   jnorm_args_then!(CB, ( a: TyA, b: TyB, ... ) [, extra tokens...])
// Add @expr | @item to control whether the callback emits an expression or items.
macro_rules! jnorm_args_then {
    // Explicit context + pass
    ( @ expr $cb:tt, ( $($args:tt)* ), $($pass:tt)+ ) => {
        jnorm_args_then_munch!( @ with_pass __then_expr, (), ( $cb, $($pass)+ ), $($args)* )
    };
    ( @ item $cb:tt, ( $($args:tt)* ), $($pass:tt)+ ) => {
        jnorm_args_then_munch!( @ with_pass __then_item, (), ( $cb, $($pass)+ ), $($args)* );
    };
    // Explicit context, no pass
    ( @ expr $cb:tt, ( $($args:tt)* ) ) => {
        jnorm_args_then_munch!( @ no_pass __then_expr, (), ( $cb ), $($args)* )
    };
    ( @ item $cb:tt, ( $($args:tt)* ) ) => {
        jnorm_args_then_munch!( @ no_pass __then_item, (), ( $cb ), $($args)* );
    };
    // Back-compat: default to expr
    ( $cb:tt, ( $($args:tt)* ), $($pass:tt)+ ) => {
        jnorm_args_then!( @ expr $cb, ( $($args)* ), $($pass)+ )
    };
    ( $cb:tt, ( $($args:tt)* ) ) => {
        jnorm_args_then!( @ expr $cb, ( $($args)* ) )
    };
}

// Internal helper: consume Java-style suffix-array [] without introducing tt/',' ambiguity.
macro_rules! __jnorm_arg_suffix {
    // More [] → accumulate and continue
    ( $push:tt, $cb:tt, @ $mode:ident,
      ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident,
      base( $($base:tt)+ ), dims( $($dims:tt)* ),
      [ ] $($after:tt)*
    ) => {
        __jnorm_arg_suffix!(
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
        jnorm_type_then!{
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
        jnorm_type_then!{
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
        jnorm_type_then!{
            $push, ( $($base)+ [ ] $($dims)* ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n, $($rest)*
        }
    };

    // End-of-list terminator (when there are trailing tokens that need to be passed through)
    ( $push:tt, $cb:tt, @ $mode:ident,
      ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident,
      base( $($base:tt)+ ), dims( $($dims:tt)* )
    ) => {
        jnorm_type_then!{
            $push, ( $($base)+ [ ] $($dims)* ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n
        }
    };

    // End-of-list terminator (when called with empty $($after)* - no trailing tokens)
    ( $push:tt, $cb:tt, @ $mode:ident,
      ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident,
      base( $($base:tt)+ ), dims( $($dims:tt)* ),
    ) => {
        jnorm_type_then!{
            $push, ( $($base)+ [ ] $($dims)* ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n
        }
    };
}

// Internal muncher that uses jnorm_type_then per argument type and accumulates a normalized list.
// Signature carries a mode (@ with_pass | @ no_pass) and a "( $pass )" group that is forwarded unchanged.
macro_rules! jnorm_args_then_munch {
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
        jnorm_type_then!{
            jnorm_args_then_push, ( & [ [ $($inner)+ ] ] ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // &[...]
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : & [ $($inner:tt)+ ]
      $(, $($rest:tt)*)?
    ) => {
        jnorm_type_then!{
            jnorm_args_then_push, ( & [ $($inner)+ ] ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // &Path
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : & $rust:ty
      $(, $($rest:tt)*)?
    ) => {
        jnorm_type_then!{
            jnorm_args_then_push, ( & $rust ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // Bracket arrays: [ ... ] as Ty
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : [ $($arr:tt)+ ] as $as_ty:ty
      $(, $($rest:tt)*)?
    ) => {
        jnorm_type_then!{
            jnorm_args_then_push, ( [ $($arr)+ ] as $as_ty ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // Bracket arrays: [ ... ]
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : [ $($arr:tt)+ ]
      $(, $($rest:tt)*)?
    ) => {
        jnorm_type_then!{
            jnorm_args_then_push, ( [ $($arr)+ ] ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // Java name with explicit `as`
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : $first:ident $( . $seg:ident )+ $( :: $inner:ident )* as $as_ty:ty
      $(, $($rest:tt)*)?
    ) => {
        jnorm_type_then!{
            jnorm_args_then_push, ( $first $( . $seg )+ $( :: $inner )* as $as_ty ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // Java name (no as, no suffix)
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : $first:ident $( . $seg:ident )+ $( :: $inner:ident )*
      $(, $($rest:tt)*)?
    ) => {
        jnorm_type_then!{
            jnorm_args_then_push, ( $first $( . $seg )+ $( :: $inner )* ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // Default-package with explicit `as`
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : . $outer:ident $( :: $inner:ident )* as $as_ty:ty
      $(, $($rest:tt)*)?
    ) => {
        jnorm_type_then!{
            jnorm_args_then_push, ( . $outer $( :: $inner )* as $as_ty ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // Default-package (no as, no suffix)
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : . $outer:ident $( :: $inner:ident )*
      $(, $($rest:tt)*)?
    ) => {
        jnorm_type_then!{
            jnorm_args_then_push, ( . $outer $( :: $inner )* ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n $(, $($rest)*)?
        }
    };

    // Primitive
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : $p:ident
      $(, $($rest:tt)*)?
    ) => {
        jnorm_type_then!{
            jnorm_args_then_push, ( $p ),
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
        __jnorm_arg_suffix!{
            jnorm_args_then_push, $cb, @ $mode,
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
        __jnorm_arg_suffix!(
            jnorm_args_then_push, $cb, @ $mode,
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
        __jnorm_arg_suffix!{
            jnorm_args_then_push, $cb, @ $mode,
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
        __jnorm_arg_suffix!{
            jnorm_args_then_push, $cb, @ $mode,
            ( $($acc)* ), ( $($pass)* ), $n,
            base( $p ), dims(),
            $($after)+
        }
    };
}

// Push one normalized arg into accumulator and continue munching.
macro_rules! jnorm_args_then_push {
    ( ( $($norm:tt)+ ), $cb:tt, @ $mode:ident, ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident, $($rest:tt)* ) => {
        jnorm_args_then_munch!{ @ $mode $cb, ( $($acc)* $n: $($norm)+ , ), ( $($pass)* ), $($rest)* }
    };
    ( ( $($norm:tt)+ ), $cb:tt, @ $mode:ident, ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident ) => {
        jnorm_args_then_munch!{ @ $mode $cb, ( $($acc)* $n: $($norm)+ ), ( $($pass)* ) }
    };
}

// ---------- helpers: presence of rust(...), building pieces ----------
macro_rules! _is_rust {
    ( prim ( $p:ident ) ) => {
        0
    };
    ( obj  ( $($t:tt)* ) as rust ( $($as:tt)+ ) ) => {
        0
    };
    ( rust ( $($t:tt)* ) ) => {
        1
    };
}

macro_rules! any_rust {
    // No args
    ( ( ), ( $rk:ident ( $($rt:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? ) ) => {
        _is_rust!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? )
    };
    // Args present
    ( ( $( $an:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* $(,)? ),
      ( $rk:ident ( $($rt:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? )
    ) => {{
        0
        $( | _is_rust!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ) )*
        | _is_rust!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? )
    }};
}

macro_rules! ret_desc_lit       {
    ( prim ( $p:ident ) ) => { jdesc!($p) };
    ( obj  ( $($j:tt)+ ) as rust ( $as:ty ) ) => { jdesc!($($j)+) };
    ( rust ( $($r:tt)+ ) ) => { "{}" };
}
macro_rules! ret_desc_fmt_piece {
    ( prim ( $p:ident ) ) => { jdesc!($p) };
    ( obj  ( $($j:tt)+ ) as rust ( $as:ty ) ) => { jdesc!($($j)+) };
    ( rust ( $($r:tt)+ ) ) => { "{}" };
}
macro_rules! ret_desc_fmt_val   {
    ( prim ( $p:ident ) ) => {};
    ( obj  ( $($j:tt)+ ) as rust ( $as:ty ) ) => {};
    ( rust ( $($r:tt)+ ) ) => { , rust_jdesc!($($r)+) };
}

// Parameter & return Rust types from normalized forms
macro_rules! _param_ty_from_norm {
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
macro_rules! _ret_ty_from_norm {
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

// Build signature (literal vs format) for NORMALIZED args/ret.

macro_rules! build_sig_literal {
    ( ( $( $n:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* )
      -> ( $rk:ident ( $($rt:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? )
    ) => {
        String::from(concat!(
            "(",
            $( jdesc!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ) ),*,
            ")",
            jdesc!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? ),
            "\0"
        ))
    };
}
macro_rules! build_sig_format {
    ( ( $( $n:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* )
      -> ( $rk:ident ( $($rt:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? )
    ) => {
        {
            let mut buf = String::new();
            fn extend_with_internal(buf: &mut String, s: &str) {
                if s.len() > 1 && s.chars().nth(0) != Some('[') {
                    buf.push('L');
                    buf.extend(s.chars().map(|c| if c == '.' { '/' } else { c }));
                    buf.push(';');
                } else {
                    buf.extend(s.chars().map(|c| if c == '.' { '/' } else { c }));
                }
            }
            buf.push_str("(");
            $( extend_with_internal(&mut buf, rust_jdesc!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? )); )*
            buf.push_str(")");
            extend_with_internal(&mut buf, rust_jdesc!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? ));
            buf.push_str("\0");
            buf
        }
    };
}

// ---------- helpers to build lookup signature from normalized args/ret ----------

// Given normalized args and ret, produce the final signature expr
macro_rules! _lookup_sig_from_norm {
    ( ( $($nargs:tt)* ), ( $($nret:tt)* ) ) => {
        if any_rust!( ( $($nargs)* ), ( $($nret)* ) ) == 0 {
            build_sig_literal!( ( $($nargs)* ) -> ( $($nret)* ) )
        } else {
            build_sig_format!( ( $($nargs)* ) -> ( $($nret)* ) )
        }
    };
}

// Glue: normalize ret next
macro_rules! _lookup_sig_emit {
    // Accept grouped normalized return
    ( ( $($norm_ret:tt)+ ), ( $($norm_args:tt)* ) ) => {
        _lookup_sig_from_norm!( ( $($norm_args)* ), ( $($norm_ret)+ ) )
    };
}
macro_rules! _lookup_sig_from_norm_args {
    ( ( $($norm_args:tt)* ), ( $($raw_ret:tt)+ ) ) => {
        jnorm_ret_then!( @ expr _lookup_sig_emit, ( $($raw_ret)+ ), ( $($norm_args)* ) )
    };
}

// ----- Convert an argument expression to a `jvalue` (by normalized kind) -----

// Strip leading & from a type so we can call Reference::as_raw on the referent
macro_rules! strip_ref_ty {
    ( & $inner:ty ) => {
        $inner
    };
    ( $t:ty ) => {
        $t
    };
}

// Get a `jobject` handle from a reference-y Rust wrapper (type can include &)
macro_rules! as_jobject_handle {
    ( $ty:ty, $expr:expr ) => {{
        <strip_ref_ty!($ty) as $crate::refs::Reference>::as_raw(($expr).as_ref())
    }};
}

// A single normalized arg → jni::sys::jvalue
macro_rules! _jvalue_of_norm_arg {
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

    // Java objects (normalized as "obj(...) as rust(Ty)") → object slot
    ( $name:ident : obj  ( $($j:tt)+ ) as rust ( $as_ty:ty ) ) => {
        jni::sys::jvalue {
            l: as_jobject_handle!($as_ty, $name),
        }
    };

    // Rust refs (normalized as "rust(Ty)") → object slot
    ( $name:ident : rust ( $as_ty:ty ) ) => {
        jni::sys::jvalue {
            l: as_jobject_handle!($as_ty, $name),
        }
    };
}

// Build `[jvalue; N]` from a normalized arg list
macro_rules! build_jni_args_array {
    ( ( $( $an:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* $(,)? ) ) => {{
        [ $( _jvalue_of_norm_arg!( $an : $ak($($at)*) $( as rust ( $($aas)+ ) )? ) ),* ]
    }};
}

// Expand a normalized args list into a typed parameter list
macro_rules! param_list_from_args {
    ( ( $( $an:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* $(,)? ) ) => {
        $( $an: _param_ty_from_norm!( $ak($($at)*) $( as rust ( $($aas)+ ) )? ) ),*
    };
}

// Normalize (args, ret) pair, then invoke callback that expects two groups:
//   $cb!( ( <normalized args> ), ( <normalized ret> ) )
macro_rules! jnorm_sig_then {
    ( $cb:ident, ( $($args:tt)* ), ( $($ret:tt)+ ) ) => {
        $cb!( ( jnorm_args!( $($args)* ) ), ( jnorm_ret!( $($ret)+ ) ) )
    };
    // Variant for callbacks that expect "args -> ret":
    ( $cb:ident, ( $($args:tt)* ) -> ( $($ret:tt)+ ) ) => {
        $cb!( ( jnorm_args!( $($args)* ) ) -> ( jnorm_ret!( $($ret)+ ) ) )
    };
}

// Helper to unwrap one layer of parentheses around a token tree
macro_rules! unwrap_parens {
    ( ( $($inner:tt)* ) ) => { $($inner)* };
}

// ----- LOOKUP FN -----
// Normalize args/ret before calling any_rust/build_sig_*
macro_rules! jgen_method_lookup_fn {
    (
        $this:path,
        $jname:ident,
        $rname:ident,
        ( $($args:tt)* ),
        ( $($ret:tt)+ )
    ) => {
        paste! {
            fn [<_ $rname _lookup>](env: &mut Env) -> Result<JMethodID> {
                let class: &JClass = $this::lookup_class()?;

                let sig = jnorm_args_then!( @expr _lookup_sig_from_norm_args, ( $($args)* ), ( $($ret)+ ) );

                env.get_method_id(class, stringify!($jname), &sig)
            }
        }
    };
}

// ---------- helpers to emit the call fn from normalized args/ret ----------
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

macro_rules! _jni_call_from_norm_ret {
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( void ) )
    ) => {
        unsafe {
            jni_call_check_ex!(
                $env,
                v1_1,
                CallVoidMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            );
            Ok(())
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jboolean ) )
    ) => {
        unsafe {
            let ret: jni::sys::jboolean = jni_call_check_ex!(
                $env,
                v1_1,
                CallBooleanMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )?;
            Ok(ret)
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jbyte ) )
    ) => {
        unsafe {
            let ret: jni::sys::jbyte = jni_call_check_ex!(
                $env,
                v1_1,
                CallByteMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )?;
            Ok(ret)
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jchar ) )
    ) => {
        unsafe {
            let ret: jni::sys::jchar = jni_call_check_ex!(
                $env,
                v1_1,
                CallCharMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )?;
            Ok(ret)
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jshort ) )
    ) => {
        unsafe {
            let ret: jni::sys::jshort = jni_call_check_ex!(
                $env,
                v1_1,
                CallShortMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )?;
            Ok(ret)
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jint ) )
    ) => {
        unsafe {
            let ret: jni::sys::jint = jni_call_check_ex!(
                $env,
                v1_1,
                CallIntMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )?;
            Ok(ret)
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jlong ) )
    ) => {
        unsafe {
            let ret: jni::sys::jlong = jni_call_check_ex!(
                $env,
                v1_1,
                CallLongMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )?;
            Ok(ret)
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jfloat ) )
    ) => {
        unsafe {
            let ret: jni::sys::jfloat = jni_call_check_ex!(
                $env,
                v1_1,
                CallFloatMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )?;
            Ok(ret)
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( prim ( jdouble ) )
    ) => {
        unsafe {
            let ret: jni::sys::jdouble = jni_call_check_ex!(
                $env,
                v1_1,
                CallDoubleMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )?;
            Ok(ret)
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( obj  ( $($rt:tt)+ ) as rust ( $as_ty:ty ) )
    ) => {
        unsafe {
            let ret_obj: jni::sys::jobject = jni_call_check_ex!(
                $env,
                v1_1,
                CallObjectMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )?;
            Ok(<$as_ty>::from_raw($env, ret_obj)?)
        }
    };
    (
        $env:expr, $this:expr, $method_id:expr, $jni_args:expr,
        ( rust ( $as_ty:ty ) )
    ) => {
        unsafe {
            let ret_obj: jni::sys::jobject = jni_call_check_ex!(
                $env,
                v1_1,
                CallObjectMethodA,
                ($this).as_ref().as_raw(),
                $method_id,
                $jni_args.as_ptr()
            )?;
            Ok(<$as_ty>::from_raw($env, ret_obj)?)
        }
    };
}

// Helper macro to determine if a normalized return type is primitive (including void)
macro_rules! _is_primitive_return {
    ( prim ( $p:ident ) ) => {
        true
    };
    ( obj ( $($rt:tt)+ ) as rust ( $($ret_as:tt)+ ) ) => {
        false
    };
    ( rust ( $($ret_as:tt)+ ) ) => {
        false
    };
}

// Final emitter: we now have normalized args and ret; generate the whole fn
macro_rules! _emit_method_call_fn {
    (
        $envType:ty,
        $this:path,
        $rname:ident,
        ( $( $an:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* $(,)? ),
        ( $rk:ident ( $($rt:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? )
    ) => {
        paste! {
            fn [<_ $rname _call>](
                env: $envType,
                this: &$this,
                method_id: JMethodID,
                $( $an: _param_ty_from_norm!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ) ),*
            ) -> Result< _ret_ty_from_norm!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? ) > {
                let jni_args = build_jni_args_array!( ( $( $an : $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ),* ) );

                let _ = &this;

                _jni_call_from_norm_ret!{
                    env,
                    this,
                    method_id,
                    &jni_args,
                    ( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? )
                }
            }
        }
    };
}

// Glue: normalize ret after args are normalized
macro_rules! _emit_call_fn {
    // Primitive return types (including void) use &Env
    (
        ( prim ( $($p:tt)+ ) ),
        $this:path,
        $rname:ident,
        ( $( $nargs:tt )* )
    ) => {
        _emit_method_call_fn!( &Env, $this, $rname, ( $( $nargs )* ), ( prim( $($p)+ ) ) );
    };
    // Object return types use &mut Env
    (
        ( obj ( $($rt:tt)+ ) as rust ( $($ret_as:tt)+ ) ),
        $this:path,
        $rname:ident,
        ( $( $nargs:tt )* )
    ) => {
        _emit_method_call_fn!( &mut Env, $this, $rname, ( $( $nargs )* ), ( obj( $($rt)+ ) as rust( $($ret_as)+ ) ) );
    };
    // Rust return types use &mut Env
    (
        ( rust ( $($ret_as:tt)+ ) ),
        $this:path,
        $rname:ident,
        ( $( $nargs:tt )* )
    ) => {
        _emit_method_call_fn!( &mut Env, $this, $rname, ( $( $nargs )* ), ( rust( $($ret_as)+ ) ) );
    };
}

macro_rules! _emit_call_from_norm_args {
    (
        ( $( $nargs:tt )* ),
        $this:path,
        $rname:ident,
        ( $($raw_ret:tt)+ )
    ) => {
        jnorm_ret_then!( @ item _emit_call_fn, ( $($raw_ret)+ ), $this, $rname, ( $( $nargs )* ) );
    };
}

// ----- CALL FN (normalize first, then emit) -----
macro_rules! jgen_method_call_fn {
    (
        $this:path,
        $rname:ident,
        ( $($args:tt)* ),
        ( $($ret:tt)+ )
    ) => {
        jnorm_args_then!( @item _emit_call_from_norm_args, ( $($args)* ), $this, $rname, ( $($ret)+ ) );
    };
}

// ---------- Public: jgen_bind_method ----------
/*
Usage:
jgen_bind_method!(
    this: JFoo,
    javaMethodName as rust_method_name,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> void
);

outputs:

fn _rust_method_name_lookup(env: &mut Env) -> Result<JMethodID> {
    let class: &JClass = JFoo::lookup_class()?;
    let sig = if /* all literal */ {
        build_sig_literal!((a: rust(&JString) as &JString, b: obj(java.lang.String) as JString, c: prim(jint) as jint) -> (prim(void) as void))
    } else {
        build_sig_format!((a: rust(&JString) as &JString, b: obj(java.lang.String) as JString, c: prim(jint) as jint) -> (prim(void) as void))
    };
    // Use `class`, `sig`, and the method name to look up the method ID
    env.get_method_id(class, "javaMethodName", &sig)
}

fn _rust_method_name_call(env: Env, this: &JFoo, method_id: JMethodID, a: &JString, b: &JString, c: jint) -> Result<void> {
    let class: &JClass = JFoo::lookup_class()?;
    jni_call_check_ex!(self, v1_1, CallVoidMethodA, obj, method_id, jni_args)
}
*/
macro_rules! jgen_bind_method {
    (
        this: $this:path,
        $jname:ident as $rname:ident,
        sig: ( $($args:tt)* ) -> $($rty:tt)+
    ) => {

        jgen_method_lookup_fn!(
            $this,
            $jname,
            $rname,
            ( $($args)* ),
            ( $($rty)+ )
        );
        jgen_method_call_fn!(
            $this,
            $rname,
            ( $($args)* ),
            ( $($rty)+ )
        );
    };
}

// ------------------
// EXAMPLES / TESTS *
// ------------------

// ----------- Minimal placeholders so this compiles in isolation -----------
use std::{borrow::Cow, ffi::CStr, fmt};

type Result<T> = core::result::Result<T, ()>;
type JMethodID = *mut jni::sys::_jmethodID;

#[derive(Default)]
struct JFoo;

impl AsRef<JFoo> for JFoo {
    fn as_ref(&self) -> &JFoo {
        self
    }
}

impl Reference for JFoo {
    fn class_name() -> Cow<'static, str> {
        Cow::Borrowed("com/example/JFoo")
    }

    fn as_raw(&self) -> jni::sys::jobject {
        std::ptr::null_mut()
    }
}
#[derive(Default)]
struct Env;

impl Env {
    fn get_method_id(&mut self, _class: &JClass, _name: &str, _sig: &str) -> Result<JMethodID> {
        Ok(std::ptr::null_mut())
    }

    fn get_raw(&self) -> *mut jni::sys::JNIEnv {
        std::ptr::null_mut()
    }

    fn exception_check(&self) -> bool {
        false
    }
}

static GLOBAL_JCLASS: JClass = JClass;
impl JFoo {
    fn lookup_class() -> Result<&'static JClass> {
        Ok(&GLOBAL_JCLASS)
    }
}

struct JNIStr;
impl fmt::Display for JNIStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JNI_STR")
    }
}

mod sys {
    pub use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort, jvalue};
}
mod refs {
    pub(crate) use super::Reference;
}
trait Reference {
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

const _: &str = jdesc!(prim(jint)); // "I"
const _: &str = jdesc!(prim(void)); // "V"

// 2) Objects and arrays (normalized)
const _: &str = jdesc!(obj(java.lang.String) as rust(JObject)); // "Ljava/lang/String;"
const _: &str = jdesc!(obj([java.lang.String]) as rust(JObjectArray<JObject>)); // "[Ljava/lang/String;"
const _: &str = jdesc!(obj([[java.lang.String]]) as rust(JObjectArray<JString>)); // "[Ljava/lang/String;"

const _: &str = jdesc!(obj([[java.lang.String]]) as rust(JObjectArray<JObjectArray<JObject>>)); // "[[Ljava/lang/String;"

const _: &str = jdesc!(obj(java.util.Map::Entry) as rust(JObject)); // "Ljava/util/Map$Entry;"
const _: &str = jdesc!(obj([java.util.Map::Entry]) as rust(JObjectArray<JObject>)); // "[Ljava/util/Map$Entry;"
const _: &str = jdesc!(obj(.Hello) as rust(JObject)); // "LHello;"
const _: &str = jdesc!(obj([.Hello]) as rust(JObjectArray<JObject>)); // "[LHello;"
const _: &str = jdesc!(obj([jchar]) as rust(JPrimitiveArray<jchar>)); // "[C"

// 1) Primitives (with and without 'j')
const _: &str = jdesc_of!(jint); // "I"
const _: &str = jdesc_of!(int); // "I"
const _: &str = jdesc_of!(void); // "V"

// 2) Objects and arrays (internal names)
const _: &str = jdesc_of!(java.lang.String); // "Ljava/lang/String;"
const _: &str = jdesc_of!([java.lang.String]); // "[Ljava/lang/String;"
const _: &str = jdesc_of!([[java.lang.String]]); // "[[Ljava/lang/String;"
const _: &str = jdesc_of!(java.util.Map::Entry); // "Ljava/util/Map$Entry;"
const _: &str = jdesc_of!([java.util.Map::Entry]); // "[Ljava/util/Map$Entry;"
const _: &str = jdesc_of!(.Hello); // "LHello;"
const _: &str = jdesc_of!([.Hello]); // "[LHello;"
const _: &str = jdesc_of!(char); // "C"
const _: &str = jdesc_of!([char]); // "[C"

const _: &str = jdesc_of!(java.lang.String); // "Ljava/lang/String;"
const _: &str = jdesc_of!([[java.lang.String]]); // "[Ljava/lang/String;"

// -------------------- Example --------------------

jgen_bind_method!(
    this: JFoo,
    javaFunction as rust_function_0,
    sig: (a: java.lang.String, c: jint) -> void
);

jgen_bind_method!(
    this: JFoo,
    javaFunction as rust_function_1,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> jint
);

jgen_bind_method!(
    this: JFoo,
    javaFunction as rust_function_2,
    sig: (a: &JString, b: java.lang.String[], c: jint) -> void
);

jgen_bind_method!(
    this: JFoo,
    javaFunction as rust_function_3,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> JString
);

jgen_bind_method!(
    this: JFoo,
    javaFunction as rust_function_4,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> crate::objects::JString
);

jgen_bind_method!(
    this: JFoo,
    javaFunction as rust_function_5,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> java.lang.String
);

jgen_bind_method!(
    this: JFoo,
    javaFunction as rust_function_6,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> java.lang.String[]
);
jgen_bind_method!(
    this: JFoo,
    javaFunction as rust_function_7,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> java.lang.String[] as JObjectArray<JString>
);

jgen_bind_method!(
    this: JFoo,
    javaFunction as rust_function_8,
    sig: (a: &JString, b: java.lang.String[], c: jint[]) -> void
);

// Test multi-dimensional primitive arrays with rust-like syntax
jgen_bind_method!(
    this: JFoo,
    javaFunction as rust_function_10,
    sig: (a: &[[jint]], b: &[[[jchar]]], c: &[[jdouble]]) -> &[[jbyte]]
);

// Test macro that actually calls the real normalization functions
macro_rules! print_jni_sig_for {
    ( ( $($args:tt)* ) -> $($ret:tt)+ ) => {
        {
            let sig = jnorm_args_then!( @expr _lookup_sig_from_norm_args, ( $($args)* ), ( $($ret)+ ) );
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
    - Output only one literal or format signature without a runtime if statement.
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
    println!("jint[] -> {}", jdesc_of!(jint[]));
    println!("jbyte[] -> {}", jdesc_of!(jbyte[]));
    println!("jchar[] -> {}", jdesc_of!(jchar[]));

    println!("\n2D Primitive array descriptors:");
    println!("jint[][] -> {}", jdesc_of!(jint[][]));
    println!("jbyte[][] -> {}", jdesc_of!(jbyte[][]));
    println!("jchar[][] -> {}", jdesc_of!(jchar[][]));

    println!("\n3D Primitive array descriptors:");
    println!("jint[][][] -> {}", jdesc_of!(jint[][][]));
    println!("jchar[][][] -> {}", jdesc_of!(jchar[][][]));

    println!("\nRust-like syntax descriptors:");
    println!("&[[jint]] -> {}", jdesc_of!(&[[jint]]));
    println!("&[[[jchar]]] -> {}", jdesc_of!(&[[[jchar]]]));

    println!("\nBracket syntax descriptors:");
    println!("[[jint]] -> {}", jdesc_of!([[jint]]));
    println!("[[[jchar]]] -> {}", jdesc_of!([[[jchar]]]));

    let foo = JFoo::default();
    let mut env = Env::default();

    // Don't call these, just make sure they compile
    let _ = || {
        // Test a primitive return function (should use &Env)
        let method_0 = _rust_function_0_lookup(&mut env).unwrap();
        let _void_ret = _rust_function_0_call(
            &env,
            &foo,
            method_0,
            &JObject::default(), // java.lang.String normalized as JObject
            42,
        )
        .unwrap();

        // Test an object return function (should use &mut Env)
        let method_6 = _rust_function_6_lookup(&mut env).unwrap();
        let _obj_ret = _rust_function_6_call(
            &mut env, // Note: &mut Env for object return
            &foo,
            method_6,
            &JString::default(),
            &JString::default(),
            42,
        )
        .unwrap();
    };
}
