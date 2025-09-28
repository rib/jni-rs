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
    ( & $rust:path ) => {
        format!("L{};", < $rust as Reference >::class_name())
    };

    // array of Rust refs: &[&JString], [[&JString]], etc.
    ( & [ $($inner:tt)+ ] ) => {
        format!("[{}]", rust_jdesc!( $($inner)+ ))
    };

    // enforce that arrays only contain Rust refs (reject e.g. &[java.lang.String])
    ( [ $($any:tt)+ ] ) => {
        compile_error!("Rust arrays must be references to Rust Reference types, e.g., &[&JString]")
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

    // & [ ... ]
    ( ( $n:ident : & [ $($inner:tt)+ ] , $($rest:tt)* ) ( $($acc:tt)* ) ) => {
        jnorm_args_munch!( ( $($rest)* ) ( $($acc)* $n: rust(rust_array_wrapper_ty(&[ $($inner)+ ])) as rust_array_wrapper_ty(&[ $($inner)+ ]), ) )
    };
    ( ( $n:ident : & [ $($inner:tt)+ ] ) ( $($acc:tt)* ) ) => {
        ( $($acc)* $n: rust(rust_array_wrapper_ty(&[ $($inner)+ ])) as rust_array_wrapper_ty(&[ $($inner)+ ]) )
    };

    // & Path
    ( ( $n:ident : & $rust:path , $($rest:tt)* ) ( $($acc:tt)* ) ) => {
        jnorm_args_munch!( ( $($rest)* ) ( $($acc)* $n: rust($rust) as & $rust , ) )
    };
    ( ( $n:ident : & $rust:path ) ( $($acc:tt)* ) ) => {
        ( $($acc)* $n: rust($rust) as & $rust )
    };

    // package name with 'as'
    ( ( $n:ident : $first:ident . $($restname:tt)+ as $as_ty:path , $($rest:tt)* ) ( $($acc:tt)* ) ) => {
        jnorm_args_munch!( ( $($rest)* ) ( $($acc)* $n: obj( $first . $($restname)+ ) as $as_ty , ) )
    };
    ( ( $n:ident : $first:ident . $($restname:tt)+ as $as_ty:path ) ( $($acc:tt)* ) ) => {
        ( $($acc)* $n: obj( $first . $($restname)+ ) as $as_ty )
    };

    // package name, default as JObject
    ( ( $n:ident : $first:ident . $($restname:tt)+ , $($rest:tt)* ) ( $($acc:tt)* ) ) => {
        jnorm_args_munch!( ( $($rest)* ) ( $($acc)* $n: obj( $first . $($restname)+ ) as JObject , ) )
    };
    ( ( $n:ident : $first:ident . $($restname:tt)+ ) ( $($acc:tt)* ) ) => {
        ( $($acc)* $n: obj( $first . $($restname)+ ) as JObject )
    };

    // default package with 'as'
    ( ( $n:ident : . $outer:ident $( :: $inner:ident )* as $as_ty:path , $($rest:tt)* ) ( $($acc:tt)* ) ) => {
        jnorm_args_munch!( ( $($rest)* ) ( $($acc)* $n: obj( . $outer $( :: $inner )* ) as $as_ty , ) )
    };
    ( ( $n:ident : . $outer:ident $( :: $inner:ident )* as $as_ty:path ) ( $($acc:tt)* ) ) => {
        ( $($acc)* $n: obj( . $outer $( :: $inner )* ) as $as_ty )
    };

    // default package, default as JObject
    ( ( $n:ident : . $outer:ident $( :: $inner:ident )* , $($rest:tt)* ) ( $($acc:tt)* ) ) => {
        jnorm_args_munch!( ( $($rest)* ) ( $($acc)* $n: obj( . $outer $( :: $inner )* ) as JObject , ) )
    };
    ( ( $n:ident : . $outer:ident $( :: $inner:ident )* ) ( $($acc:tt)* ) ) => {
        ( $($acc)* $n: obj( . $outer $( :: $inner )* ) as JObject )
    };

    // primitives (last to avoid eating 'java' as a primitive)
    ( ( $n:ident : $p:ident , $($rest:tt)* ) ( $($acc:tt)* ) ) => {
        jnorm_args_munch!( ( $($rest)* ) ( $($acc)* $n: prim( jprim_canon!($p) ) as jprim_canon!($p) , ) )
    };
    ( ( $n:ident : $p:ident ) ( $($acc:tt)* ) ) => {
        ( $($acc)* $n: prim( jprim_canon!($p) ) as jprim_canon!($p) )
    };
}

