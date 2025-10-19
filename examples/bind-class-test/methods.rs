//! Support macros for `declare_reference_type` that emit method bindings

#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_rust_method_arg_lifetime {
    ( prim ( void )) => {
        compile_error!("'void' is not a valid parameter type");
    };
    ( prim ( $p:ident )) => {};

    ( obj ($type:tt, $lt:lifetime) ) => {
        $lt
    };
    ( obj ($type:tt, $lt:lifetime) ) => {
        $lt
    };
    ( rust ($type:tt, $lt:lifetime) ) => {
        $lt
    };
    ( rust ($type:tt, $lt:lifetime) ) => {
        $lt
    };
    ( array ($kind:ident, $elem:tt, $dims:tt, $lt:lifetime) ) => {
        $lt
    };
    ( array ($kind:ident, $elem:tt, $dims:tt, $lt:lifetime) ) => {
        $lt
    };
}

/// Derive a borrowed Rust method argument type from a normalized type
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_rust_method_arg_type {
    ( prim ( void ) as $as:tt) => { compile_error!("'void' is not a valid parameter type"); };
    ( prim ( $p:ident ) as $as:tt) => { $crate::sys::$p };
    //( $lt:lifetime, $kind:ident $arg:tt as ($($as:tt)*) ) => { impl AsRef<$crate::__jgen_inject_lt_in_type!($lt, $($as)*)>};

    ( obj ($type:tt, $lt:lifetime) as ($($as:tt)*) ) => { impl AsRef<$crate::__jgen_inject_lt_in_type!($lt, $($as)*)> };
    ( obj ($type:tt, $lt:lifetime) as ($($as:tt)*) ) => { impl AsRef<$crate::__jgen_inject_lt_in_type!($lt, $($as)*)> };
    ( rust ($type:tt, $lt:lifetime) as ($($as:tt)*) ) => { impl AsRef<$crate::__jgen_inject_lt_in_type!($lt, $($as)*)> };
    ( rust ($type:tt, $lt:lifetime) as ($($as:tt)*) ) => { impl AsRef<$crate::__jgen_inject_lt_in_type!($lt, $($as)*)> };
    ( array ($kind:ident, $elem:tt, $dims:tt, $lt:lifetime) as ($($as:tt)*) ) => { impl AsRef<$crate::__jgen_inject_lt_in_type!($lt, $($as)*)> };
    ( array ($kind:ident, $elem:tt, $dims:tt, $lt:lifetime) as ($($as:tt)*) ) => { impl AsRef<$crate::__jgen_inject_lt_in_type!($lt, $($as)*)> };

    /*
    ( obj ($type:tt, $lt:lifetime) as ( $as_path0:ident $( :: $path:ident )* ) ) => { impl AsRef<$as_path0$(::$path)* <'local>> };
    ( obj ($type:tt, $lt:lifetime) as ( $as_path0:ident $( :: $path:ident )* <$($gen:ty),*> ) ) => { impl AsRef<$as_path0$(::$path)* <'local, $($gen),*>> };
    ( rust ($type:tt, $lt:lifetime) as ( $as_path0:ident $( :: $path:ident )* ) ) => { impl AsRef<$as_path0$(::$path)*<'local>> };
    ( rust ($type:tt, $lt:lifetime) as ( $as_path0:ident $( :: $path:ident )* <$($gen:ty),*> ) ) => { impl AsRef<$as_path0$(::$path)*<'local, $($gen),*>> };
    ( array ($kind:ident, $elem:tt, $dims:tt, $lt:lifetime) as ( $as_path0:ident $( :: $path:ident )* ) ) => { impl AsRef<$as_path0$(::$path)*<'local>> };
    ( array ($kind:ident, $elem:tt, $dims:tt, $lt:lifetime) as ( $as_path0:ident $( :: $path:ident )* <$($gen:ty),*> ) ) => { impl AsRef<$as_path0$(::$path)*<'local, $($gen),*>> };
    */
}

