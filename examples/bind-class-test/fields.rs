//! Support macros for `declare_reference_type` that emit field bindings

/// Compose a JNI field signature as `Cow::<JNIStr>::Borrowed()`
///
/// Returns a `Cow::Borrowed`
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_emit_field_sig_literal {
    ( ( $field_kind:ident $field:tt as $field_as:tt ) ) => {
        const {
            let s = concat!(
                $crate::__java_type_to_internal_literal!( $field_kind $field as $field_as ),
                "\0"
            );
            // Safety: the concat! above guarantees a null terminator with no interior nulls
            let cs = unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(s.as_bytes()) };
            let js = $crate::strings::JNIStr::from_cstr(cs);
            std::borrow::Cow::Borrowed(js)
        }
    };
}

/// Compose a JNI field signature dynamically into a `Cow::<JNIStr>::Owned()`
///
/// Returns a `Cow::Owned`
#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_emit_field_sig_dynamic_owned {
    ( ( $field_kind:ident $field:tt as $field_as:tt )
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
            extend_with_internal(&mut buf, &$crate::__any_type_to_internal_owned!( $field_kind $field as $field_as ));
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

#[doc(hidden)]
#[macro_export]
macro_rules! __jsig_emit_field_sig_cow {
    ( ( prim $prim:tt as $as:tt) ) => { $crate::__jsig_emit_field_sig_literal!( ( prim $prim as $as ) ) };
    ( ( obj $obj:tt as $as:tt) ) => { $crate::__jsig_emit_field_sig_literal!( ( obj $obj as $as ) ) };
    ( ( array $arr:tt as $as:tt) ) => { $crate::__jsig_emit_field_sig_literal!( ( array $arr as $as ) ) };
    ( ( rust $rust:tt as $as:tt) ) => { $crate::__jsig_emit_field_sig_dynamic_owned!( ( rust $rust as $as ) ) };
    ( ( array (rust, $($rest:tt)+ ) as $as:tt) ) => { $crate::__jsig_emit_field_sig_dynamic_owned!( ( array(rust, $($rest)+ ) as $as ) ) };
}

/// Shim that routes from normalization macros to the real callback
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_field_id_lookup__then_emit_cow {
    ( ( $($norm:tt)* ), $real_cb:ident $(, $($rest:tt)* )? ) => {
        $crate::$real_cb!{ ( $($norm)* ) $(, $($rest)* )? }
    };
}

/// Emit a FieldID lookup helper
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_field_id_lookup {
    (
        $this:path,
        $jname:literal,
        $rname:ident,
        $lookup_api:ident,
        $ret_id_ty:ty,
        ( $($fty:tt)+ )
    ) => {
        paste::paste! {
            fn [<_ $rname _lookup>](env: &mut Env, class: &JClass) -> $crate::errors::Result<$ret_id_ty> {
                let sig = $crate::__jsig_normalize_type_then!( __jgen_emit_field_id_lookup__then_emit_cow, {'any0, 'any1}, ( $($fty)+ ), __jsig_emit_field_sig_cow );
                env.$lookup_api(class, $jname, &sig)
            }
        }
    };
}

/// Emit JNI get field call based on normalized type
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_jni_sys_get_field {
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jboolean ) as $as:tt) ) => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind BooleanField>], ($this).as_raw(), $field_id.into_raw()) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jbyte ) as $as:tt) )    => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind ByteField>],    ($this).as_raw(), $field_id.into_raw()) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jchar ) as $as:tt) )    => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind CharField>],    ($this).as_raw(), $field_id.into_raw()) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jshort ) as $as:tt) )   => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind ShortField>],   ($this).as_raw(), $field_id.into_raw()) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jint ) as $as:tt) )     => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind IntField>],     ($this).as_raw(), $field_id.into_raw()) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jlong ) as $as:tt) )    => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind LongField>],    ($this).as_raw(), $field_id.into_raw()) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jfloat ) as $as:tt) )   => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind FloatField>],   ($this).as_raw(), $field_id.into_raw()) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( prim ( jdouble ) as $as:tt) )  => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind DoubleField>],  ($this).as_raw(), $field_id.into_raw()) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( obj  ( $($jt:tt)+ ) as ( $as_ty:ty ) ) ) => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; let ret_obj: jni::sys::jobject = $crate::jni_call_check_ex!($env, v1_1, [<$kind ObjectField>], ($this).as_raw(), $field_id.into_raw())?; Ok(<$as_ty>::from_raw($env, ret_obj)) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, ( rust ( $as_ty:ty, $lt:lifetime ) as $as:tt) ) => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; let ret_obj: jni::sys::jobject = $crate::jni_call_check_ex!($env, v1_1, [<$kind ObjectField>], ($this).as_raw(), $field_id.into_raw())?; Ok(<$as_ty>::from_raw($env, ret_obj)) } } }};
}

