// Import the `paste!` macro
use paste::paste;

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
    ( $($norm:tt)+ ) => { jdesc!( $($norm)+ ) };
}
macro_rules! jdesc_of {
    ( $($raw:tt)+ ) => {
        jnorm_type_then!( jdesc_of_emit, ( $($raw)+ ) )
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
    ( [ jboolean ] ) => { JPrimitiveArray<jboolean> };
    ( [ jbyte    ] ) => { JPrimitiveArray<jbyte>    };
    ( [ jchar    ] ) => { JPrimitiveArray<jchar>    };
    ( [ jshort   ] ) => { JPrimitiveArray<jshort>   };
    ( [ jint     ] ) => { JPrimitiveArray<jint>     };
    ( [ jlong    ] ) => { JPrimitiveArray<jlong>    };
    ( [ jfloat   ] ) => { JPrimitiveArray<jfloat>   };
    ( [ jdouble  ] ) => { JPrimitiveArray<jdouble>  };
    ( [ boolean  ] ) => { JPrimitiveArray<jboolean> };
    ( [ byte     ] ) => { JPrimitiveArray<jbyte>    };
    ( [ char     ] ) => { JPrimitiveArray<jchar>    };
    ( [ short    ] ) => { JPrimitiveArray<jshort>   };
    ( [ int      ] ) => { JPrimitiveArray<jint>     };
    ( [ long     ] ) => { JPrimitiveArray<jlong>    };
    ( [ float    ] ) => { JPrimitiveArray<jfloat>   };
    ( [ double   ] ) => { JPrimitiveArray<jdouble>  };

    // base: 1D object arrays (named package or default package)
    ( [ $first:ident . $($rest:tt)+ ] ) => { JObjectArray<JObject> };
    ( [ . $outer:ident $( :: $inner:ident )* ] ) => { JObjectArray<JObject> };
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
//   - .NoPackage -> obj(NoPackage) as rust(JObject)
//   - .NoPackage as JString -> obj(NoPackage) as rust(JString)
//   - java.lang.String as JString -> obj(java.lang.String) as rust(JString)
//   - &JString -> rust(JString)
//   - &[JString] -> rust(JObjectArray<JString>)
//   - &[[JString]] -> rust(JObjectArray<JObjectArray<JString>>)
//   - &[jint] -> rust(JPrimitiveArray<jint>)
// ...existing code...
macro_rules! jnorm_type_then {
    // ----- Rust reference arrays -----
    ( $cb:tt, ( & [ [ $($inner:tt)+ ] ] ) $(, $($pass:tt)* )? ) => {
        jnorm_type_then!(@objarr $cb, () (), [ [ $($inner)+ ] ] $(, $($pass)* )? )
    };
    ( $cb:tt, ( & [ $ty:path ] ) $(, $($pass:tt)* )? ) => {
        $cb!( rust( JObjectArray<$ty> ) $(, $($pass)* )? )
    };
    ( $cb:tt, ( & [ & $ty:path ] ) $(, $($pass:tt)* )? ) => {
        $cb!( rust( JObjectArray<$ty> ) $(, $($pass)* )? )
    };
    // & [primitive]
    ( $cb:tt, ( & [ jboolean ] ) $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jboolean> ) $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ boolean ] )  $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jboolean> ) $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ jbyte ] )    $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jbyte> )    $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ byte ] )     $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jbyte> )    $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ jchar ] )    $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jchar> )    $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ char ] )     $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jchar> )    $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ jshort ] )   $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jshort> )   $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ short ] )    $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jshort> )   $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ jint ] )     $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jint> )     $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ int ] )      $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jint> )     $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ jlong ] )    $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jlong> )    $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ long ] )     $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jlong> )    $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ jfloat ] )   $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jfloat> )   $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ float ] )    $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jfloat> )   $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ jdouble ] )  $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jdouble> )  $(, $($pass)* )? ) };
    ( $cb:tt, ( & [ double ] )   $(, $($pass:tt)* )? ) => { $cb!( rust( JPrimitiveArray<jdouble> )  $(, $($pass)* )? ) };

    // Reject multi-dimensional primitive arrays by the &-array path only
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ $p:ident ] $(, $($pass:tt)* )? ) => {
        compile_error!("Multi-dimensional primitive arrays are not supported")
    };
    // Object array recursion for &[[...]]
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ [ $($inner:tt)+ ] ] $(, $($pass:tt)* )? ) => {
        jnorm_type_then!(@objarr $cb, ( $($open)* JObjectArray< ) ( > $($close)* ), [ $($inner)+ ] $(, $($pass)* )? )
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ $ty:path ] $(, $($pass:tt)* )? ) => {
        $cb!( rust( $($open)* $ty $($close)* ) $(, $($pass)* )? )
    };
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ & $ty:path ] $(, $($pass:tt)* )? ) => {
        $cb!( rust( $($open)* $ty $($close)* ) $(, $($pass)* )? )
    };

    // ----- Java-style arrays (non-Rust), N-dimensional -----

    // Bracket form with explicit `as` (any dimension)
    ( $cb:tt, ( [ $($arr:tt)+ ] as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!( obj( [ $($arr)+ ] ) as rust( $as_ty ) $(, $($pass)* )? )
    };
    // Bracket form without `as` (any dimension) → compute default Rust wrapper type
    ( $cb:tt, ( [ $($arr:tt)+ ] ) $(, $($pass:tt)* )? ) => {
        $cb!( obj( [ $($arr)+ ] ) as rust( rust_type_for_java_array_default!([ $($arr)+ ]) ) $(, $($pass)* )? )
    };

    // Suffix form → rewrite to bracket form
    ( $cb:tt, ( $first:ident . $($rest:tt)+ [ ] $($tail:tt)* ) $(, $($pass:tt)* )? ) => {
        jnorm_type_then!(@suffix $cb, ( [ $first . $($rest)+ ] ), $($tail)* $(, $($pass)* )? )
    };
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* [ ] $($tail:tt)* ) $(, $($pass:tt)* )? ) => {
        jnorm_type_then!(@suffix $cb, ( [ . $outer $( :: $inner )* ] ), $($tail)* $(, $($pass)* )? )
    };
    ( $cb:tt, ( $p:ident [ ] $($tail:tt)* ) $(, $($pass:tt)* )? ) => {
        jnorm_type_then!(@suffix $cb, ( [ $p ] ), $($tail)* $(, $($pass)* )? )
    };
    // Internal suffix recursion
    ( @suffix $cb:tt, ( [ $($acc:tt)+ ] ), [ ] $($tail:tt)* $(, $($pass:tt)* )? ) => {
        jnorm_type_then!(@suffix $cb, ( [ [ $($acc)+ ] ] ), $($tail)* $(, $($pass)* )? )
    };
    ( @suffix $cb:tt, ( [ $($acc:tt)+ ] ), as $as_ty:ty $(, $($pass:tt)* )? ) => {
        $cb!( obj( [ $($acc)+ ] ) as rust( $as_ty ) $(, $($pass)* )? )
    };
    ( @suffix $cb:tt, ( [ $($acc:tt)+ ] ), $(, $($pass:tt)* )? ) => {
        $cb!( obj( [ $($acc)+ ] ) as rust( rust_type_for_java_array_default!([ $($acc)+ ]) ) $(, $($pass)* )? )
    };

    // ----- Simple Rust ref: &Path -----
    ( $cb:tt, ( & $rust:ty ) $(, $($pass:tt)* )? ) => { $cb!( rust( $rust ) $(, $($pass)* )? ) };

    // ----- Java object type (named package) -----
    ( $cb:tt, ( $first:ident . $($rest:tt)+ as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!( obj( $first . $($rest)+ ) as rust( $as_ty ) $(, $($pass)* )? )
    };
    ( $cb:tt, ( $first:ident . $($rest:tt)+ ) $(, $($pass:tt)* )? ) => {
        $cb!( obj( $first . $($rest)+ ) as rust( JObject ) $(, $($pass)* )? )
    };

    // ----- Java object type (default package) -----
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!( obj( . $outer $( :: $inner )* ) as rust( $as_ty ) $(, $($pass)* )? )
    };
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* ) $(, $($pass:tt)* )? ) => {
        $cb!( obj( . $outer $( :: $inner )* ) as rust( JObject ) $(, $($pass)* )? )
    };

    // ----- Primitives (canonicalize) -----
    ( $cb:tt, ( void )     $(, $($pass:tt)* )? ) => { $cb!( prim( void )     $(, $($pass)* )? ) };
    ( $cb:tt, ( jboolean ) $(, $($pass:tt)* )? ) => { $cb!( prim( jboolean ) $(, $($pass)* )? ) };
    ( $cb:tt, ( boolean )  $(, $($pass:tt)* )? ) => { $cb!( prim( jboolean ) $(, $($pass)* )? ) };
    ( $cb:tt, ( jbyte )    $(, $($pass:tt)* )? ) => { $cb!( prim( jbyte )    $(, $($pass)* )? ) };
    ( $cb:tt, ( byte )     $(, $($pass:tt)* )? ) => { $cb!( prim( jbyte )    $(, $($pass)* )? ) };
    ( $cb:tt, ( jchar )    $(, $($pass:tt)* )? ) => { $cb!( prim( jchar )    $(, $($pass)* )? ) };
    ( $cb:tt, ( char )     $(, $($pass:tt)* )? ) => { $cb!( prim( jchar )    $(, $($pass)* )? ) };
    ( $cb:tt, ( jshort )   $(, $($pass:tt)* )? ) => { $cb!( prim( jshort )   $(, $($pass)* )? ) };
    ( $cb:tt, ( short )    $(, $($pass:tt)* )? ) => { $cb!( prim( jshort )   $(, $($pass)* )? ) };
    ( $cb:tt, ( jint )     $(, $($pass:tt)* )? ) => { $cb!( prim( jint )     $(, $($pass)* )? ) };
    ( $cb:tt, ( int )      $(, $($pass:tt)* )? ) => { $cb!( prim( jint )     $(, $($pass)* )? ) };
    ( $cb:tt, ( jlong )    $(, $($pass:tt)* )? ) => { $cb!( prim( jlong )    $(, $($pass)* )? ) };
    ( $cb:tt, ( long )     $(, $($pass:tt)* )? ) => { $cb!( prim( jlong )    $(, $($pass)* )? ) };
    ( $cb:tt, ( jfloat )   $(, $($pass:tt)* )? ) => { $cb!( prim( jfloat )   $(, $($pass)* )? ) };
    ( $cb:tt, ( float )    $(, $($pass:tt)* )? ) => { $cb!( prim( jfloat )   $(, $($pass)* )? ) };
    ( $cb:tt, ( jdouble )  $(, $($pass:tt)* )? ) => { $cb!( prim( jdouble )  $(, $($pass)* )? ) };
    ( $cb:tt, ( double )   $(, $($pass:tt)* )? ) => { $cb!( prim( jdouble )  $(, $($pass)* )? ) };

    // ----- Already-normalized (idempotent) -----
    ( $cb:tt, ( prim ( $($p:tt)+ ) ) $(, $($pass:tt)* )? ) => { $cb!( prim( $($p)+ ) $(, $($pass)* )? ) };
    ( $cb:tt, ( obj ( $($j:tt)+ ) as rust ( $as:ty ) ) $(, $($pass:tt)* )? ) => { $cb!( obj( $($j)+ ) as rust( $as ) $(, $($pass)* )? ) };
    ( $cb:tt, ( rust ( $as:ty ) ) $(, $($pass:tt)* )? ) => { $cb!( rust( $as ) $(, $($pass)* )? ) };
}