macro_rules! jnorm_args {
    ( $( $tokens:tt )* ) => {
        jnorm_args_munch!( ( $( $tokens )* ) () )
    };
}

// Return normalization → keep same “as …” rule so downstream is uniform
macro_rules! jnorm_ret {
    ( $p:ident ) => { prim( jprim_canon!($p) ) as jprim_canon!($p) };
    ( $first:ident . $($rest:tt)+ as $as_ty:path ) => { obj( $first . $($rest)+ ) as $as_ty };
    ( $first:ident . $($rest:tt)+ ) => { obj( $first . $($rest)+ ) as JObject };
    ( . $outer:ident $( :: $inner:ident )* as $as_ty:path ) => { obj( . $outer $( :: $inner )* ) as $as_ty };
    ( . $outer:ident $( :: $inner:ident )* ) => { obj( . $outer $( :: $inner )* ) as JObject };
    ( & $rust:path ) => { rust(& $rust) as & $rust };
    ( & [ $($inner:tt)+ ] ) => { rust(&[ $($inner)+ ]) as &[ $($inner)+ ] };
}

// ---------- helpers: presence of rust(...), building pieces ----------
macro_rules! _is_rust {
    ( prim ( $($t:tt)* ) as $($as:tt)+ ) => {
        0
    };
    ( obj  ( $($t:tt)* ) as $($as:tt)+ ) => {
        0
    };
    ( rust ( $($t:tt)* ) as $($as:tt)+ ) => {
        1
    };
}
macro_rules! any_rust {
    ( ( $( $n:ident : $k:ident ( $($t:tt)* ) as $as_ty:path ),* ), ( $rk:ident ( $($rt:tt)* ) as $ret_ty:path ) ) => {{
        0 $( | _is_rust!($k($($t)*) as $as_ty) )* | _is_rust!($rk($($rt)*) as $ret_ty)
    }};
}

macro_rules! _arg_desc_lit     { ( prim ( $p:ident ) as $as:path ) => { jdesc!($p) };
                                 ( obj  ( $($j:tt)+ ) as $as:path ) => { jdesc!($($j)+) }; }
macro_rules! _arg_desc_fmt_piece{ ( prim ( $p:ident ) as $as:path ) => { jdesc!($p) };
                                 ( obj  ( $($j:tt)+ ) as $as:path ) => { jdesc!($($j)+) };
                                 ( rust ( & $($r:tt)+ ) as $as:path ) => { "{}" }; }
macro_rules! _arg_desc_fmt_vals { ( prim ( $p:ident ) as $as:path ) => {};
                                 ( obj  ( $($j:tt)+ ) as $as:path ) => {};
                                 ( rust ( & $($r:tt)+ ) as $as:path ) => { , rust_jdesc!(& $($r)+) }; }

macro_rules! ret_desc_lit       { ( prim ( $p:ident ) as $as:path ) => { jdesc!($p) };
                                 ( obj  ( $($j:tt)+ ) as $as:path ) => { jdesc!($($j)+) }; }
macro_rules! ret_desc_fmt_piece { ( prim ( $p:ident ) as $as:path ) => { jdesc!($p) };
                                 ( obj  ( $($j:tt)+ ) as $as:path ) => { jdesc!($($j)+) };
                                 ( rust ( & $($r:tt)+ ) as $as:path ) => { "{}" }; }
macro_rules! ret_desc_fmt_val   { ( prim ( $p:ident ) as $as:path ) => {};
                                 ( obj  ( $($j:tt)+ ) as $as:path ) => {};
                                 ( rust ( & $($r:tt)+ ) as $as:path ) => { , rust_jdesc!(& $($r)+) }; }

