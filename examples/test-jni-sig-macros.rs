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
    // Accept grouped normalized type
    ( ( $($norm:tt)+ ) ) => { jdesc!( $($norm)+ ) };
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

    // Reject multi-dimensional primitive arrays by the &-array path only
    ( @objarr $cb:tt, ( $($open:tt)* ) ( $($close:tt)* ), [ $p:ident ] $(, $($pass:tt)* )? ) => {
        compile_error!("Multi-dimensional primitive arrays are not supported")
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

    // Suffix form → rewrite to bracket form
    ( $cb:tt, ( $first:ident . $($rest:tt)+ [ ] $($tail:tt)* ) $(, $($pass:tt)* )? ) => {
        jnorm_type_then!{@suffix $cb, ( [ $first . $($rest)+ ] ), $($tail)* $(, $($pass)* )? }
    };
    ( $cb:tt, ( . $outer:ident $( :: $inner:ident )* [ ] $($tail:tt)* ) $(, $($pass:tt)* )? ) => {
        jnorm_type_then!{@suffix $cb, ( [ . $outer $( :: $inner )* ] ), $($tail)* $(, $($pass)* )? }
    };
    ( $cb:tt, ( $p:ident [ ] $($tail:tt)* ) $(, $($pass:tt)* )? ) => {
        jnorm_type_then!{@suffix $cb, ( [ $p ] ), $($tail)* $(, $($pass)* )? }
    };
    // Internal suffix recursion
    ( @suffix $cb:tt, ( [ $($acc:tt)+ ] ), [ ] $($tail:tt)* $(, $($pass:tt)* )? ) => {
        jnorm_type_then!{@suffix $cb, ( [ [ $($acc)+ ] ] ), $($tail)* $(, $($pass)* )? }
    };
    ( @suffix $cb:tt, ( [ $($acc:tt)+ ] ), as $as_ty:ty $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ $($acc)+ ] ) as rust( $as_ty ) ) $(, $($pass)* )? }
    };
    ( @suffix $cb:tt, ( [ $($acc:tt)+ ] ), $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( [ $($acc)+ ] ) as rust( rust_type_for_java_array_default!([ $($acc)+ ]) ) ) $(, $($pass)* )? }
    };

    // ----- Simple Rust ref: &Path -----
    ( $cb:tt, ( & $rust:ty ) $(, $($pass:tt)* )? ) => { $cb!{ ( rust( $rust ) ) $(, $($pass)* )? } };

    // ----- Java object type -----
    ( $cb:tt, ( $first:ident . $($rest:tt)+ as $as_ty:ty ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( $first . $($rest)+ ) as rust( $as_ty ) ) $(, $($pass)* )? }
    };
    ( $cb:tt, ( $first:ident . $($rest:tt)+ ) $(, $($pass:tt)* )? ) => {
        $cb!{ ( obj( $first . $($rest)+ ) as rust( $crate::objects::JObject ) ) $(, $($pass)* )? }
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

    // ---------- With trailing comma ----------
    // &[[...]],
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident : & [ [ $($inner:tt)+ ] ] , $($rest:tt)* ) => {
        jnorm_type_then!{ jnorm_args_then_push, ( & [ [ $($inner)+ ] ] ), $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n, $($rest)* }
    };
    // &[...],
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident : & [ $($inner:tt)+ ] , $($rest:tt)* ) => {
        jnorm_type_then!{ jnorm_args_then_push, ( & [ $($inner)+ ] ), $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n, $($rest)* }
    };
    // &Path,
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident : & $rust:ty , $($rest:tt)* ) => {
        jnorm_type_then!{ jnorm_args_then_push, ( & $rust ), $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n, $($rest)* }
    };
    // java.name with as,
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : $first:ident $( . $seg:ident )+ $( :: $inner:ident )* as $as_ty:ty , $($rest:tt)* ) => {
        jnorm_type_then!{
            jnorm_args_then_push,
            ( $first $( . $seg )+ $( :: $inner )* as $as_ty ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n, $($rest)*
        }
    };
    // java.name,
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : $first:ident $( . $seg:ident )+ $( :: $inner:ident )* , $($rest:tt)* ) => {
        jnorm_type_then!{
            jnorm_args_then_push,
            ( $first $( . $seg )+ $( :: $inner )* ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n, $($rest)*
        }
    };
    // .Default::Inner with as,
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : . $outer:ident $( :: $inner:ident )* as $as_ty:ty , $($rest:tt)* ) => {
        jnorm_type_then!{ jnorm_args_then_push, ( . $outer $( :: $inner )* as $as_ty ), $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n, $($rest)* }
    };
    // .Default::Inner,
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : . $outer:ident $( :: $inner:ident )* , $($rest:tt)* ) => {
        jnorm_type_then!{ jnorm_args_then_push, ( . $outer $( :: $inner )* ), $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n, $($rest)* }
    };
    // primitive,
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : $p:ident , $($rest:tt)* ) => {
        jnorm_type_then!{ jnorm_args_then_push, ( $p ), $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n, $($rest)* }
    };

    // ---------- Last element (no trailing comma) ----------
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident : & [ [ $($inner:tt)+ ] ] ) => {
        jnorm_type_then!{ jnorm_args_then_push, ( & [ [ $($inner)+ ] ] ), $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n }
    };
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident : & [ $($inner:tt)+ ] ) => {
        jnorm_type_then!{ jnorm_args_then_push, ( & [ $($inner)+ ] ), $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n }
    };
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident : & $rust:ty ) => {
        jnorm_type_then!{ jnorm_args_then_push, ( & $rust ), $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n }
    };
    // java.name with as
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : $first:ident $( . $seg:ident )+ $( :: $inner:ident )* as $as_ty:ty ) => {
        jnorm_type_then!{
            jnorm_args_then_push,
            ( $first $( . $seg )+ $( :: $inner )* as $as_ty ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n
        }
    };
    // java.name
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : $first:ident $( . $seg:ident )+ $( :: $inner:ident )* ) => {
        jnorm_type_then!{
            jnorm_args_then_push,
            ( $first $( . $seg )+ $( :: $inner )* ),
            $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n
        }
    };
    // .Default::Inner with as
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : . $outer:ident $( :: $inner:ident )* as $as_ty:ty ) => {
        jnorm_type_then!{ jnorm_args_then_push, ( . $outer $( :: $inner )* as $as_ty ), $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n }
    };
    // .Default::Inner
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ),
      $n:ident : . $outer:ident $( :: $inner:ident )* ) => {
        jnorm_type_then!{ jnorm_args_then_push, ( . $outer $( :: $inner )* ), $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n }
    };
    // primitive
    ( @ $mode:ident $cb:tt, ( $($acc:tt)* ), ( $($pass:tt)* ), $n:ident : $p:ident ) => {
        jnorm_type_then!{ jnorm_args_then_push, ( $p ), $cb, @ $mode, ( $($acc)* ), ( $($pass)* ), $n }
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
    ( obj ( $( $j:tt )+ ) as rust ( $as_ty:ty ) ) => {
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
        String::from(concat!(
            "(",
            $( _desc_piece!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ) ),*,
            ")",
            _desc_piece!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? )
        ))
    };
}
macro_rules! build_sig_format {
    ( ( $( $n:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* )
      -> ( $rk:ident ( $($rt:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? )
    ) => {
        {
            let mut buf = String::new();
            buf.push_str("(");
            $( buf.push_str(_desc_piece!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? )); )*
            buf.push_str(")");
            buf.push_str(_desc_piece!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? ));
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

// Final emitter: we now have normalized args and ret; generate the whole fn
macro_rules! _emit_method_call_fn {
    (
        $this:path,
        $rname:ident,
        ( $( $an:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* $(,)? ),
        ( $rk:ident ( $($rt:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? )
    ) => {
        paste! {
            fn [<_ $rname _call>](
                env: &mut Env,
                this: &$this,
                method_id: JMethodID,
                $( $an: _param_ty_from_norm!( $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ) ),*
            ) -> Result< _ret_ty_from_norm!( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? ) > {
                let jni_args = build_jni_args_array!( ( $( $an : $ak( $($at)* ) $( as rust ( $($aas)+ ) )? ),* ) );

                let _ = &this;
                /*
                jni_call_check_ex!(
                    this, v1_1,
                    jnorm_ret_then!(_call_api_for_ret, ( $rk( $($rt)* ) $( as rust ( $($ret_as)+ ) )? )),
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

// Glue: normalize ret after args are normalized
macro_rules! _emit_call_fn {
    // Accept grouped normalized return
    (
        ( $($norm_ret:tt)+ ),
        $this:path,
        $rname:ident,
        ( $( $nargs:tt )* )
    ) => {
        _emit_method_call_fn!( $this, $rname, ( $( $nargs )* ), ( $($norm_ret)+ ) );
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

mod sys {
    pub use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort, jvalue};
}
mod refs {
    pub(crate) use super::Reference;
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

mod objects {
    use std::{borrow::Cow, ffi::CStr};

    pub struct JObject;
    impl crate::refs::Reference for JObject {
        fn class_name() -> Cow<'static, CStr> {
            Cow::Borrowed(c"java/lang/Object")
        }
        fn as_raw(&self) -> jni::sys::jobject {
            core::ptr::null_mut()
        }
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

fn main() {}