// Rework jnorm_ret_then to use jnorm_type_then (variadic; forwards extra tokens)
macro_rules! jnorm_ret_then {
    ( $cb:ident, ( $($ret:tt)+ ) $(, $($pass:tt)* )? ) => {
        jnorm_type_then!( $cb, ( $($ret)+ ) $(, $($pass)* )? )
    };
}

// Normalize an args list, then invoke a callback macro that expects normalized args:
//   jnorm_args_then!(CB, ( a: TyA, b: TyB, ... ) ) → CB!( ( a: <normA>, b: <normB>, ... ) )
macro_rules! jnorm_args_then {
    ( $cb:ident, ( $($args:tt)* ) ) => {
        jnorm_args_then_munch!( $cb, (), $($args)* )
    };
}

// Internal muncher that uses jnorm_type_then per argument type and accumulates a normalized list
/*
macro_rules! jnorm_args_then_munch {
    // End of list
    ( $cb:ident, ( $($acc:tt)* ) ) => { $cb!( ( $($acc)* ) ) };

    // With trailing comma
    ( $cb:ident, ( $($acc:tt)* ), $n:ident : $($ty:tt)+ , $($rest:tt)* ) => {
        jnorm_type_then!( jnorm_args_then_push, ( $($ty)+ ), $cb, ( $($acc)* ), $n, $($rest)* )
    };
    // Last element
    ( $cb:ident, ( $($acc:tt)* ), $n:ident : $($ty:tt)+ ) => {
        jnorm_type_then!( jnorm_args_then_push, ( $($ty)+ ), $cb, ( $($acc)* ), $n )
    };
}
    */