// Parameter & return Rust types from normalized forms
macro_rules! _param_ty_from_norm {
    ( prim ( $(_p:tt)+ ) as $as_ty:path ) => {
        $as_ty
    };
    ( obj  ( $(_j:tt)+ ) as $as_ty:path ) => {
        &$as_ty
    };
    ( rust ( $as_r:path )   as $as_use:path ) => {
        $as_use
    };
}
macro_rules! _ret_ty_from_norm {
    ( prim ( void ) as $as_ty:path ) => {
        ()
    };
    ( prim ( $(_p:tt)+ ) as $as_ty:path ) => {
        $as_ty
    };
    ( obj  ( $(_j:tt)+ ) as $as_ty:path ) => {
        $as_ty
    };
    ( rust ( $as_r:path )   as $as_use:path ) => {
        $as_use
    };
}

// Build signature (literal vs format)
macro_rules! build_sig_literal {
    ( ( $( $n:ident : $k:ident ( $($t:tt)* ) as $as_ty:path ),* ) -> ( $rk:ident ( $($rt:tt)* ) as $ret_ty:path ) ) => {
        concat!("(", $( _arg_desc_lit!($k($($t)*) as $as_ty) ),* , ")", ret_desc_lit!($rk($($rt)*) as $ret_ty))
    };
}
macro_rules! build_sig_format {
    ( ( $( $n:ident : $k:ident ( $($t:tt)* ) as $as_ty:path ),* ) -> ( $rk:ident ( $($rt:tt)* ) as $ret_ty:path ) ) => {
        format!(
            concat!("(", $( _arg_desc_fmt_piece!($k($($t)*) as $as_ty) ),* , ")", ret_desc_fmt_piece!($rk($($rt)*) as $ret_ty))
            $( _arg_desc_fmt_vals!($k($($t)*) as $as_ty) )*
            ret_desc_fmt_val!($rk($($rt)*) as $ret_ty)
        )
    };
}

// ----- Convert an argument expression to a `jvalue` (by normalized kind) -----

// How to get a `jobject` handle from a reference-y Rust wrapper.
// Edit this if your wrappers use a different API.
macro_rules! as_jobject_handle {
    // Common jni crate pattern: get raw handle from any Reference type
    ( $as_ty:path, $expr:expr ) => {{
        <$as_ty as Reference>::as_raw($expr)
    }};
}

// A single normalized arg → jni::sys::jvalue
macro_rules! _jvalue_of_norm_arg {
    ( $name:ident : prim ( jboolean ) as $as_ty:path ) => {
        jni::sys::jvalue {
            z: ($name as jni::sys::jboolean),
        }
    };
    ( $name:ident : prim ( jbyte    ) as $as_ty:path ) => {
        jni::sys::jvalue {
            b: ($name as jni::sys::jbyte),
        }
    };
    ( $name:ident : prim ( jchar    ) as $as_ty:path ) => {
        jni::sys::jvalue {
            c: ($name as jni::sys::jchar),
        }
    };
    ( $name:ident : prim ( jshort   ) as $as_ty:path ) => {
        jni::sys::jvalue {
            s: ($name as jni::sys::jshort),
        }
    };
    ( $name:ident : prim ( jint     ) as $as_ty:path ) => {
        jni::sys::jvalue {
            i: ($name as jni::sys::jint),
        }
    };
    ( $name:ident : prim ( jlong    ) as $as_ty:path ) => {
        jni::sys::jvalue {
            j: ($name as jni::sys::jlong),
        }
    };
    ( $name:ident : prim ( jfloat   ) as $as_ty:path ) => {
        jni::sys::jvalue {
            f: ($name as jni::sys::jfloat),
        }
    };
    ( $name:ident : prim ( jdouble  ) as $as_ty:path ) => {
        jni::sys::jvalue {
            d: ($name as jni::sys::jdouble),
        }
    };

    // Objects (Java or Rust) — both end up as 'l'
    ( $name:ident : obj  ( $($j:tt)+ ) as $as_ty:path ) => {
        jni::sys::jvalue {
            l: as_jobject_handle!($as_ty, $name),
        }
    };
    ( $name:ident : rust ( $as_r:path ) as $as_use:path ) => {
        jni::sys::jvalue {
            l: as_jobject_handle!($as_use, $name),
        }
    };
}

// Build `[jvalue; N]` from a normalized arg list
macro_rules! build_jni_args_array {
    ( ( $( $an:ident : $ak:ident ( $($at:tt)* ) as $aas:path ),* $(,)? ) ) => {{
        [ $( _jvalue_of_norm_arg!($an : $ak($($at)*) as $aas) ),* ]
    }};
}