/// Derive an owned Rust method argument type from a normalized type
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_rust_owned_method_arg_type {
    ( prim ( void ) as $as:tt) => { compile_error!("'void' is not a valid parameter type"); };
    ( prim ( $p:ident ) as $as:tt) => { $crate::sys::$p };
    ( obj ($type:tt, $lt:lifetime) as ( $as_path0:ident $( :: $path:ident )* ) ) => { $as_path0$(::$path)* <'local> };
    ( obj ($type:tt, $lt:lifetime) as ( $as_path0:ident $( :: $path:ident )* <$($gen:ty),*> ) ) => { $as_path0$(::$path)* <'local, $($gen,)*> };
    ( rust  ($type:tt, $lt:lifetime) as ( $as_path0:ident $( :: $path:ident )* ) ) => { $as_path0$(::$path)*<'local> };
    ( rust  ($type:tt, $lt:lifetime) as ( $as_path0:ident $( :: $path:ident )* <$($gen:ty),*> ) ) => { $as_path0$(::$path)*<'local, $($gen,)*> };
    ( array ($kind:ident, $elem:tt, $dims:tt, $lt:lifetime) as ( $as_path0:ident $( :: $path:ident )* ) ) => { $as_path0$(::$path)*<'local> };
    ( array ($kind:ident, $elem:tt, $dims:tt, $lt:lifetime) as ( $as_path0:ident $( :: $path:ident )* <$($gen:ty),*> ) ) => { $as_path0$(::$path)*<'local, $($gen,)*> };
}

/// Takes a Reference type like `path::Foo<path::Bar<path::Baz>>` and produces `path::Foo<'lt,
/// path::Bar<'lt, path::Baz<'lt>>>` by injecting the lifetime as the first generic argument of
/// every generic type.
///
/// It special-cases inner primitive types like `jint` that don't need a lifetime.
///
/// In practice the only generic reference types that we expect to see are `JPrimitiveArray` and
/// `JObjectArray` but we may as well try to handle other basic cases on a best effort basis.
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_inject_lt_in_type {
    // Entrypoints
    // Assume any paths without generics are References, not primitive types (since we assume this macro isn't used with prim() types)
    ( $lt:lifetime, $first:ident $(:: $path:ident)* ) => {
        $crate::__jgen_inject_lt_in_type!{@finish $first$(::$path)*<$lt> }
    };
    ( $lt:lifetime, $($raw:tt)+ ) => {
        //compile_error!(concat!("Debug: starting with type: ", stringify!($($raw)+)));
        $crate::__jgen_inject_lt_in_type!{@inject $lt, () ( $($raw)+ ) }
    };

    // Generic type path: add lifetime as first generic arg and recurse into all generic type arguments.
    ( @inject $lt:lifetime, ($($left:tt)*) ( $p0:ident $( :: $ps:ident )* < $($rest:tt)* ) ) => {
        //compile_error!(concat!("Debug: left = ", stringify!($($left)*), ", match = ", stringify!($p0$(::$ps)*<), ", rest = ", stringify!($($rest)*)));

        $crate::__jgen_inject_lt_in_type!{@inject $lt, ($($left)*, $p0$(::$ps)* < $lt ) ($($rest)*)  }
    };

    // Plain inner type path (that doesn't open more generic parameters) that indicates we're finished
    //
    // We want to recognise primitive types here since they shouldn't be given a lifetime.
    //
    // We want to avoid too much repetition, which means we want to match the end of any terminal
    // path so it can be checked in one place.
    //
    // This is complicated by the fact that `>>` is a single token, and the ambiguity that would
    // come from trying to match `$($p:ident ::)+` followed by a terminal `$end:ident`
    //
    // We assume that primitive types never need a path longer than four segments. In practice they
    // should only need at most three segments (e.g. `crate::sys::jint`), but we allow one extra
    // segment just in case (e.g. `other_crate::jni::sys::jint`)
    ( @inject $lt:lifetime, ($($left:tt)*) ( $p0:ident :: $p1:ident :: $p2:ident :: $end:ident > $($rest:tt)* ) ) => {
        $crate::__jgen_inject_lt_in_type!{@prim_check $lt, ($($left)*) ($p0::$p1::$p2::) $end (> $($rest)* ) }
    };
    ( @inject $lt:lifetime, ($($left:tt)*) ( $p0:ident :: $p1:ident :: $p2:ident :: $end:ident >> $($rest:tt)* ) ) => {
        $crate::__jgen_inject_lt_in_type!{@prim_check $lt, ($($left)*) ($p0::$p1::$p2::) $end (>> $($rest)* ) }
    };
    ( @inject $lt:lifetime, ($($left:tt)*) ( $p0:ident :: $p1:ident :: $end:ident > $($rest:tt)* ) ) => {
        $crate::__jgen_inject_lt_in_type!{@prim_check $lt, ($($left)*) ($p0::$p1::) $end (> $($rest)* ) }
    };
    ( @inject $lt:lifetime, ($($left:tt)*) ( $p0:ident :: $p1:ident :: $end:ident >> $($rest:tt)* ) ) => {
        $crate::__jgen_inject_lt_in_type!{@prim_check $lt, ($($left)*) ($p0::$p1::) $end (>> $($rest)* ) }
    };
    ( @inject $lt:lifetime, ($($left:tt)*) ( $p0:ident :: $end:ident > $($rest:tt)* ) ) => {
        $crate::__jgen_inject_lt_in_type!{@prim_check $lt, ($($left)*) ($p0::) $end (> $($rest)* ) }
    };
    ( @inject $lt:lifetime, ($($left:tt)*) ( $p0:ident :: $end:ident >> $($rest:tt)* ) ) => {
        $crate::__jgen_inject_lt_in_type!{@prim_check $lt, ($($left)*) ($p0::) $end (>> $($rest)* ) }
    };
    ( @inject $lt:lifetime, ($($left:tt)*) ( $end:ident > $($rest:tt)* ) ) => {
        $crate::__jgen_inject_lt_in_type!{@prim_check $lt, ($($left)*) () $end (> $($rest)* ) }
    };
    ( @inject $lt:lifetime, ($($left:tt)*) ( $end:ident >> $($rest:tt)* ) ) => {
        $crate::__jgen_inject_lt_in_type!{@prim_check $lt, ($($left)*) () $end (>> $($rest)* ) }
    };
    // Assume longer paths aren't primitive types...
    ( @inject $lt:lifetime, ($($left:tt)*) ( $p0:ident $(:: $end:ident)+ > $($rest:tt)* ) ) => {
        $crate::__jgen_inject_lt_in_type!{@finish $($left)*, $p0$(::$end)+<$lt> > $($rest)* }
    };
    ( @inject $lt:lifetime, ($($left:tt)*) ( $p0:ident $(:: $end:ident)+ >> $($rest:tt)* ) ) => {
        $crate::__jgen_inject_lt_in_type!{@finish $($left)*, $p0$(::$end)+<$lt> >> $($rest)* }
    };

    ( @prim_check $lt:lifetime, ($($left:tt)*) ($($prefix:tt)*) jint ( $($right:tt)* ) ) => {
        //compile_error!(concat!("DEBUG: left = ", stringify!($($left)*), ", prefix = ", stringify!($($prefix)*), ", end = jint, right = ", stringify!($($right)*)));
        $crate::__jgen_inject_lt_in_type!{@finish $($left)*, $($prefix)*jint $($right)* }
    };
    ( @prim_check $lt:lifetime, ($($left:tt)*) ($($prefix:tt)*) $end:ident ( $($right:tt)* ) ) => {
        //compile_error!(concat!("DEBUG: left = ", stringify!($($left)*), ", prefix = ", stringify!($($prefix)*), ", end = ", stringify!($end), ", right = ", stringify!($($right)*)));
        $crate::__jgen_inject_lt_in_type!{@finish $($left)*, $($prefix)*$end<$lt> $($right)* }
    };

    (@finish , $($output:tt)*) => {
        $($output)*
    };
    (@finish $($output:tt)*) => {
        $($output)*
    };
}

/*
/// Derive a Rust method return type from a normalized type
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_rust_return_type {
    ( $lt:lifetime, prim ( void ) as $as:tt) => { () };
    ( $lt:lifetime, prim ( $p:ident ) as $as:tt) => { $crate::sys::$p };
    ( $lt:lifetime, obj $obj:tt as ( $as_path0:ident $( :: $path:ident )* ) ) => { $as_path0$(::$path)* <$lt> };
    ( $lt:lifetime, obj $obj:tt as ( $as_path0:ident $( :: $path:ident )* <$($gen:ty),*> ) ) => { $as_path0$(::$path)* <$lt, $($gen,)*> };

    ( $lt:lifetime, rust $rust:tt as ( $as_path0:ident $( :: $path:ident )* ) ) => { $as_path0$(::$path)*<$lt> };
    ( $lt:lifetime, rust $rust:tt as ( $as_path0:ident $( :: $path:ident )* <$($gen:ty),*> ) ) => { $as_path0$(::$path)*<$lt, $($gen,)*> };

    ( $lt:lifetime, array $array:tt as ( $as_path0:ident $( :: $path:ident )* ) ) => { $as_path0$(::$path)*<$lt> };
    ( $lt:lifetime, array $array:tt as ( $as_path0:ident $( :: $path:ident )* <$($gen:ty),*> ) ) => { $as_path0$(::$path)*<$lt, $($gen,)*> };

    { @add_lt () () }
}
*/

/// Derive a Rust method return type from a normalized type
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_rust_return_type {
    ( $lt:lifetime, prim ( void ) as $as:tt) => { () };
    ( $lt:lifetime, prim ( $p:ident ) as $as:tt) => { $crate::sys::$p };

    // For object-like types: inject lifetime recursively into the provided `as (...)` type.
    ( $lt:lifetime, obj $obj:tt as ( $($as:tt)+ ) ) => {
        $crate::__jgen_inject_lt_in_type!($lt, $($as)+)
    };

    // For rust wrapper types: same treatment as objects.
    ( $lt:lifetime, rust $rust:tt as ( $($as:tt)+ ) ) => {
        $crate::__jgen_inject_lt_in_type!($lt, $($as)+)
    };

    // For arrays: same treatment; this enables deep `JObjectArray<...>` nesting.
    ( $lt:lifetime, array $array:tt as ( $($as:tt)+ ) ) => {
        $crate::__jgen_inject_lt_in_type!($lt, $($as)+)
    };
}

/// Emit a JNI call to a method with the given normalized return type
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_jni_sys_call {
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr, ( prim ( void ) as $as:tt)
    ) => {
        $crate::__pastey! { unsafe { $crate::jni_call_check_ex!($env, v1_1, [<$kind VoidMethodA>], <_ as $crate::refs::Reference>::as_raw($this), $method_id, $jni_args.as_ptr()) } }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr, ( prim ( jboolean ) as $as:tt)
    ) => {
        $crate::__pastey! { unsafe { $crate::jni_call_check_ex!($env, v1_1, [<$kind BooleanMethodA>], <_ as $crate::refs::Reference>::as_raw($this), $method_id, $jni_args.as_ptr()) } }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr, ( prim ( jbyte ) as $as:tt)
    ) => {
        $crate::__pastey! { unsafe { $crate::jni_call_check_ex!($env, v1_1, [<$kind ByteMethodA>], <_ as $crate::refs::Reference>::as_raw($this), $method_id, $jni_args.as_ptr()) } }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr, ( prim ( jchar ) as $as:tt)
    ) => {
        $crate::__pastey! { unsafe { $crate::jni_call_check_ex!($env, v1_1, [<$kind CharMethodA>], <_ as $crate::refs::Reference>::as_raw($this), $method_id, $jni_args.as_ptr()) } }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr, ( prim ( jshort ) as $as:tt)
    ) => {
        $crate::__pastey! { unsafe { $crate::jni_call_check_ex!($env, v1_1, [<$kind ShortMethodA>], <_ as $crate::refs::Reference>::as_raw($this), $method_id, $jni_args.as_ptr()) } }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr, ( prim ( jint ) as $as:tt)
    ) => {
        $crate::__pastey! { unsafe { $crate::jni_call_check_ex!($env, v1_1, [<$kind IntMethodA>], <_ as $crate::refs::Reference>::as_raw($this), $method_id, $jni_args.as_ptr()) } }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr, ( prim ( jlong ) as $as:tt)
    ) => {
        $crate::__pastey! { unsafe { $crate::jni_call_check_ex!($env, v1_1, [<$kind LongMethodA>], <_ as $crate::refs::Reference>::as_raw($this), $method_id, $jni_args.as_ptr()) } }
    };

    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr, ( prim ( jfloat ) as $as:tt)
    ) => {
        $crate::__pastey! { unsafe { $crate::jni_call_check_ex!($env, v1_1, [<$kind FloatMethodA>], <_ as $crate::refs::Reference>::as_raw($this), $method_id, $jni_args.as_ptr()) } }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr, ( prim ( jdouble ) as $as:tt)
    ) => {
        $crate::__pastey! { unsafe { $crate::jni_call_check_ex!($env, v1_1, [<$kind DoubleMethodA>], <_ as $crate::refs::Reference>::as_raw($this), $method_id, $jni_args.as_ptr()) } }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr, ( obj $obj:tt as ( $as_ty:ty ) )
    ) => {
        $crate::__pastey! { unsafe {
            let ret_obj: $crate::sys::jobject = $crate::jni_call_check_ex!($env, v1_1, [<$kind ObjectMethodA>], <_ as $crate::refs::Reference>::as_raw($this), $method_id, $jni_args.as_ptr())?;
            Ok(<$as_ty>::from_raw($env, ret_obj))
        } }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr, ( rust $rust:tt as ( $as_ty:ty ) )
    ) => {
        $crate::__pastey! { unsafe {
            let ret_obj: $crate::sys::jobject = $crate::jni_call_check_ex!($env, v1_1, [<$kind ObjectMethodA>], <_ as $crate::refs::Reference>::as_raw($this), $method_id, $jni_args.as_ptr())?;
            Ok(<$as_ty>::from_raw($env, ret_obj))
        } }
    };
    (
        $kind:ident, $env:expr, $this:expr, $method_id:expr, $jni_args:expr, ( array $array:tt as ( $as_ty:ty ) )
    ) => {
        $crate::__pastey! { unsafe {
            let ret_obj: $crate::sys::jobject = $crate::jni_call_check_ex!($env, v1_1, [<$kind ObjectMethodA>], <_ as $crate::refs::Reference>::as_raw($this), $method_id, $jni_args.as_ptr())?;
            Ok(<$as_ty>::from_raw($env, ret_obj))
        } }
    };
}

/// Emit a method ID lookup function for the given method name and signature
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_method_id_lookup {
    (
        $this:path,
        $jname:literal,
        $rname:ident,
        $lookup_api:ident,
        ( $($args:tt)* ),
        ( $($ret:tt)+ )
    ) => {
        $crate::__pastey! {
            fn [<_ $rname _lookup>](env: &mut $crate::Env, class: &$crate::objects::JClass) -> $crate::errors::Result<JMethodID> {
                let sig = &$crate::__jsig_normalize_args_ret_then!( __jsig_emit_sig_cow, ( $($args)* ), ( $($ret)+ ) );
                env.$lookup_api(class, $jname, &sig)
            }
        }
    };
}

// Final emitter: we now have normalized args and ret; generate the whole fn
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_method_impl__with_norm_args_ret {
    (
        @constructor,
        $envType:ty,
        $this:path,
        $ApiType:path,
        ($($vis:tt)*),
        $rname:ident,
        ( $( $an:ident : $ak:ident $arg:tt as $arg_as:tt ),* $(,)? ),
        ( $( $arg_lt:lifetime ),* ),
        ( $rk:ident $ret:tt as $ret_as:tt ),
        ( $( $fn:path )* ),
        ( $( $error_policy:tt )* )
    ) => {

        $crate::__pastey! {
            $($vis)* fn $rname<'env_local, $( $arg_lt ),*> (
                env: $envType,
                $( $an: $crate::__jgen_emit_rust_method_arg_type!($ak $arg as $arg_as ) ),*
            ) -> $crate::errors::Result< __jgen_emit_rust_return_type!('env_local, rust ($this) as ($this)) > {
                let api = $ApiType::get(env, &$crate::refs::LoaderContext::None)?;
                let jni_args = $crate::__jsig_args_to_jvalue_array!( ( $( $an : $ak $arg as $arg_as ),* ) );

                unsafe {
                    use $crate::refs::Reference as _;
                    let class: &$crate::objects::JClass = api.class.as_ref();
                    let ret_obj: jni::sys::jobject = $crate::jni_call_check_ex!(
                        env,
                        v1_1,
                        NewObjectA,
                        class.as_raw(),
                        api.[<$rname _method_id>].into_raw(),
                        jni_args.as_ptr()
                    )?;
                    Ok($this::from_raw(env, ret_obj))
                }
            }
        }
    };
    (
        @static_method,
        $envType:ty,
        $this:path,
        $ApiType:path,
        ($($vis:tt)*),
        $rname:ident,
        ( $( $an:ident : $ak:ident $arg:tt as $arg_as:tt ),* $(,)? ),
        ( $( $arg_lt:lifetime ),* ),
        ( $rk:ident $ret:tt as $ret_as:tt ),
        ( $( $fn:path )* ),
        ( $( $error_policy:tt )* )
    ) => {
        $crate::__pastey! {
            $($vis)* fn $rname<'env_local, $( $arg_lt ),*> (
                env: $envType,
                $( $an: $crate::__jgen_emit_rust_method_arg_type!( $ak $arg as $arg_as ) ),*
            ) -> $crate::errors::Result< __jgen_emit_rust_return_type!('env_local, $rk $ret as $ret_as ) > {
                let api = $ApiType::get(env, &$crate::refs::LoaderContext::None)?;
                let jni_args = $crate::__jsig_args_to_jvalue_array!( ( $( $an : $ak $arg as $arg_as ),* ) );

                let class: &$crate::objects::JClass = api.class.as_ref();
                $crate::__jgen_emit_jni_sys_call!{
                    CallStatic,
                    env,
                    class,
                    api.[<$rname _method_id>].into_raw(),
                    jni_args,
                    ( $rk $ret as $ret_as )
                }
            }
        }
    };
    (
        @method,
        $envType:ty,
        $this:path,
        $ApiType:path,
        ($($vis:tt)*),
        $rname:ident,
        ( $( $an:ident : $ak:ident $arg:tt as $arg_as:tt ),* $(,)? ),
        ( $( $arg_lt:lifetime ),* ),
        ( $rk:ident $ret:tt as $ret_as:tt ),
        ( $( $fn:path )* ),
        ( $( $error_policy:tt )* )
    ) => {
        $crate::__pastey! {
            $($vis)* fn $rname<'env_local, $( $arg_lt ),*> (
                &self,
                env: $envType,
                $( $an: $crate::__jgen_emit_rust_method_arg_type!( $ak $arg as $arg_as ) ),*
            ) -> $crate::errors::Result< $crate::__jgen_emit_rust_return_type!('env_local, $rk $ret as $ret_as ) > {
                let api = $ApiType::get(env, &$crate::refs::LoaderContext::None)?;
                let jni_args = $crate::__jsig_args_to_jvalue_array!( ( $( $an : $ak $arg as $arg_as ),* ) );

                $crate::__jgen_emit_jni_sys_call!{
                    Call,
                    env,
                    self,
                    api.[<$rname _method_id>].into_raw(),
                    jni_args,
                    ( $rk $ret as $ret_as )
                }
            }
        }
    };
}

/// A shim before __jgen_emit_method_impl__with_norm_args_ret that determines whether &Env or &mut Env is needed
#[doc(hidden)]
#[macro_export]
macro_rules! __jgen_emit_method_impl__with_norm_args_ret__shim {
    // Primitive return types (including void) use &Env
    (
        $args:tt,
        $arg_lts:tt,
        ( prim $prim:tt as $as:tt ),
        @$for:ident,
        $this:path,
        $ApiType:path,
        $vis:tt,
        $rname:ident,
        ( $( $fn:path )* ),
        ( $( $error_policy:ty )* )
    ) => {
        $crate::__jgen_emit_method_impl__with_norm_args_ret!( @$for, &Env<'env_local>, $this, $ApiType, $vis, $rname, $args, $arg_lts, ( prim $prim as $as ), ( $( $fn )* ), ( $( $error_policy )* ) );
    };
    // Object return types use &mut Env because they create new local references
    (
        $args:tt,
        $arg_lts:tt,
        ( obj $obj:tt as $as:tt ),
        @$for:ident,
        $this:path,
        $ApiType:path,
        $vis:tt,
        $rname:ident,
        ( $( $fn:path )* ),
        ( $( $error_policy:ty )* )
    ) => {
        $crate::__jgen_emit_method_impl__with_norm_args_ret!( @$for, &mut Env<'env_local>, $this, $ApiType, $vis, $rname, $args, $arg_lts, ( obj $obj as $as ), ( $( $fn )* ), ( $( $error_policy )* ) );
    };
    // Rust return types use &mut Env because they create new local references
    (
        $args:tt,
        $arg_lts:tt,
        ( rust $rust:tt as $as:tt ),
        @$for:ident,
        $this:path,
        $ApiType:path,
        $vis:tt,
        $rname:ident,
        ( $( $fn:path )* ),
        ( $( $error_policy:ty )* )
    ) => {
        $crate::__jgen_emit_method_impl__with_norm_args_ret!( @$for, &mut Env<'env_local>, $this, $ApiType, $vis, $rname, $args, $arg_lts, ( rust $rust as $as ), ( $( $fn )* ), ( $( $error_policy )* ) );
    };
    // Array return types use &mut Env because they create new local references
    (
        $args:tt,
        $arg_lts:tt,
        ( array $array:tt as $as:tt ),
        @$for:ident,
        $this:path,
        $ApiType:path,
        $vis:tt,
        $rname:ident,
        ( $( $fn:path )* ),
        ( $( $error_policy:ty )* )
    ) => {
        $crate::__jgen_emit_method_impl__with_norm_args_ret!( @$for, &mut Env<'env_local>, $this, $ApiType, $vis, $rname, $args, $arg_lts, ( array $array as $as ), ( $( $fn )* ), ( $( $error_policy )* ) );
    };
}

/// Emit Rust instance method bindings into `impl $Type { ... }`
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_method_binding {
    ( @$for:ident, $vis:tt, $rname:ident, $jname:tt, $args:tt, $arg_lts:tt, ( $($ret:tt)+ ), ( $($fn:tt)* ), ( $($error_policy:tt)* ), $Type:path, $ApiType:path ) => {
        $crate::__jgen_emit_method_impl__with_norm_args_ret__shim!{ $args, $arg_lts, ( $($ret)+ ), @$for, $Type, $ApiType, $vis, $rname, ( $($fn)* ), ( $($error_policy)* )}
    };
}

// Note: up until now $jname was passed in parens to allow lazy `stringify!()` expansion, so we unwrap it here
#[doc(hidden)]
#[macro_export]
macro_rules! __drt_emit_api_method_id_init {
    ( @$for:ident, $vis:tt, $rname:ident, ($($jname:tt)*), $args:tt, $arg_lts:tt, $ret:tt, ( $($fn:tt)* ), ( $($error_policy:tt)* ), $env:expr, $class:expr ) => {
        {
            let sig = &$crate::__jsig_emit_sig_cow!( $args, $arg_lts, $ret );
            $env.get_method_id($class, $($jname)*, &sig)?
        }
    };
}

// Note: up until now $jname was passed in parens to allow lazy `stringify!()` expansion, so we unwrap it here
#[doc(hidden)]
#[macro_export]
macro_rules! __drt_emit_api_static_method_id_init {
    ( @$for:ident, $vis:tt, $rname:ident, ($($jname:tt)*), $args:tt, $arg_lts:tt, $ret:tt, ( $($fn:tt)* ), ( $($error_policy:tt)* ), $env:expr, $class:expr ) => {{
        let sig = &$crate::__jsig_emit_sig_cow!($args, $arg_lts, $ret);
        $env.get_static_method_id($class, $($jname)*, &sig)?
    }};
}
/// Invoke a callback with parsed method components after reordering arguments
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__parse_method_body__finish {
    ( $args:tt, $arg_lts:tt, ($($ret:tt)+), @$for:ident, $vis:tt, $rname:ident, $jname:tt, ($($fn:tt)*), ($($error_policy:tt)*), $callback:ident $(, $($extra:tt)* )? ) => {
        $crate::$callback!{ @$for, $vis, $rname, $jname, $args, $arg_lts, ( $($ret)+ ), ( $($fn)* ), ( $($error_policy)* ) $(, $($extra)* )? }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _jni_str_camel {
    ($ident:ident) => {
        const {
            $crate::__pastey! {
                let s: &str = concat!(stringify!([<$ident:lower_camel>]), "\0");

                // SAFETY: `stringify!` of an identifier cannot contain interior NULs,
                // and we just appended exactly one trailing NUL with concat!.
                let cstr = unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(s.as_bytes()) };
                $crate::strings::JNIStr::from_cstr(cstr)
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _to_jni_str {
    ($s:literal) => {
        const {
            $crate::__pastey! {
                let s: &str = concat!($s, "\0");

                // SAFETY: `stringify!` of an identifier cannot contain interior NULs,
                // and we just appended exactly one trailing NUL with concat!.
                let cstr = unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(s.as_bytes()) };
                $crate::strings::JNIStr::from_cstr(cstr)
            }
        }
    };
}

/// Parse one method body `{ name = "javaName", sig = (args) -> ret, fn = path, error_policy = Type }` in any key order.
///
/// Keys can be repeated and the last one wins.
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__parse_method_body {
    // entrypoints, with/without visibility

    ( @static_native_method $callback:ident, $rname:ident = { $($method_desc:tt)* }, $error_policy:ty $(, $($extra:tt)* )? ) => {
        //compile_error!(concat!("parsing: ", stringify!( $($tt)+ ) ));
        // Note: we pass the jname around as a single :tt in parens so the stringify can be expanded lazily
        $crate::__drt__parse_method_body!{@parse @static_native_method, $callback, $rname, () ( ($crate::_jni_str_camel!($rname)) ) ( ) ( ) ( ) ( $error_policy ) [ $($method_desc)* ] ( $(, $($extra)*)? )}
    };
    ( @native_method $callback:ident, $rname:ident = { $($method_desc:tt)* }, $error_policy:ty $(, $($extra:tt)* )? ) => {
        //compile_error!(concat!("parsing: ", stringify!( $($tt)+ ) ));
        // Note: we pass the jname around as a single :tt in parens so the stringify can be expanded lazily
        $crate::__drt__parse_method_body!{@parse @native_method, $callback, $rname, () ( ($crate::_jni_str_camel!($rname)) ) ( ) ( ) ( ) ( $error_policy ) [ $($method_desc)* ] ( $(, $($extra)*)? )}
    };

    ( @constructor $callback:ident, $rname:ident = { $($method_desc:tt)* } $(, $($extra:tt)* )? ) => {
        //compile_error!(concat!("parsing: ", stringify!( $($tt)+ ) ));
        $crate::__drt__parse_method_body!{@parse @constructor, $callback, $rname, (pub) ( __init__ ) ( ) ( ) ( ) ( ) [ $($method_desc)* ] ( $(, $($extra)*)? )}
    };
    ( @$for:ident $callback:ident, $rname:ident = { $($method_desc:tt)* } $(, $($extra:tt)* )? ) => {
        //compile_error!(concat!("parsing: ", stringify!( $($tt)+ ) ));
        // Note: we pass the jname around as a single :tt in parens so the stringify can be expanded lazily
        $crate::__drt__parse_method_body!{@parse @$for, $callback, $rname, (pub) ( ($crate::_jni_str_camel!($rname)) ) ( ) ( ) ( ) ( ) [ $($method_desc)* ] ( $(, $($extra)*)? )}
    };

    // Parsing finished
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($args:tt)* ) ( $($ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@check @$for, $cb, $rname, $vis ( $jname ) ( $($args)* ) ( $($ret)* ) ( $($fn)* ) ( ($($error_policy)*) ) [] $extra}
    };

    // name = literal
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $($_jname:tt)* ) ( $($args:tt)* ) ( $($ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ name = $jname:literal , $($more:tt)* ] $extra:tt ) => {
        //compile_error!(concat!("parsed name: name = ", stringify!($jname), ", args = ", stringify!($($args)*), ", ret = ", stringify!($($ret)*), ", fn = ", stringify!($($fn)*), ", error_policy = ", stringify!($($error_policy)*), ", more = ", stringify!($($more)*)));
        $crate::__drt__parse_method_body!{@parse @$for, $cb, $rname, $vis ( ($crate::_to_jni_str!($jname)) ) ( $($args)* ) ( $($ret)* ) ( $($fn)* ) ( $($error_policy)* ) [ $($more)* ] $extra}
    };
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $($_jname:tt)* ) ( $($args:tt)* ) ( $($ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ name = $jname:literal ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@check @$for, $cb, $rname, $vis ( ($crate::_to_jni_str!($jname)) ) ( $($args)* ) ( $($ret)* ) ( $($fn)* ) ( $($error_policy)* ) [] $extra}
    };

    // vis = pub
    ( @parse @$for:ident, $cb:ident, $rname:ident, $_vis:tt ( $jname:tt ) ( $($args:tt)* ) ( $($ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ vis = pub , $($more:tt)* ] $extra:tt ) => {
        //compile_error!(concat!("parsed name: name = ", stringify!($jname), ", args = ", stringify!($($args)*), ", ret = ", stringify!($($ret)*), ", fn = ", stringify!($($fn)*), ", error_policy = ", stringify!($($error_policy)*), ", more = ", stringify!($($more)*)));
        $crate::__drt__parse_method_body!{@parse @$for, $cb, $rname, (pub) ( $jname ) ( $($args)* ) ( $($ret)* ) ( $($fn)* ) ( $($error_policy)* ) [ $($more)* ] $extra}
    };
    ( @parse @$for:ident, $cb:ident, $rname:ident, $_vis:tt ( $jname:tt ) ( $($args:tt)* ) ( $($ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ vis = pub ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@check @$for, $cb, $rname, (pub) ( $jname ) ( $($args)* ) ( $($ret)* ) ( $($fn)* ) ( $($error_policy)* ) [] $extra}
    };
    // vis = pub(<scope>)
    ( @parse @$for:ident, $cb:ident, $rname:ident, $_vis:tt ( $jname:tt ) ( $($args:tt)* ) ( $($ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ vis = pub($($scope:tt)*) , $($more:tt)* ] $extra:tt ) => {
        //compile_error!(concat!("parsed name: name = ", stringify!($jname), ", args = ", stringify!($($args)*), ", ret = ", stringify!($($ret)*), ", fn = ", stringify!($($fn)*), ", error_policy = ", stringify!($($error_policy)*), ", more = ", stringify!($($more)*)));
        $crate::__drt__parse_method_body!{@parse @$for, $cb, $rname, (pub($($scope)*)) ( $jname ) ( $($args)* ) ( $($ret)* ) ( $($fn)* ) ( $($error_policy)* ) [ $($more)* ] $extra}
    };
    ( @parse @$for:ident, $cb:ident, $rname:ident, $_vis:tt ( $jname:tt ) ( $($args:tt)* ) ( $($ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ vis = pub($($scope:tt)*) ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@check @$for, $cb, $rname, (pub($($scope)*)) ( $jname ) ( $($args)* ) ( $($ret)* ) ( $($fn)* ) ( $($error_policy)* ) [] $extra}
    };

    // sig = (args) -> (ret)
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($_args:tt)* ) ( $($_ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ sig = ( $($args:tt)* ) -> ( $($ret:tt)+ ) , $($more:tt)* ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@parse @$for, $cb, $rname, $vis ( $jname ) ( ( $($args)* ) ) ( ( $($ret)+ ) ) ( $($fn)* ) ( $($error_policy)* ) [ $($more)* ] $extra}
    };
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($_args:tt)* ) ( $($_ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ sig = ( $($args:tt)* ) -> ( $($ret:tt)+ ) ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@check @$for, $cb, $rname, $vis ( $jname ) ( ( $($args)* ) ) ( ( $($ret)+ ) ) ( $($fn)* ) ( $($error_policy)* ) [] $extra}
    };

    // sig = (args) -> ret:ident
    // Note: due to the 'Forwarding a matched fragment' rule we don't try and special case `ret:path`
    // because our attempt to normalize into `( $ret )` will fail to match primitive types like `( jint )`
    // because we're no longer able to match the tokens.
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($_args:tt)* ) ( $($_ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ sig = ( $($args:tt)* ) -> $ret:ident , $($more:tt)* ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@parse @$for, $cb, $rname, $vis ( $jname ) ( ( $($args)* ) ) ( ( $ret ) ) ( $($fn)* ) ( $($error_policy)* ) [ $($more)* ] $extra}
    };
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($_args:tt)* ) ( $($_ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ sig = ( $($args:tt)* ) -> $ret:ident ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@check @$for, $cb, $rname, $vis ( $jname ) ( ( $($args)* ) ) ( ( $ret ) ) ( $($fn)* ) ( $($error_policy)* ) [] $extra}
    };

    // sig = (args) -> ()
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($_args:tt)* ) ( $($_ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ sig = ( $($args:tt)* ) -> () , $($more:tt)* ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@parse @$for, $cb, $rname, $vis ( $jname ) ( ( $($args)* ) ) ( ( void ) ) ( $($fn)* ) ( $($error_policy)* ) [ $($more)* ] $extra}
    };
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($_args:tt)* ) ( $($_ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ sig = ( $($args:tt)* ) -> () ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@check @$for, $cb, $rname, $vis ( $jname ) ( ( $($args)* ) ) ( ( void ) ) ( $($fn)* ) ( $($error_policy)* ) [] $extra}
    };

    // sig = (args)
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($_args:tt)* ) ( $($_ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ sig = ( $($args:tt)* ) , $($more:tt)* ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@parse @$for, $cb, $rname, $vis ( $jname ) ( ( $($args)* ) ) ( ( void ) ) ( $($fn)* ) ( $($error_policy)* ) [ $($more)* ] $extra}
    };
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($_args:tt)* ) ( $($_ret:tt)* ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ sig = ( $($args:tt)* ) ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@check @$for, $cb, $rname, $vis ( $jname ) ( ( $($args)* ) ) ( ( void ) ) ( $($fn)* ) ( $($error_policy)* ) [] $extra}
    };

    // fn = path
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($args:tt)* ) ( $($ret:tt)* ) ( $($_fn:tt)* ) ( $($error_policy:tt)* ) [ fn = $fn:path , $($more:tt)* ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@parse @$for, $cb, $rname, $vis ( $jname ) ( $($args)* ) ( $($ret)* ) ( ($fn) ) ( $($error_policy)* ) [ $($more)* ] $extra}
    };
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($args:tt)* ) ( $($ret:tt)* ) ( $($_fn:tt)* ) ( $($error_policy:tt)* ) [ fn = $fn:path ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@check @$for, $cb, $rname, $vis ( $jname ) ( $($args)* ) ( $($ret)* ) ( ($fn) ) ( $($error_policy)* ) [] $extra}
    };

    // error_policy = type
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($args:tt)* ) ( $($ret:tt)* ) ( $($fn:tt)* ) ( $($_error_policy:tt)* ) [ error_policy = $error_policy:ty , $($more:tt)* ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@parse @$for, $cb, $rname, $vis ( $jname ) ( $($args)* ) ( $($ret)* ) ( $($fn)* ) ( $error_policy ) [ $($more)* ] $extra}
    };
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($args:tt)* ) ( $($ret:tt)* ) ( $($fn:tt)* ) ( $($_error_policy:tt)* ) [ error_policy = $error_policy:ty ] $extra:tt ) => {
        $crate::__drt__parse_method_body!{@check @$for, $cb, $rname, $vis ( $jname ) ( $($args)* ) ( $($ret)* ) ( $($fn)* ) ( $error_policy ) [] $extra}
    };

    // unexpected
    ( @parse @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($_args:tt)* ) ( $($_ret:tt)+ ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [ $bad:tt $($more:tt)* ] $extra:tt ) => {
        compile_error!(concat!("unexpected token in method body: ", stringify!($bad)));
    };

    // missing required keys (don't need to check name since it has a default)
    ( @check @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $($args:tt)* ) ( ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [] $extra:tt ) => {
        compile_error!(concat!("method '", stringify!($rname), "' missing required `sig` key"));
    };

    // validation and finalization - @constructor with non-void return
    //( @finish @constructor, $cb:ident, $rname:ident ( ($jname:literal) ) ( ( $($args:tt)* ) ) ( ( $ret:tt ) ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [] ($($extra:tt)*) ) => {
    //    compile_error!(concat!("define_reference_type!: constructor '", stringify!($rname), "' must have void return type, found: ", stringify!($ret)));
    //};

    // validation - @constructor, @method or @static_method with fn set
    ( @check @constructor, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( ( $($args:tt)* ) ) ( ( void ) ) ( ($fn:path) ) ( $($error_policy:tt)* ) [] $extra:tt ) => {
        compile_error!(concat!("constructor '", stringify!($rname), "' should not have `fn` key set"));
    };
    ( @check @method, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( ( $($args:tt)* ) ) ( ( $($ret:tt)+ ) ) ( ($fn:path) ) ( $($error_policy:tt)* ) [] $extra:tt ) => {
        compile_error!(concat!("method '", stringify!($rname), "' should not have `fn` key set"));
    };
    ( @check @static_method, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( ( $($args:tt)* ) ) ( ( $($ret:tt)+ ) ) ( ($fn:path) ) ( $($error_policy:tt)* ) [] $extra:tt ) => {
        compile_error!(concat!("static method '", stringify!($rname), "' should not have `fn` key set"));
    };

    ( @check @constructor, $cb:ident, $rname:ident, $vis:tt ( __init__ ) ( $args:tt ) ( ( void ) ) $fn:tt $error_policy:tt [] $extra:tt) => {
        $crate::__drt__parse_method_body!{@finish @constructor, $cb, $rname, $vis ( ($crate::strings::JNIStr::from_cstr(c"<init>")) ) ( $args ) ( ( void ) ) $fn $error_policy [] $extra}
    };
    ( @check @constructor, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $args:tt ) ( ( void ) ) $fn:tt $error_policy:tt [] $extra:tt) => {
        compile_error!(concat!("Constructor '", stringify!($rname), "' must not override the 'name' key"));
    };
    ( @check @constructor, $cb:ident, $rname:ident, $vis:tt ( __init__ ) ( $args:tt ) ( $ret:tt ) $fn:tt $error_policy:tt [] $extra:tt) => {
        compile_error!(concat!("Constructor signature for '", stringify!($rname), "' must have void return type"));
    };
    ( @check @constructor, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $args:tt ) ( $ret:tt ) $fn:tt $error_policy:tt [] $extra:tt) => {
        compile_error!(concat!("Constructor '", stringify!($rname), "' must have void return type and not override the 'name' key"));
    };

    ( @check @static_method, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $args:tt ) ( $ret:tt ) $fn:tt $error_policy:tt [] $extra:tt) => {
        $crate::__drt__parse_method_body!{@finish @static_method, $cb, $rname, $vis ( $jname ) ( $args ) ( $ret ) $fn $error_policy [] $extra}
    };
    ( @check @method, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $args:tt ) ( $ret:tt ) $fn:tt $error_policy:tt [] $extra:tt) => {
        $crate::__drt__parse_method_body!{@finish @method, $cb, $rname, $vis ( $jname ) ( $args ) ( $ret ) $fn $error_policy [] $extra}
    };
    ( @check @static_native_method, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $args:tt ) ( $ret:tt ) $fn:tt $error_policy:tt [] $extra:tt) => {
        $crate::__drt__parse_method_body!{@finish @static_native_method, $cb, $rname, $vis ( $jname ) ( $args ) ( $ret ) $fn $error_policy [] $extra}
    };
    ( @check @native_method, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( $args:tt ) ( $ret:tt ) $fn:tt $error_policy:tt [] $extra:tt) => {
        $crate::__drt__parse_method_body!{@finish @native_method, $cb, $rname, $vis ( $jname ) ( $args ) ( $ret ) $fn $error_policy [] $extra}
    };

    // finalize callback with extra args
    ( @finish @$for:ident, $cb:ident, $rname:ident, $vis:tt ( $jname:tt ) ( ( $($args:tt)* ) ) ( ( $($ret:tt)+ ) ) ( $($fn:tt)* ) ( $($error_policy:tt)* ) [] ( $(, $($extra:tt)* )? )) => {
        //compile_error!(concat!("DEBUG: parsed method ", stringify!($rname), " = { name = ", stringify!($jname), ", sig = (", stringify!($($args)*), ") -> (", stringify!($($ret)+), "), for = ", stringify!($for), ", fn = ", stringify!($($fn)*), ", error_policy = ", stringify!($($error_policy)*), " }, extra = ", stringify!($($extra)+) ));
        $crate::__jsig_normalize_args_ret_then!{ __drt__parse_method_body__finish, ( $($args)* ), ( $($ret)+ ), @$for, $vis, $rname, $jname, ( $($fn)* ), ( $($error_policy)* ), $cb $(, $($extra)*)? }
    };
}
