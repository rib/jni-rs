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
    (boolean) => {
        "Z"
    };
    (jbyte) => {
        "B"
    };
    (byte) => {
        "B"
    };
    (jchar) => {
        "C"
    };
    (char) => {
        "C"
    };
    (jshort) => {
        "S"
    };
    (short) => {
        "S"
    };
    (jint) => {
        "I"
    };
    (int) => {
        "I"
    };
    (jlong) => {
        "J"
    };
    (long) => {
        "J"
    };
    (jfloat) => {
        "F"
    };
    (float) => {
        "F"
    };
    (jdouble) => {
        "D"
    };
    (double) => {
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

// ---------- Type → internal descriptor (literals only: prim + Java) ----------
macro_rules! jdesc {
    ( [ $($inner:tt)+ ] ) => { concat!("[", jdesc!($($inner)+)) };

    // Java (named)
    ( $first:ident . $($rest:tt)+ ) => {
        concat!("L", jname_internal!($first . $($rest)+), ";")
    };
    // Java (default package)
    ( . $outer:ident $( :: $inner:ident )* ) => {
        concat!("L", jname_internal!(. $outer $( :: $inner )*), ";")
    };

    // primitives
    (void)=>{ jprim!(void) };
    (jboolean)=>{ jprim!(jboolean) }; (boolean)=>{ jprim!(boolean) };
    (jbyte)=>{ jprim!(jbyte) };       (byte)=>{ jprim!(byte) };
    (jchar)=>{ jprim!(jchar) };       (char)=>{ jprim!(char) };
    (jshort)=>{ jprim!(jshort) };     (short)=>{ jprim!(short) };
    (jint)=>{ jprim!(jint) };         (int)=>{ jprim!(int) };
    (jlong)=>{ jprim!(jlong) };       (long)=>{ jprim!(long) };
    (jfloat)=>{ jprim!(jfloat) };     (float)=>{ jprim!(float) };
    (jdouble)=>{ jprim!(jdouble) };   (double)=>{ jprim!(double) };
}

// ---------- Rust-side descriptor (runtime, uses Reference::class_name) ----------
macro_rules! rust_jdesc {
    // single reference: &JString
    ( $rust:ty ) => {
        format!("L{};", <$rust as $crate::refs::Reference>::class_name())
    };
}

// ---------- NORMALIZATION ----------

macro_rules! rust_array_wrapper_ty {
    // Strip leading reference
    ( & [ $($inner:tt)+ ] ) => { rust_array_wrapper_ty!( [ $($inner)+ ] ) };

    // ----- multi-dimensional primitive arrays: reject -----
    ( [ [ jboolean ] ] ) => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[jboolean]])"); };
    ( [ [ boolean ] ] )  => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[boolean]])"); };
    ( [ [ jbyte ] ] )    => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[jbyte]])"); };
    ( [ [ byte ] ] )     => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[byte]])"); };
    ( [ [ jchar ] ] )    => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[jchar]])"); };
    ( [ [ char ] ] )     => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[char]])"); };
    ( [ [ jshort ] ] )   => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[jshort]])"); };
    ( [ [ short ] ] )    => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[short]])"); };
    ( [ [ jint ] ] )     => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[jint]])"); };
    ( [ [ int ] ] )      => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[int]])"); };
    ( [ [ jlong ] ] )    => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[jlong]])"); };
    ( [ [ long ] ] )     => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[long]])"); };
    ( [ [ jfloat ] ] )   => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[jfloat]])"); };
    ( [ [ float ] ] )    => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[float]])"); };
    ( [ [ jdouble ] ] )  => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[jdouble]])"); };
    ( [ [ double ] ] )   => { compile_error!("Multi-dimensional primitive arrays are not supported (found [[double]])"); };

    // ----- single-dimension primitive arrays -----
    ( [ jboolean ] ) => { JPrimitiveArray<jboolean> };
    ( [ boolean ] )  => { JPrimitiveArray<jboolean> };
    ( [ jbyte ] )    => { JPrimitiveArray<jbyte> };
    ( [ byte ] )     => { JPrimitiveArray<jbyte> };
    ( [ jchar ] )    => { JPrimitiveArray<jchar> };
    ( [ char ] )     => { JPrimitiveArray<jchar> };
    ( [ jshort ] )   => { JPrimitiveArray<jshort> };
    ( [ short ] )    => { JPrimitiveArray<jshort> };
    ( [ jint ] )     => { JPrimitiveArray<jint> };
    ( [ int ] )      => { JPrimitiveArray<jint> };
    ( [ jlong ] )    => { JPrimitiveArray<jlong> };
    ( [ long ] )     => { JPrimitiveArray<jlong> };
    ( [ jfloat ] )   => { JPrimitiveArray<jfloat> };
    ( [ float ] )    => { JPrimitiveArray<jfloat> };
    ( [ jdouble ] )  => { JPrimitiveArray<jdouble> };
    ( [ double ] )   => { JPrimitiveArray<jdouble> };

    // ----- object arrays (arbitrary depth) -----
    ( [ [ $($inner:tt)+ ] ] ) => {
        JObjectArray< rust_array_wrapper_ty!( [ $($inner)+ ] ) >
    };

    // Optional: allow element references (e.g., &[&JString]) by stripping '&'
    ( [ & $ty:path ] ) => { JObjectArray<$ty> };

    // Base case: single-dimension object array
    ( [ $ty:path ] ) => { JObjectArray<$ty> };
}