/// Emit JNI set field call based on normalized type
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_jni_sys_set_field {
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jboolean ) as $as:tt) ) => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind BooleanField>], ($this).as_raw(), $field_id.into_raw(), $val as jni::sys::jboolean) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jbyte ) as $as:tt) )    => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind ByteField>],    ($this).as_raw(), $field_id.into_raw(), $val as jni::sys::jbyte) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jchar ) as $as:tt) )    => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind CharField>],    ($this).as_raw(), $field_id.into_raw(), $val as jni::sys::jchar) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jshort ) as $as:tt) )   => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind ShortField>],   ($this).as_raw(), $field_id.into_raw(), $val as jni::sys::jshort) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jint ) as $as:tt) )     => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind IntField>],     ($this).as_raw(), $field_id.into_raw(), $val as jni::sys::jint) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jlong ) as $as:tt) )    => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind LongField>],    ($this).as_raw(), $field_id.into_raw(), $val as jni::sys::jlong) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jfloat ) as $as:tt) )   => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind FloatField>],   ($this).as_raw(), $field_id.into_raw(), $val as jni::sys::jfloat) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( prim ( jdouble ) as $as:tt) )  => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind DoubleField>],  ($this).as_raw(), $field_id.into_raw(), $val as jni::sys::jdouble) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( obj  ( $($jt:tt)+ ) as ( $as_ty:ty ) ) ) => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind ObjectField>], ($this).as_raw(), $field_id.into_raw(), __jsig_obj_reference_as_raw!($as_ty, $val)) } } }};
    ( $kind:ident, $env:expr, $this:expr, $field_id:expr, $val:ident, ( rust ( $as_ty:ty, $lt:lifetime ) as $as:tt) ) => {{ paste::paste! { unsafe { use $crate::refs::Reference as _; $crate::jni_call_check_ex!($env, v1_1, [<$kind ObjectField>], ($this).as_raw(), $field_id.into_raw(), __jsig_obj_reference_as_raw!($as_ty, $val)) } } }};
}