macro_rules! jnorm_args_then_munch {
    // End of list
    ( $cb:tt, ( $($acc:tt)* ) ) => { $cb!( ( $($acc)* ) ) };

    // ---------- With trailing comma ----------
    // &[[...]],
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : & [ [ $($inner:tt)+ ] ] , $($rest:tt)* ) => {
        jnorm_type_then!( jnorm_args_then_push, ( & [ [ $($inner)+ ] ] ), $cb, ( $($acc)* ), $n, $($rest)* )
    };
    // &[...],
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : & [ $($inner:tt)+ ] , $($rest:tt)* ) => {
        jnorm_type_then!( jnorm_args_then_push, ( & [ $($inner)+ ] ), $cb, ( $($acc)* ), $n, $($rest)* )
    };
    // &Path,
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : & $rust:ty , $($rest:tt)* ) => {
        jnorm_type_then!( jnorm_args_then_push, ( & $rust ), $cb, ( $($acc)* ), $n, $($rest)* )
    };
    // java.name with as,
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : $first:ident . $($restname:tt)+ as $as_ty:ty , $($rest:tt)* ) => {
        jnorm_type_then!( jnorm_args_then_push, ( $first . $($restname)+ as $as_ty ), $cb, ( $($acc)* ), $n, $($rest)* )
    };
    // java.name,
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : $first:ident . $($restname:tt)+ , $($rest:tt)* ) => {
        jnorm_type_then!( jnorm_args_then_push, ( $first . $($restname)+ ), $cb, ( $($acc)* ), $n, $($rest)* )
    };
    // .Default::Inner with as,
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : . $outer:ident $( :: $inner:ident )* as $as_ty:ty , $($rest:tt)* ) => {
        jnorm_type_then!( jnorm_args_then_push, ( . $outer $( :: $inner )* as $as_ty ), $cb, ( $($acc)* ), $n, $($rest)* )
    };
    // .Default::Inner,
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : . $outer:ident $( :: $inner:ident )* , $($rest:tt)* ) => {
        jnorm_type_then!( jnorm_args_then_push, ( . $outer $( :: $inner )* ), $cb, ( $($acc)* ), $n, $($rest)* )
    };
    // primitive,
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : $p:ident , $($rest:tt)* ) => {
        jnorm_type_then!( jnorm_args_then_push, ( $p ), $cb, ( $($acc)* ), $n, $($rest)* )
    };
    // ---------- Last element (no trailing comma) ----------
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : & [ [ $($inner:tt)+ ] ] ) => {
        jnorm_type_then!( jnorm_args_then_push, ( & [ [ $($inner)+ ] ] ), $cb, ( $($acc)* ), $n )
    };
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : & [ $($inner:tt)+ ] ) => {
        jnorm_type_then!( jnorm_args_then_push, ( & [ $($inner)+ ] ), $cb, ( $($acc)* ), $n )
    };
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : & $rust:ty ) => {
        jnorm_type_then!( jnorm_args_then_push, ( & $rust ), $cb, ( $($acc)* ), $n )
    };
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : $first:ident . $($restname:tt)+ as $as_ty:ty ) => {
        jnorm_type_then!( jnorm_args_then_push, ( $first . $($restname)+ as $as_ty ), $cb, ( $($acc)* ), $n )
    };
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : $first:ident . $($restname:tt)+ ) => {
        jnorm_type_then!( jnorm_args_then_push, ( $first . $($restname)+ ), $cb, ( $($acc)* ), $n )
    };
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : . $outer:ident $( :: $inner:ident )* as $as_ty:ty ) => {
        jnorm_type_then!( jnorm_args_then_push, ( . $outer $( :: $inner )* as $as_ty ), $cb, ( $($acc)* ), $n )
    };
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : . $outer:ident $( :: $inner:ident )* ) => {
        jnorm_type_then!( jnorm_args_then_push, ( . $outer $( :: $inner )* ), $cb, ( $($acc)* ), $n )
    };
    ( $cb:tt, ( $($acc:tt)* ), $n:ident : $p:ident ) => {
        jnorm_type_then!( jnorm_args_then_push, ( $p ), $cb, ( $($acc)* ), $n )
    };
}