// ----- Select the correct CallXxxMethodA for the return type -----
macro_rules! _call_api_for_ret {
    ( prim ( void     ) as $rty:path ) => {
        CallVoidMethodA
    };
    ( prim ( jboolean ) as $rty:path ) => {
        CallBooleanMethodA
    };
    ( prim ( jbyte    ) as $rty:path ) => {
        CallByteMethodA
    };
    ( prim ( jchar    ) as $rty:path ) => {
        CallCharMethodA
    };
    ( prim ( jshort   ) as $rty:path ) => {
        CallShortMethodA
    };
    ( prim ( jint     ) as $rty:path ) => {
        CallIntMethodA
    };
    ( prim ( jlong    ) as $rty:path ) => {
        CallLongMethodA
    };
    ( prim ( jfloat   ) as $rty:path ) => {
        CallFloatMethodA
    };
    ( prim ( jdouble  ) as $rty:path ) => {
        CallDoubleMethodA
    };

    // Anything reference-like (Java object or "rust(&T)") returns an object
    ( obj  ( $($rt:tt)+ ) as $rty:path ) => {
        CallObjectMethodA
    };
    ( rust ( $r:path )       as $rty:path ) => {
        CallObjectMethodA
    };
}

// ----- LOOKUP FN -----
macro_rules! jgen_method_lookup_fn {
    (
        $this:ident,
        $jname:ident,
        $rname:ident,
        $args:tt,
        $ret:tt
    ) => {
        paste! {
            fn [<_ $rname _lookup>](env: &mut Env) -> Result<JMethodID> {
                let class: &JClass = $this::lookup_class()?;
                let sig = if any_rust!( ( unwrap_parens!($args) ), ( $ret ) ) == 0 {
                    build_sig_literal!( ( unwrap_parens!($args) ) -> ( $ret ) )
                } else {
                    build_sig_format!( ( unwrap_parens!($args) ) -> ( $ret ) )
                };
                env.get_method_id(class, stringify!($jname), &sig)
            }
        }
    };
}

// ----- CALL FN (build jvalue[] and pick the API) -----
macro_rules! jgen_method_call_fn {
    (
        $this:ident,
        $rname:ident,
        $args:tt,
        $ret:tt
    ) => {
        paste! {
            fn [<_ $rname _call>](
                env: Env,
                this: &$this,
                method_id: JMethodID,
                param_list_from_norm!( ( unwrap_parens!($args) ) )
            ) -> Result<_ret_ty_from_norm!( $ret )> {
                // Build the jvalue array
                let jni_args = build_jni_args_array!( ( unwrap_parens!($args) ) );

                // Choose the correct CallXxxMethodA variant based on return
                let _ = &this; // silence unused for some linters
                // Your codebase likely wraps raw calls with a helper macro; we mirror your earlier example:
                jni_call_check_ex!(
                    this, v1_1,
                    _call_api_for_ret!( $ret ),
                    obj, // (your helper macro may use this token for dispatch; adjust if needed)
                    method_id,
                    &jni_args
                )
            }
        }
    };
}

// Expand a normalized args list into a typed parameter list
macro_rules! param_list_from_norm {
    ( ( $( $an:ident : $ak:ident ( $($at:tt)* ) as $aas:path ),* $(,)? ) ) => {
        $( $an: _param_ty_from_norm!($ak($($at)*) as $aas) ),*
    };
}

// Helper to unwrap one layer of parentheses around a token tree
macro_rules! unwrap_parens {
    ( ( $($inner:tt)* ) ) => { $($inner)* };
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
        sig: ( $( $an:ident : $($aty:tt)+ ),* $(,)? ) -> $($rty:tt)+
    ) => {

        jgen_method_lookup_fn!(
            $this,
            $jname,
            $rname,
            jnorm_args!( $( $an : $($aty)+ ),* ),
            jnorm_ret!( $($rty)+ )
        );
        jgen_method_call_fn!(
            $this,
            $rname,
            jnorm_args!( $( $an : $($aty)+ ),* ),
            jnorm_ret!( $($rty)+ )
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
    javaFunction as rust_function,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> void
);