/*  Inputs:
      name: jchar
      name: java.lang.String
      name: java.lang.String as JString
      name: &JString
      name: &[&JString]
    Outputs:
      name: prim(jchar) as jchar
      name: obj(java.lang.String) as JObject
      name: obj(java.lang.String) as JString
      name: rust(&JString) as &JString
      name: rust(&[&JString]) as &[&JString]
*/
macro_rules! jnorm_args_munch {
    ( ( ) ( $($acc:tt)* ) ) => { ( $($acc)* ) };

    // package name with 'as'
    ( ( $n:ident : $first:ident . $($restname:tt)+ as $as_ty:ty , $($rest:tt)* ) ( $($acc:tt)* ) ) => {
        jnorm_args_munch!( ( $($rest)* ) ( $($acc)* $n: obj( $first . $($restname)+ ) as rust( $as_ty ) , ) )
    };
    ( ( $n:ident : $first:ident . $($restname:tt)+ as $as_ty:ty ) ( $($acc:tt)* ) ) => {
        ( $($acc)* $n: obj( $first . $($restname)+ ) as rust( $as_ty ) )
    };

    // package name, default as JObject
    ( ( $n:ident : $first:ident . $($restname:tt)+ , $($rest:tt)* ) ( $($acc:tt)* ) ) => {
        jnorm_args_munch!( ( $($rest)* ) ( $($acc)* $n: obj( $first . $($restname)+ ) as rust( JObject ) , ) )
    };
    ( ( $n:ident : $first:ident . $($restname:tt)+ ) ( $($acc:tt)* ) ) => {
        ( $($acc)* $n: obj( $first . $($restname)+ ) as rust( JObject ) )
    };

    // default package with 'as'
    ( ( $n:ident : . $outer:ident $( :: $inner:ident )* as $as_ty:ty , $($rest:tt)* ) ( $($acc:tt)* ) ) => {
        jnorm_args_munch!( ( $($rest)* ) ( $($acc)* $n: obj( . $outer $( :: $inner )* ) as rust( $as_ty ) , ) )
    };
    ( ( $n:ident : . $outer:ident $( :: $inner:ident )* as $as_ty:ty ) ( $($acc:tt)* ) ) => {
        ( $($acc)* $n: obj( . $outer $( :: $inner )* ) as rust( $as_ty ) )
    };

    // default package, default as JObject
    ( ( $n:ident : . $outer:ident $( :: $inner:ident )* , $($rest:tt)* ) ( $($acc:tt)* ) ) => {
        jnorm_args_munch!( ( $($rest)* ) ( $($acc)* $n: obj( . $outer $( :: $inner )* ) as rust( JObject ) , ) )
    };
    ( ( $n:ident : . $outer:ident $( :: $inner:ident )* ) ( $($acc:tt)* ) ) => {
        ( $($acc)* $n: obj( . $outer $( :: $inner )* ) as rust( JObject ) )
    };

    // & [ ... ]
    ( ( $n:ident : & [ $($inner:tt)+ ] , $($rest:tt)* ) ( $($acc:tt)* ) ) => {
        jnorm_args_munch!( ( $($rest)* ) ( $($acc)* $n: rust(rust_array_wrapper_ty(&[ $($inner)+ ])), ) )
    };
    ( ( $n:ident : & [ $($inner:tt)+ ] ) ( $($acc:tt)* ) ) => {
        ( $($acc)* $n: rust(rust_array_wrapper_ty(&[ $($inner)+ ])) )
    };

    // & Path
    ( ( $n:ident : & $rust:path , $($rest:tt)* ) ( $($acc:tt)* ) ) => {
        jnorm_args_munch!( ( $($rest)* ) ( $($acc)* $n: rust($rust) , ) )
    };
    ( ( $n:ident : & $rust:path ) ( $($acc:tt)* ) ) => {
        ( $($acc)* $n: rust($rust) )
    };

    // primitives (last to avoid eating 'java' as a primitive)
    ( ( $n:ident : $p:ident , $($rest:tt)* ) ( $($acc:tt)* ) ) => {
        jnorm_args_munch!( ( $($rest)* ) ( $($acc)* $n: prim( jprim_canon!($p) ) , ) )
    };
    ( ( $n:ident : $p:ident ) ( $($acc:tt)* ) ) => {
        ( $($acc)* $n: prim( jprim_canon!($p) ) )
    };
}