// Push one normalized arg into accumulator and continue munching
macro_rules! jnorm_args_then_push {
    ( $($norm:tt)+, $cb:ident, ( $($acc:tt)* ), $n:ident, $($rest:tt)* ) => {
        jnorm_args_then_munch!( $cb, ( $($acc)* $n: $($norm)+ , ), $($rest)* )
    };
    ( $($norm:tt)+, $cb:ident, ( $($acc:tt)* ), $n:ident ) => {
        jnorm_args_then_munch!( $cb, ( $($acc)* $n: $($norm)+ ) )
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
    ( obj ( $(_j:tt)+ ) as rust ( $as_ty:ty ) ) => {
        &$as_ty
    };
    ( rust ( $as_ty:ty ) ) => {
        $as_ty
    };
}
macro_rules! _ret_ty_from_norm {
    ( prim ( void ) ) => {
        ()
    };
    ( prim ( $p:ident ) ) => {
        $crate::sys::$p
    };
    ( obj ( $(_j:tt)+ ) as rust ( $as_ty:ty ) ) => {
        $as_ty
    };
    ( rust ( $as_ty:ty ) ) => {
        $as_ty
    };
}

// Build signature (literal vs format) for NORMALIZED args/ret.
macro_rules! _desc_piece {
    ( $k:ident ( $($t:tt)* ) $( as rust ( $($as:tt)+ ) )? ) => {
        jdesc!( $k ( $($t)* ) $( as rust ( $($as)+ ) )? )
    };
}
macro_rules! _desc_val {
    ( prim ( $($t:tt)* ) ) => {};
    ( obj  ( $($j:tt)+ ) as rust ( $as:ty ) ) => {};
    ( rust ( $as:ty ) ) => { , rust_jdesc!($as) };
}

macro_rules! build_sig_literal {
    ( ( $( $n:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* )
      -> ( $rk:ident ( $($rt:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? )
    ) => {
        concat!(
            "(",
            $( _desc_piece!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ) ),*,
            ")",
            _desc_piece!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? )
        )
    };
}
macro_rules! build_sig_format {
    ( ( $( $n:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* )
      -> ( $rk:ident ( $($rt:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? )
    ) => {
        format!(
            concat!(
                "(",
                $( _desc_piece!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ) ),*,
                ")",
                _desc_piece!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? )
            )
            $( _desc_val!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ) )*
            _desc_val!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? )
        )
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
        <strip_ref_ty!($ty) as $crate::refs::Reference>::as_raw($expr)
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

// ----- Select the correct CallXxxMethodA for the return type -----
macro_rules! _call_api_for_ret {
    ( prim ( void     ) ) => {
        CallVoidMethodA
    };
    ( prim ( jboolean ) ) => {
        CallBooleanMethodA
    };
    ( prim ( jbyte    ) ) => {
        CallByteMethodA
    };
    ( prim ( jchar    ) ) => {
        CallCharMethodA
    };
    ( prim ( jshort   ) ) => {
        CallShortMethodA
    };
    ( prim ( jint     ) ) => {
        CallIntMethodA
    };
    ( prim ( jlong    ) ) => {
        CallLongMethodA
    };
    ( prim ( jfloat   ) ) => {
        CallFloatMethodA
    };
    ( prim ( jdouble  ) ) => {
        CallDoubleMethodA
    };

    // Any reference-like return → object
    ( obj  ( $($rt:tt)+ ) $( as rust ( $($as:tt)+ ) )? ) => {
        CallObjectMethodA
    };
    ( rust ( $($r:tt)+ ) ) => {
        CallObjectMethodA
    };
}

// Helper to unwrap one layer of parentheses around a token tree
macro_rules! unwrap_parens {
    ( ( $($inner:tt)* ) ) => { $($inner)* };
}

// ----- LOOKUP FN -----
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

                let sig = if any_rust!( ( $($args)* ), ( $($ret)+ ) ) == 0 {
                    build_sig_literal!( ( $($args)* ) -> ( $($ret)+ ) )
                } else {
                    build_sig_format!( ( $($args)* ) -> ( $($ret)+ ) )
                };

                env.get_method_id(class, stringify!($jname), &sig)
            }
        }
    };
}

