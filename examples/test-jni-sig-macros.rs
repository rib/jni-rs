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
macro_rules! jnorm_arg {
    ( $name:ident : $p:ident ) => {
        $name: prim( jprim_canon!($p) ) as jprim_canon!($p)
    };
    ( $name:ident : $first:ident . $($rest:tt)+ as $as_ty:ty ) => {
        $name: obj( $first . $($rest)+ ) as $as_ty
    };
    ( $name:ident : $first:ident . $($rest:tt)+ ) => {
        $name: obj( $first . $($rest)+ ) as JObject
    };
    ( $name:ident : . $outer:ident $( :: $inner:ident )* as $as_ty:ty ) => {
        $name: obj( . $outer $( :: $inner )* ) as $as_ty
    };
    ( $name:ident : . $outer:ident $( :: $inner:ident )* ) => {
        $name: obj( . $outer $( :: $inner )* ) as JObject
    };
    ( $name:ident : & $rust:path ) => {
        $name: rust(& $rust) as & $rust
    };
    ( $name:ident : & [ $($inner:tt)+ ] ) => {
        $name: rust(&[ $($inner)+ ]) as &[ $($inner)+ ]
    };
}
macro_rules! jnorm_args {
    ( $( $name:ident : $($ty:tt)+ ),* $(,)? ) => {
        ( $( jnorm_arg!($name : $($ty)+) ),* )
    };
}

// Return normalization → keep same “as …” rule so downstream is uniform
macro_rules! jnorm_ret {
    ( $p:ident ) => { prim( jprim_canon!($p) ) as jprim_canon!($p) };
    ( $first:ident . $($rest:tt)+ as $as_ty:ty ) => { obj( $first . $($rest)+ ) as $as_ty };
    ( $first:ident . $($rest:tt)+ ) => { obj( $first . $($rest)+ ) as JObject };
    ( . $outer:ident $( :: $inner:ident )* as $as_ty:ty ) => { obj( . $outer $( :: $inner )* ) as $as_ty };
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
    ( ( $( $n:ident : $k:ident ( $($t:tt)* ) as $as_ty:ty ),* ), ( $rk:ident ( $($rt:tt)* ) as $ret_ty:ty ) ) => {{
        0 $( | _is_rust!($k($($t)*) as $as_ty) )* | _is_rust!($rk($($rt)*) as $ret_ty)
    }};
}

macro_rules! _arg_desc_lit     { ( prim ( $p:ident ) as $as:ty ) => { jdesc!($p) };
                                 ( obj  ( $($j:tt)+ ) as $as:ty ) => { jdesc!($($j)+) }; }
macro_rules! _arg_desc_fmt_piece{ ( prim ( $p:ident ) as $as:ty ) => { jdesc!($p) };
                                 ( obj  ( $($j:tt)+ ) as $as:ty ) => { jdesc!($($j)+) };
                                 ( rust ( & $($r:tt)+ ) as $as:ty ) => { "{}" }; }
macro_rules! _arg_desc_fmt_vals { ( prim ( $p:ident ) as $as:ty ) => {};
                                 ( obj  ( $($j:tt)+ ) as $as:ty ) => {};
                                 ( rust ( & $($r:tt)+ ) as $as:ty ) => { , rust_jdesc!(& $($r)+) }; }

macro_rules! ret_desc_lit       { ( prim ( $p:ident ) as $as:ty ) => { jdesc!($p) };
                                 ( obj  ( $($j:tt)+ ) as $as:ty ) => { jdesc!($($j)+) }; }
macro_rules! ret_desc_fmt_piece { ( prim ( $p:ident ) as $as:ty ) => { jdesc!($p) };
                                 ( obj  ( $($j:tt)+ ) as $as:ty ) => { jdesc!($($j)+) };
                                 ( rust ( & $($r:tt)+ ) as $as:ty ) => { "{}" }; }
macro_rules! ret_desc_fmt_val   { ( prim ( $p:ident ) as $as:ty ) => {};
                                 ( obj  ( $($j:tt)+ ) as $as:ty ) => {};
                                 ( rust ( & $($r:tt)+ ) as $as:ty ) => { , rust_jdesc!(& $($r)+) }; }

// Parameter & return Rust types from normalized forms
macro_rules! _param_ty_from_norm {
    ( prim ( $(_p:tt)+ ) as $as_ty:ty ) => {
        $as_ty
    };
    ( obj  ( $(_j:tt)+ ) as $as_ty:ty ) => {
        &$as_ty
    };
    ( rust ( $as_r:ty )   as $as_use:ty ) => {
        $as_use
    };
}
macro_rules! _ret_ty_from_norm {
    ( prim ( $(_p:tt)+ ) as $as_ty:ty ) => {
        $as_ty
    };
    ( obj  ( $(_j:tt)+ ) as $as_ty:ty ) => {
        $as_ty
    };
    ( rust ( $as_r:ty )   as $as_use:ty ) => {
        $as_use
    };
}