// Instance field get/set implementations
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_field_impl__with_norm_type {
    ( @static_field, $envGet:ty, $this:path, $ApiType:path, $rname:ident, $get:ident, $set:ident, ( $fk:ident $field:tt as $as:tt ), ( $($field_lt:lifetime)? ) ) => {
        paste::paste! {
            pub fn $get<'env_local>(env: $envGet) -> $crate::errors::Result< $crate::__jgen_emit_rust_return_type!('env_local, $fk $field as $as ) > {
                let api = $ApiType::get(env, &$crate::refs::LoaderContext::None)?;
                let class: &$crate::objects::JClass = api.class.as_ref();
                $crate::__jgen_emit_jni_sys_get_field!( GetStatic, env, class, api.[<$rname _field_id>], ( $fk $field as $as ) )
            }
            pub fn $set<$( $field_lt )?>(env: &Env, val: $crate::__jgen_emit_rust_method_arg_type!( $fk $field as $as )) -> $crate::errors::Result<()> {
                let api = $ApiType::get(env, &$crate::refs::LoaderContext::None)?;
                let class: &$crate::objects::JClass = api.class.as_ref();
                $crate::__jgen_emit_jni_sys_set_field!( SetStatic, env, class, api.[<$rname _field_id>], val, ( $fk $field as $as ) )
            }
        }
    };
    ( @field, $envGet:ty, $this:path, $ApiType:path, $rname:ident, $get:ident, $set:ident, ( $fk:ident $field:tt as $as:tt ), ( $($field_lt:lifetime)? ) ) => {
        paste::paste! {
            pub fn $get<'env_local>(&self, env: $envGet) -> $crate::errors::Result< __jgen_emit_rust_return_type!('env_local, $fk $field as $as ) > {
                let api = $ApiType::get(env, &$crate::refs::LoaderContext::None)?;
                $crate::__jgen_emit_jni_sys_get_field!( Get, env, self, api.[<$rname _field_id>], ( $fk $field as $as ) )
            }
            pub fn $set<$( $field_lt )?>(&self, env: &Env, val: $crate::__jgen_emit_rust_method_arg_type!( $fk $field as $as )) -> $crate::errors::Result<()> {
                let api = $ApiType::get(env, &$crate::refs::LoaderContext::None)?;
                $crate::__jgen_emit_jni_sys_set_field!( Set, env, self, api.[<$rname _field_id>], val, ( $fk $field as $as ) )
            }
        }
    };
}
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_field_impl__shim {
    ( ( prim $prim:tt as $as:tt), $field_lt:tt, @$for:ident, $this:path, $ApiType:path, $rname:ident, $get_rname:ident, $set_rname:ident ) => { $crate::__jgen_emit_field_impl__with_norm_type!( @$for, &Env<'env_local>, $this, $ApiType, $rname, $get_rname, $set_rname, ( prim $prim as $as ), $field_lt ); };
    ( ( $kind:ident $arg:tt as $as:tt ), $field_lt:tt, @$for:ident, $this:path, $ApiType:path, $rname:ident, $get_rname:ident, $set_rname:ident ) => { $crate::__jgen_emit_field_impl__with_norm_type!( @$for, &mut Env<'env_local>, $this, $ApiType, $rname, $get_rname, $set_rname, ( $kind $arg as $as ), $field_lt ); };
}
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_field_impl {
    ( @$for:ident, $this:path, $ApiType:path, $rname:ident, $get:ident, $set:ident, ( $($fty:tt)+ ) ) => {
        $crate::__jsig_normalize_type_then!( __then_call, ( $($fty)+ ), __jgen_emit_field_impl__shim, @$for, $this, $ApiType, $rname, $get, $set );
    };
}

/// Public macro to bind an instance field
#[macro_export]
macro_rules! jgen_bind_field {
    (
        this: $this:path,
        $jname:literal as $rname:ident,
        sig: ($($fty:tt)+)
    ) => {
        paste::paste!{
            impl [<$this API>] {
                $crate::__jgen_emit_field_id_lookup!(
                    $this,
                    $jname,
                    $rname,
                    get_field_id,
                    JFieldID,
                    ( $($fty)+ )
                );
            }
            impl $this {
                $crate::__jgen_emit_field_impl!(
                    @field,
                    $this,
                    [< $this API>],
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
        $jname:literal as $rname:ident,
        sig: $fty:path
    ) => {
        $crate::jgen_bind_field!(
            this: $this,
            $jname as $rname,
            sig: ( $fty )
        );
    };
}

/// Public macro to bind a static field
#[macro_export]
macro_rules! jgen_bind_static_field {
    (
        this: $this:path,
        $jname:literal as $rname:ident,
        sig: ($($fty:tt)+)
    ) => {
        paste::paste!{
            impl [<$this API>] {
                $crate::__jgen_emit_field_id_lookup!(
                    $this,
                    $jname,
                    $rname,
                    get_static_field_id,
                    JStaticFieldID,
                    ( $($fty)+ )
                );
            }
            impl $this {
                $crate::__jgen_emit_field_impl!(
                    @static_field,
                    $this,
                    [< $this API>],
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
        $jname:literal as $rname:ident,
        sig: $fty:path
    ) => {
        $crate::jgen_bind_static_field!(
            this: $this,
            $jname as $rname,
            sig: ( $fty )
        );
    };
}

/// Invoke a callback with parsed method components after reordering arguments
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__parse_field_body__finish {
    ( $field:tt, $field_lt:tt, $_lifetimes:tt, @$for:ident, $rname:ident, $get_rname:ident, $set_rname:ident, $jname:tt, $callback:ident $(, $($extra:tt)* )? ) => {
        $crate::$callback!{ @$for, $rname, $get_rname, $set_rname, $jname, $field, $field_lt $(, $($extra)* )? }
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
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__parse_field_body {
    // entrypoint
    ( @$for:ident $callback:ident, $rname:ident = { $($field_desc:tt)* } $(, $($extra:tt)* )?) => {
        //compile_error!(concat!("parsing: ", stringify!( $($field_desc)+ ) ));
        paste::paste! {
            $crate::__drt__parse_field_body!{@parse @$for $callback, $rname ( ($crate::_jni_str_camel!($rname)) ) ( ) ( $rname ) ( [<set_ $rname>] ) [ $($field_desc)* ] ( $(, $($extra)*)? ) }
        }
    };

    // name = literal
    ( @parse @$for:ident $cb:ident, $rname:ident ( $_jname:tt ) ( $($type:tt)* ) ( $get:ident ) ( $set:ident ) [ name = $jname:literal , $($more:tt)* ] $extra:tt ) => {
        $crate::__drt__parse_field_body!{@parse @$for $cb, $rname ( ($crate::_to_jni_str!($jname)) ) ( $($type)* ) ( $get ) ( $set ) [ $($more)* ] $extra }
    };
    ( @parse @$for:ident $cb:ident, $rname:ident ( $_jname:tt ) ( $($type:tt)* ) ( $get:ident ) ( $set:ident ) [ name = $jname:literal ] $extra:tt ) => {
        $crate::__drt__parse_field_body!{@finish @$for $cb, $rname ( ($crate::_to_jni_str!($jname)) ) ( $($type)* ) ( $get ) ( $set ) [] $extra }
    };

    // sig = (type)
    ( @parse @$for:ident $cb:ident, $rname:ident ( $jname:tt ) ( $($_type:tt)* ) ( $get:ident ) ( $set:ident ) [ sig = ( $($new_type:tt)* ) , $($more:tt)* ] $extra:tt ) => {
        $crate::__drt__parse_field_body!{@parse @$for $cb, $rname ( $jname ) ( ( $($new_type)* ) ) ( $get ) ( $set ) [ $($more)* ] $extra }
    };
    ( @parse @$for:ident $cb:ident, $rname:ident ( $jname:tt ) ( $($_type:tt)* ) ( $get:ident ) ( $set:ident ) [ sig = ( $($new_type:tt)* ) ] $extra:tt ) => {
        $crate::__drt__parse_field_body!{@finish @$for $cb, $rname ( $jname ) ( ( $($new_type)* ) ) ( $get ) ( $set ) [] $extra }
    };

    // sig = type:ident
    // Note: due to the 'Forwarding a matched fragment' rule we don't try and special case `type:path`
    // because our attempt to normalize will fail to match primitive types like `( jint )`
    // (since we can't match `:path` meta variables as tokens).
    ( @parse @$for:ident $cb:ident, $rname:ident ( $jname:tt ) ( $($_type:tt)* ) ( $get:ident ) ( $set:ident ) [ sig = $new_type:ident , $($more:tt)* ] $extra:tt ) => {
        $crate::__drt__parse_field_body!{@parse @$for $cb, $rname ( $jname ) ( ( $new_type ) ) ( $get ) ( $set ) [ $($more)* ] $extra }
    };
    ( @parse @$for:ident $cb:ident, $rname:ident ( $jname:tt ) ( $($_type:tt)* ) ( $get:ident ) ( $set:ident ) [ sig = $new_type:ident ] $extra:tt ) => {
        $crate::__drt__parse_field_body!{@finish @$for $cb, $rname ( $jname ) ( ( $new_type ) ) ( $get ) ( $set ) [] $extra }
    };

    // sig = ()
    ( @parse @$for:ident $cb:ident, $rname:ident ( $jname:tt ) ( $($_type:tt)* ) ( $get:ident ) ( $set:ident ) [ sig = () , $($more:tt)* ] $extra:tt ) => {
        compile_error!("field `sig` cannot be void `()`");
    };
    ( @parse @$for:ident $cb:ident, $rname:ident ( $jname:tt ) ( $($_type:tt)* ) ( $get:ident ) ( $set:ident ) [ sig = () ] $extra:tt ) => {
        compile_error!("field `sig` cannot be void `()`");
    };

    // unexpected
    ( @parse @$for:ident $cb:ident, $rname:ident ( $($jname:tt)* ) ( $($type:tt)* ) ( $get:ident ) ( $set:ident ) [ $bad:tt $($more:tt)* ] $extra:tt ) => {
        compile_error!(concat!("unexpected token in method body: ", stringify!($bad)));
    };

    // missing required keys
    ( @finish @$for:ident $cb:ident, $rname:ident ( ) ( $($type:tt)* ) ( $get:ident ) ( $set:ident ) [] $extra:tt ) => {
        compile_error!("field missing required `name` key")
    };
    ( @finish @$for:ident $cb:ident, $rname:ident ( $jname:tt ) ( ) ( $get:ident ) ( $set:ident ) [] $extra:tt ) => {
        compile_error!("field missing required `sig` key")
    };

    // finalize callback
    ( @finish @$for:ident $cb:ident, $rname:ident ( $jname:tt ) ( ( $($type:tt)+ ) ) ( $get:ident ) ( $set:ident ) [] ($(, $($extra:tt)* )?) ) => {
        //compile_error!(concat!("DEBUG: field type before normalization: ", stringify!( $($type)+ ) ));
        // We only have at most one type that may need an associated lifetime so we can provide a minimal bag of two
        // lifetimes (we provide two so we don't have to handle the bag becoming empty).
        $crate::__jsig_normalize_type_then!{ __drt__parse_field_body__finish, { 'any0, 'any1 }, ( $($type)* ), @$for, $rname, $get, $set, $jname, $cb $(, $($extra)*)? }
    };
}

/// Emit Rust instance field bindings into `impl $Type { ... }`
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_field_binding {
    ( @$for:ident, $rname:ident, $get:ident, $set:ident, $jname:tt, $field:tt, $field_lt:tt, $Type:path, $ApiType:path ) => {
        $crate::__jgen_emit_field_impl__shim!{ $field, $field_lt, @$for, $Type, $ApiType, $rname, $get, $set}
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __drt_emit_api_field_id_init {
    ( @$for:ident, $rname:ident, $get:ident, $set:ident, ($($jname:tt)*), ( $($type:tt)+ ), $field_lt:tt, $env:expr, $class:expr ) => {
        {
            let sig = $crate::__jsig_emit_field_sig_cow!( ( $($type)* ) );
            $env.get_field_id($class, $($jname)*, &sig)?
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __drt_emit_api_static_field_id_init {
    ( @$for:ident, $rname:ident, $get:ident, $set:ident, ($($jname:tt)*), ( $($type:tt)+ ), $field_lt:tt, $env:expr, $class:expr ) => {
        {
            let sig = $crate::__jsig_emit_field_sig_cow!( ( $($type)* ) );
            $env.get_static_field_id($class, $($jname)*, &sig)?
        }
    };
}