// ----- CALL FN (build jvalue[] and pick the API) -----
macro_rules! jgen_method_call_fn {
    (
        $this:path,
        $rname:ident,
        ( $($args:tt)* ),
        ( $($ret:tt)+ )
    ) => {
        paste! {
            fn [<_ $rname _call>](
                env: &mut Env,
                this: &$this,
                method_id: JMethodID,
                jnorm_args_then!(param_list_from_args, ( $($args)* ))
            ) -> Result< jnorm_ret_then!(_ret_ty_from_norm, ( $($ret)+ )) > {
                let jni_args = jnorm_args_then!(build_jni_args_array, ( $($args)* ));

                let _ = &this;
                /*
                jni_call_check_ex!(
                    this, v1_1,
                    jnorm_ret_then!(_call_api_for_ret, ( $($ret)+ )),
                    obj,
                    method_id,
                    &jni_args
                )
                */
                todo!()
            }
        }
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
type JMethodID = ();

#[derive(Default)]
struct JString;
#[derive(Default)]
struct JClass;
#[derive(Default)]
struct JFoo;
#[derive(Default)]
struct Env;

impl Env {
    fn get_method_id(&mut self, _class: &JClass, _name: &str, _sig: &str) -> Result<JMethodID> {
        Ok(())
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

trait Reference {
    fn class_name() -> Cow<'static, CStr>;
    fn as_raw(&self) -> jni::sys::jobject;
}

impl Reference for JString {
    fn class_name() -> Cow<'static, CStr> {
        Cow::Borrowed(c"java/lang/String")
    }
    fn as_raw(&self) -> jni::sys::jobject {
        core::ptr::null_mut()
    }
}

macro_rules! jni_call_check_ex {
    ($this:expr, $ver:ident, $api:ident, $obj:ident, $mid:expr, $args:expr) => {{
        let _ = (
            $this,
            stringify!($ver),
            stringify!($api),
            stringify!($obj),
            $mid,
            $args,
        );
        Ok(Default::default())
    }};
}

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

// 3) Normalization
// name: prim(jchar) as jchar
const _: () = {
    let _ = stringify!(jnorm_arg!(c: jchar));
};
// name: obj(java.lang.String) as JObject
const _: () = {
    let _ = stringify!(jnorm_arg!(s: java.lang.String));
};
// name: obj(java.lang.String) as JString
const _: () = {
    let _ = stringify!(jnorm_arg!(s: java.lang.String as JString));
};
// name: rust(&JString) as &JString
const _: () = {
    let _ = stringify!(jnorm_arg!(s: &JString));
};
// list form
const _: () = {
    let _ = stringify!(jnorm_args!(a: int, b: java.util.Map::Entry, c: &JString));
};

// Keep primitive alias used in examples
#[allow(non_camel_case_types)]
type jint = i32;

// -------------------- Example --------------------
/*
jgen_bind_method!(
    this: JFoo,
    javaFunction as rust_function_a,
    sig: (a: java.lang.String, c: jint) -> void
);

jgen_bind_method!(
    this: JFoo,
    javaFunction as rust_function,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> void
);
*/

fn main() {}