// Build signature (literal vs format)
macro_rules! build_sig_literal {
    ( ( $( $n:ident : $k:ident ( $($t:tt)* ) as $as_ty:ty ),* ) -> ( $rk:ident ( $($rt:tt)* ) as $ret_ty:ty ) ) => {
        concat!("(", $( _arg_desc_lit!($k($($t)*) as $as_ty) ),* , ")", ret_desc_lit!($rk($($rt)*) as $ret_ty))
    };
}
macro_rules! build_sig_format {
    ( ( $( $n:ident : $k:ident ( $($t:tt)* ) as $as_ty:ty ),* ) -> ( $rk:ident ( $($rt:tt)* ) as $ret_ty:ty ) ) => {
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
    // Common jni crate pattern: &T -> JObject -> into_raw()
    ( $as_ty:ty $expr:expr ) => {{
        <$as_ty as &crate::refs::Reference>::as_raw($expr)
    }};
}

// A single normalized arg → jni::sys::jvalue
macro_rules! _jvalue_of_norm_arg {
    ( $name:ident : prim ( jboolean ) as $as_ty:ty ) => {
        jni::sys::jvalue {
            z: ($name as jni::sys::jboolean),
        }
    };
    ( $name:ident : prim ( jbyte    ) as $as_ty:ty ) => {
        jni::sys::jvalue {
            b: ($name as jni::sys::jbyte),
        }
    };
    ( $name:ident : prim ( jchar    ) as $as_ty:ty ) => {
        jni::sys::jvalue {
            c: ($name as jni::sys::jchar),
        }
    };
    ( $name:ident : prim ( jshort   ) as $as_ty:ty ) => {
        jni::sys::jvalue {
            s: ($name as jni::sys::jshort),
        }
    };
    ( $name:ident : prim ( jint     ) as $as_ty:ty ) => {
        jni::sys::jvalue {
            i: ($name as jni::sys::jint),
        }
    };
    ( $name:ident : prim ( jlong    ) as $as_ty:ty ) => {
        jni::sys::jvalue {
            j: ($name as jni::sys::jlong),
        }
    };
    ( $name:ident : prim ( jfloat   ) as $as_ty:ty ) => {
        jni::sys::jvalue {
            f: ($name as jni::sys::jfloat),
        }
    };
    ( $name:ident : prim ( jdouble  ) as $as_ty:ty ) => {
        jni::sys::jvalue {
            d: ($name as jni::sys::jdouble),
        }
    };

    // Objects (Java or Rust) — both end up as 'l'
    ( $name:ident : obj  ( $($j:tt)+ ) as $as_ty:ty ) => {
        jni::sys::jvalue {
            l: as_jobject_handle!($name),
        }
    };
    ( $name:ident : rust ( $as_r:ty ) as $as_use:ty ) => {
        jni::sys::jvalue {
            l: as_jobject_handle!($name),
        }
    };
}

// Build `[jvalue; N]` from a normalized arg list
macro_rules! build_jni_args_array {
    ( ( $( $an:ident : $ak:ident ( $($at:tt)* ) as $aas:ty ),* $(,)? ) ) => {{
        [ $( _jvalue_of_norm_arg!($an : $ak($($at)*) as $aas) ),* ]
    }};
}

// ----- Select the correct CallXxxMethodA for the return type -----
macro_rules! _call_api_for_ret {
    ( prim ( void     ) as $rty:ty ) => {
        CallVoidMethodA
    };
    ( prim ( jboolean ) as $rty:ty ) => {
        CallBooleanMethodA
    };
    ( prim ( jbyte    ) as $rty:ty ) => {
        CallByteMethodA
    };
    ( prim ( jchar    ) as $rty:ty ) => {
        CallCharMethodA
    };
    ( prim ( jshort   ) as $rty:ty ) => {
        CallShortMethodA
    };
    ( prim ( jint     ) as $rty:ty ) => {
        CallIntMethodA
    };
    ( prim ( jlong    ) as $rty:ty ) => {
        CallLongMethodA
    };
    ( prim ( jfloat   ) as $rty:ty ) => {
        CallFloatMethodA
    };
    ( prim ( jdouble  ) as $rty:ty ) => {
        CallDoubleMethodA
    };

    // Anything reference-like (Java object or "rust(&T)") returns an object
    ( obj  ( $($rt:tt)+ ) as $rty:ty ) => {
        CallObjectMethodA
    };
    ( rust ( $r:ty )       as $rty:ty ) => {
        CallObjectMethodA
    };
}

// ----- LOOKUP FN -----
macro_rules! jgen_method_lookup_fn {
    (
        this: $this:path,
        jname: $jname:ident,
        rname: $rname:ident,
        args: ( $( $an:ident : $ak:ident ( $($at:tt)* ) as $aas:ty ),* $(,)? ),
        ret:  $rk:ident ( $($rt:tt)* ) as $rty:ty
    ) => {
        paste! {
            fn [<_$rname _lookup>](env: &mut Env) -> Result<JMethodID> {
                let class: &JClass = <$this>::lookup_class()?;
                let sig = if any_rust!(( $( $an : $ak($($at)*) as $aas ),* ), ( $rk($($rt)*) as $rty )) == 0 {
                    build_sig_literal!(( $( $an : $ak($($at)*) as $aas ),* ) -> ( $rk($($rt)*) as $rty ))
                } else {
                    build_sig_format!(( $( $an : $ak($($at)*) as $aas ),* ) -> ( $rk($($rt)*) as $rty ))
                };
                env.get_method_id(class, stringify!($jname), &sig)
            }
        }
    };
}

// ----- CALL FN (build jvalue[] and pick the API) -----
macro_rules! jgen_method_call_fn {
    (
        this: $this:path,
        rname: $rname:ident,
        args: ( $( $an:ident : $ak:ident ( $($at:tt)* ) as $aas:ty ),* $(,)? ),
        ret:  $rk:ident ( $($rt:tt)* ) as $rty:ty
    ) => {
        paste! {
            fn [<_$rname _call>](
                env: Env,
                this: &$this,
                method_id: JMethodID,
                $( $an: _param_ty_from_norm!($ak($($at)*) as $aas) ),*
            ) -> Result<_ret_ty_from_norm!($rk($($rt)*) as $rty)> {
                // Build the jvalue array
                let jni_args = build_jni_args_array!(( $( $an : $ak($($at)*) as $aas ),* ));

                // Choose the correct CallXxxMethodA variant based on return
                let _ = &this; // silence unused for some linters
                // Your codebase likely wraps raw calls with a helper macro; we mirror your earlier example:
                jni_call_check_ex!(
                    this, v1_1,
                    _call_api_for_ret!($rk($($rt)*) as $rty),
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
        sig: ( $( $an:ident : $($aty:tt)+ ),* $(,)? ) -> $($rty:tt)+
    ) => {
        jgen_method_lookup_fn!(
            this: $this,
            jname: $jname,
            rname: $rname,
            args: jnorm_args!( $( $an : $($aty)+ ),* ),
            ret:  jnorm_ret!( $($rty)+ )
        );
        jgen_method_call_fn!(
            this: $this,
            rname: $rname,
            args: jnorm_args!( $( $an : $($aty)+ ),* ),
            ret:  jnorm_ret!( $($rty)+ )
        );
    };
}

macro_rules! jgen_bind_method {
    (
        this: $this:path,
        &jname:ident as $rname:ident,
        sig: ( $( $an:ident : $($aty:tt)+ ),* $(,)? ) -> $($rty:tt)+
    ) => {
        jgen_method_lookup_fn!(,
            this: $this,
            jname: $jname,
            // normalize each arg with jnorm_arg!
            args: jnorm_args!( $( $an : $($aty)+ ),* ),
            // normalize return
            ret:  jnorm_ret!( $($rty)+ )
        );
        jgen_method_call_fn!(
            fn $fname,
            this: $this,
            // normalize each arg with jnorm_arg!
            args: jnorm_args!( $( $an : $($aty)+ ),* ),
            // normalize return
            ret:  jnorm_ret!( $($rty)+ )
        );
    };
}

// Inner form that consumes normalized args/ret and generates the fn
macro_rules! jgen_method_call_fn {
    (
        fn $fname:ident,
        this: $this:path,
        args: ( $( $an:ident : $ak:ident ( $($at:tt)* ) as $aas:ty ),* ),
        ret:  $rk:ident ( $($rt:tt)* ) as $rty:ty
    ) => {
        fn $fname(this: &$this, $( $an: _param_ty_from_norm!($ak($($at)*) as $aas) ),*, env: Env) -> _ret_ty_from_norm!($rk($($rt)*) as $rty) {
            let class: &JClass = this::lookup_class();

            let sig = if any_rust!(( $( $an : $ak($($at)*) as $aas ),* ), ( $rk($($rt)*) as $rty )) == 0 {
                // all literal
                build_sig_literal!(( $( $an : $ak($($at)*) as $aas ),* ) -> ( $rk($($rt)*) as $rty ))
            } else {
                // at least one rust(...) → format!
                build_sig_format!(( $( $an : $ak($($at)*) as $aas ),* ) -> ( $rk($($rt)*) as $rty ))
            };

            // Use `class`, `sig`, `env`, and args to perform the JNI call:
            todo!()
        }
    };
}

// ------------------
// EXAMPLES / TESTS *
// ------------------

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
// name: rust(JString) as JString
const _: () = {
    let _ = stringify!(jnorm_arg!(s: JString));
};
// list form
const _: () = {
    let _ = stringify!(jnorm_args!(a: int, b: java.util.Map::Entry, c: JString));
};

// ----------- Minimal placeholders so this compiles in isolation -----------
type JObject = ();
type JClass = ();
type Env = ();
type jint = i32;
struct JFoo;
struct JString;
impl Reference for JString {
    fn class_name() -> Cow<'static, JNIStr> {
        unimplemented!()
    }
}

// -------------------- Example --------------------
jgen_bind_method!(
    this: JFoo,
    javaFunction as rust_function,
    sig: (a: &JString, b: java.lang.String as JString, c: jint) -> void
);