macro_rules! jnorm_args {
    ( $( $tokens:tt )* ) => {
        jnorm_args_munch!( ( $( $tokens )* ) () )
    };
}

// Return normalization → keep same “as …” rule so downstream is uniform
macro_rules! jnorm_ret {
    ( $first:ident . $($rest:tt)+ as $as_ty:ty ) => { obj( $first . $($rest)+ ) as rust( $as_ty ) };
    ( $first:ident . $($rest:tt)+ ) => { obj( $first . $($rest)+ ) as rust( JObject ) };
    ( . $outer:ident $( :: $inner:ident )* as $as_ty:ty ) => { obj( . $outer $( :: $inner )* ) as rust( $as_ty ) };
    ( . $outer:ident $( :: $inner:ident )* ) => { obj( . $outer $( :: $inner )* ) as rust( JObject ) };
    ( & $rust:ty ) => { rust($rust) };
    ( & [ $($inner:tt)+ ] ) => { rust(rust_array_wrapper_ty(&[ $($inner)+ ])) };
    ( $p:ident ) => { prim( jprim_canon!($p) ) };
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

macro_rules! _arg_desc_lit     { ( prim ( $p:ident ) ) => { jdesc!($p) };
                                 ( obj  ( $($j:tt)+ ) as rust ( $as:ty ) ) => { jdesc!($($j)+) }; }
macro_rules! _arg_desc_fmt_piece{ ( prim ( $p:ident ) ) => { jdesc!($p) };
                                 ( obj  ( $($j:tt)+ ) as rust ( $as:ty ) ) => { jdesc!($($j)+) };
                                 ( rust ( $($r:tt)+ ) ) => { "{}" }; }
macro_rules! _arg_desc_fmt_vals { ( prim ( $p:ident ) ) => {};
                                 ( obj  ( $($j:tt)+ ) as rust ( $as:ty ) ) => {};
                                 ( rust ( $($r:tt)+ ) ) => { , rust_jdesc!($($r)+) }; }

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

// Build signature (literal vs format)

macro_rules! build_sig_literal {
    ( ( $( $n:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* )
      -> ( $rk:ident ( $($rt:tt)* ) $( as rust ( $($ret_as:tt)+ ) )? )
    ) => {
        concat!(
            "(",
            $( _arg_desc_lit!( $ak($($at)*) $( as rust ( $($aas)+ ) )? ) ),*,
            ")",
            ret_desc_lit!( $rk($($rt)*) $( as rust ( $($ret_as)+ ) )? )
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
                $( _arg_desc_fmt_piece!( $ak($($at)*) $( as rust ( $($aas)+ ) )? ) ),*,
                ")",
                ret_desc_fmt_piece!( $rk($($rt)*) $( as rust ( $($ret_as)+ ) )? )
            )
            $( _arg_desc_fmt_vals!( $ak($($at)*) $( as rust ( $($aas)+ ) )? ) )*
            ret_desc_fmt_val!( $rk($($rt)*) $( as rust ( $($ret_as)+ ) )? )
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
macro_rules! param_list_from_norm {
    ( ( $( $an:ident : $ak:ident ( $($at:tt)* ) $( as rust ( $($aas:tt)+ ) )? ),* $(,)? ) ) => {
        $( $an: _param_ty_from_norm!( $ak($($at)*) $( as rust ( $($aas)+ ) )? ) ),*
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

                // Expand normalization at the call site of the helpers.
                let sig = if any_rust!(
                    ( jnorm_args!( $($args)* ) ),
                    ( jnorm_ret!( $($ret)+ ) )
                ) == 0 {
                    build_sig_literal!(
                        ( jnorm_args!( $($args)* ) )
                        -> ( jnorm_ret!( $($ret)+ ) )
                    )
                } else {
                    build_sig_format!(
                        ( jnorm_args!( $($args)* ) )
                        -> ( jnorm_ret!( $($ret)+ ) )
                    )
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
                param_list_from_norm!( ( jnorm_args!( $($args)* ) ) )
            ) -> Result<_ret_ty_from_norm!( jnorm_ret!( $($ret)+ ) )> {
                // Build the jvalue array
                let jni_args = build_jni_args_array!( ( jnorm_args!( $($args)* ) ) );

                // Choose the correct CallXxxMethodA variant based on return
                let _ = &this; // silence unused for some linters
                // Your codebase likely wraps raw calls with a helper macro; we mirror your earlier example:
                jni_call_check_ex!(
                    this, v1_1,
                    _call_api_for_ret!( jnorm_ret!( $($ret)+ ) ),
                    obj, // (your helper macro may use this token for dispatch; adjust if needed)
                    method_id,
                    &jni_args
                )
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
use std::{borrow::Cow, fmt};

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
    fn class_name() -> Cow<'static, JNIStr>;
    fn as_raw(&self) -> jni::sys::jobject;
}

impl Reference for JString {
    fn class_name() -> Cow<'static, JNIStr> {
        Cow::Owned(JNIStr)
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

// 1) Primitives (with and without 'j')
const _: &str = jdesc!(jint); // "I"
const _: &str = jdesc!(int); // "I"
const _: &str = jdesc!(void); // "V"

// 2) Objects and arrays (internal names)
const _: &str = jdesc!(java.lang.String); // "Ljava/lang/String;"
const _: &str = jdesc!([java.lang.String]); // "[Ljava/lang/String;"
const _: &str = jdesc!([[java.lang.String]]); // "[[Ljava/lang/String;"
const _: &str = jdesc!(java.util.Map::Entry); // "Ljava/util/Map$Entry;"
const _: &str = jdesc!([java.util.Map::Entry]); // "[Ljava/util/Map$Entry;"
const _: &str = jdesc!(.Hello); // "LHello;"
const _: &str = jdesc!([.Hello]); // "[LHello;"
const _: &str = jdesc!(char); // "C"
const _: &str = jdesc!([char]); // "[C"

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
