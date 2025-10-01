//use crate::objects::JClass;
//use crate::refs::LoaderContext;
//use crate::Env;

#[cfg(doc)]
use crate::{objects::JObject, refs::Reference};

/// Call the initializer through a shim to help with type inference.
/// Helper macro to emit From implementation and cast method for aliases
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_aliases {
    ($Type:ident, [ $($aliases:tt)* ]) => {
        $crate::__drt__process_aliases! { $Type, $($aliases)* }
    };
}

/// Helper macro to process individual aliases
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__process_aliases {
    // Handle syntax: method_name = Type
    ($Type:ident, $method_name:ident = $AliasType:ident $(, $($rest:tt)*)?  ) => {
        // Generate From implementation
        impl<'l> From<$Type<'l>> for $AliasType<'l> {
            fn from(value: $Type<'l>) -> $AliasType<'l> {
                let raw = value.into_raw();
                unsafe { <$AliasType as $crate::refs::Reference>::kind_from_raw(raw) }
            }
        }

        $(
            $crate::__drt__process_aliases! { $Type, $($rest)* }
        )?
    };

    // Base case - no more aliases
    ($Type:ident,) => {};
    ($Type:ident) => {};
}

/// Helper macro to emit cast methods for aliases
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_alias_methods {
    ($Type:ident, [ $($aliases:tt)* ]) => {
        $crate::__drt__process_alias_methods! { $Type, $($aliases)* }
    };
}

/// Helper macro to process individual alias methods
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__process_alias_methods {
    // Handle syntax: method_name = Type - generate cast method
    ($Type:ident, $method_name:ident = $AliasType:ident $(, $($rest:tt)*)?  ) => {
        #[doc = concat!(r#"Casts this `"#, stringify!($Type), r#"` to a `"#, stringify!($AliasType), r#"`

This does not require a runtime type check since any `"#, stringify!($Type), r#"` is also a `"#, stringify!($AliasType), r#"`"#)]
        pub fn $method_name(&self) -> $crate::refs::Cast<'local, '_, $AliasType<'local>> {
            // SAFETY: we know that any instance of this type is also an instance of the alias type
            unsafe { $crate::refs::Cast::<$AliasType>::new_unchecked(self) }
        }

        $(
            $crate::__drt__process_alias_methods! { $Type, $($rest)* }
        )?
    };

    // Base case - no more aliases
    ($Type:ident,) => {};
    ($Type:ident) => {};
}

/// Helper macro to emit alias cast method
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_alias_method {
    ($Type:ident, $method_name:ident = $AliasType:ident) => {
        #[doc = concat!(r#"Casts this `"#, stringify!($Type), r#"` to a `"#, stringify!($AliasType), r#"`

This does not require a runtime type check since any `"#, stringify!($Type), r#"` is also a `"#, stringify!($AliasType), r#"`"#)]
        pub fn $method_name(&self) -> $crate::refs::Cast<'local, '_, $AliasType<'local>> {
            // SAFETY: we know that any instance of this type is also an instance of the alias type
            unsafe { $crate::refs::Cast::<$AliasType>::new_unchecked(self) }
        }
    };
    ($Type:ident, $AliasType:ident) => {
        // No method generated for old-style aliases without method names
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __drt__expand_init {
    ($Type:ident, $env:ident, $loader:ident, __drt__InitKindEnvClass, $Init:expr) => {{
        let class = $loader.load_class_for_type::<$Type>(true, $env)?;
        Self::call_init($env, &class, $Init)
    }};
    ($Type:ident, $env:ident, $loader:ident, __drt__InitKindLoader, $Init:expr) => {
        Self::call_init_with_loader($env, $loader, $Init)
    };
}

/// The actual emitter, parameterized by the resolved API ident.
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_with_api {
    (
        $ApiTy:ident,
        $Type:ident,
        $Class:expr,
        $RawTy:ident,
        $InitKind:ident,
        $Init:expr,
        [ $($Aliases:tt)* ],
        { $($Methods:tt)* },
        { $($StaticMethods:tt)* },
        { $($Fields:tt)* },
        { $($StaticFields:tt)* } $(,)?
    ) => {
        paste::paste!{
            // ---------- API struct ----------
            //pub(crate) struct $ApiTy {
            //    class: $crate::refs::Global<$crate::objects::JClass<'static>>,
            //    $( $Fields )*
            //}

            impl $ApiTy {
                #[allow(unused)]
                fn call_init<F, R>(env: &mut $crate::Env, class: &JClass, init: F) -> R
                where
                    F: FnOnce(&mut $crate::Env, &JClass) -> R,
                {
                    init(env, class)
                }

                #[allow(unused)]
                fn call_init_with_loader<F, R>(env: &mut $crate::Env, loader: &$crate::refs::LoaderContext, init: F) -> R
                where
                    F: FnOnce(&mut $crate::Env, &$crate::refs::LoaderContext) -> R,
                {
                    init(env, loader)
                }

                fn get<'any_local>(
                    env: &$crate::Env<'_>,
                    loader_context: &$crate::refs::LoaderContext<'any_local, '_>,
                ) -> $crate::errors::Result<&'static Self> {
                    static CELL: once_cell::sync::OnceCell<$ApiTy> = once_cell::sync::OnceCell::new();
                    CELL.get_or_try_init(|| {
                        env.with_local_frame($crate::DEFAULT_LOCAL_FRAME_CAPACITY, |env| {
                            $crate::__drt__expand_init!($Type,
                                env,
                                loader_context,
                                $InitKind,
                                $Init
                            )
                        })
                    })
                }
            }

            // ---------- Wrapper and impls (unchanged from your version) ----------
            #[doc = concat!(r#"A `"#, $Class, r#"` wrapper that is tied to a JNI local reference frame.

See the [`JObject`] documentation for more information about reference
wrappers, how to cast them, and local reference frame lifetimes.

[`JObject`]: $crate::objects::JObject
"#)]
            #[repr(transparent)]
            #[derive(Debug, Default)]
            pub struct $Type<'local>($crate::objects::JObject<'local>);

            impl<'local> AsRef<$Type<'local>> for $Type<'local> {
                #[inline] fn as_ref(&self) -> &$Type<'local> { self }
            }
            impl<'local> AsRef<$crate::objects::JObject<'local>> for $Type<'local> {
                #[inline] fn as_ref(&self) -> &$crate::objects::JObject<'local> { self }
            }
            impl<'local> ::std::ops::Deref for $Type<'local> {
                type Target = $crate::objects::JObject<'local>;
                #[inline] fn deref(&self) -> &Self::Target { &self.0 }
            }
            impl<'local> From<$Type<'local>> for $crate::objects::JObject<'local> {
                #[inline] fn from(other: $Type<'local>) -> $crate::objects::JObject<'local> { other.0 }
            }

            impl<'local> $Type<'local> {
                #[doc = concat!(r#"Creates a [`"#, stringify!($Type), r#"`] that wraps the given `raw` [jobject]

# Safety

- `raw` must be a valid raw JNI local reference (or `null`).
- `raw` must be an instance of `"#, $Class, r#"`.
- There must not be any other owning [Reference] wrapper for the same reference.
- The local reference must belong to the current thread and not outlive the
  JNI stack frame associated with the [Env] `'local` lifetime.

[jobject]: crate::sys::jobject
[Reference]: crate::refs::Reference
[Env]: crate::Env
"#)]
                #[inline]
                pub unsafe fn from_raw<'env_inner>(
                    env: &$crate::Env<'env_inner>,
                    raw: $crate::sys::$RawTy,
                ) -> $Type<'env_inner> {
                    let jobj: $crate::sys::jobject = raw as $crate::sys::jobject;
                    $Type($crate::objects::JObject::from_raw(env, jobj))
                }

                #[doc = concat!(r#"Creates a new null reference.

Null references are always valid and do not belong to a local reference frame. Therefore,
the returned [`"#, stringify!($Type), r#"`] always has the `'static` lifetime."#)]
                #[inline]
                pub const fn null() -> $Type<'static> {
                    $Type($crate::objects::JObject::null())
                }

                /// Unwrap to the raw jni type.
                #[inline]
                pub fn into_raw(self) -> $crate::sys::$RawTy {
                    (self.0.into_raw()) as $crate::sys::$RawTy
                }

                #[doc = concat!(r#"Cast a local reference to a [`"#, stringify!($Type), r#"`]

This will do a runtime (`IsInstanceOf`) check that the object is an instance of `"#, $Class, r#"`.

Also see these other options for casting local or global references to a [`"#, stringify!($Type), r#"`]:
- [Env::as_cast](crate::Env::as_cast)
- [Env::new_cast_local_ref](crate::Env::new_cast_local_ref)
- [Env::cast_local](crate::Env::cast_local)
- [Env::new_cast_global_ref](crate::Env::new_cast_global_ref)
- [Env::cast_global](crate::Env::cast_global)

# Errors

Returns [Error::WrongObjectType] if the `IsInstanceOf` check fails.

[Error::WrongObjectType]: crate::errors::Error::WrongObjectType
"#)]
                #[inline]
                pub fn cast_local<'any_local>(
                    obj: impl $crate::refs::Reference
                       + Into<$crate::objects::JObject<'any_local>>
                       + AsRef<$crate::objects::JObject<'any_local>>,
                    env: &mut $crate::Env<'_>,
                ) -> $crate::errors::Result<$Type<'any_local>> {
                    env.cast_local::<$Type>(obj)
                }

                // ---------- Alias cast methods ----------
                $crate::__drt__emit_alias_methods!($Type, [ $($Aliases)* ]);
            }

                        // ---------- Safe upcasts ----------
            $crate::__drt__emit_aliases!($Type, [ $($Aliases)* ]);

            // ---------- Reference impl ----------
            unsafe impl $crate::refs::Reference for $Type<'_> {
                type Kind<'env> = $Type<'env>;
                type GlobalKind = $Type<'static>;

                #[inline]
                fn as_raw(&self) -> $crate::sys::jobject { self.0.as_raw() }

                #[inline]
                fn class_name() -> ::std::borrow::Cow<'static, $crate::strings::JNIStr> {
                    const CLASS_NAME: &$crate::strings::JNIStr = $crate::strings::JNIStr::from_cstr(
                        match ::std::ffi::CStr::from_bytes_with_nul(concat!($Class, "\0").as_bytes()) {
                            Ok(cstr) => cstr,
                            Err(_) => panic!("Class name is not a valid C string"),
                        }
                    );
                    ::std::borrow::Cow::Borrowed(CLASS_NAME)
                }

                #[inline]
                fn lookup_class<'caller>(
                    env: &$crate::Env<'_>,
                    loader_context: $crate::refs::LoaderContext,
                ) -> $crate::errors::Result<
                    impl ::std::ops::Deref<
                        Target = $crate::refs::Global<$crate::objects::JClass<'static>>
                    > + 'caller
                > {
                    let api = $ApiTy::get(env, &loader_context)?;
                    Ok(&api.class)
                }

                #[inline]
                unsafe fn kind_from_raw<'env>(local_ref: $crate::sys::jobject) -> Self::Kind<'env> {
                    $Type($crate::objects::JObject::kind_from_raw(local_ref))
                }

                #[inline]
                unsafe fn global_kind_from_raw(global_ref: $crate::sys::jobject) -> Self::GlobalKind {
                    $Type($crate::objects::JObject::global_kind_from_raw(global_ref))
                }
            }
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

#[doc(hidden)]
#[macro_export]
macro_rules! __define_reference_type_gen {
    (
        type      = $Type:ident,
        class     = $Class:expr,
        raw       = $RawTy:ident,
        api       = $ApiName:ident,
        init_kind = $InitKind:ident,
        init      = $Init:expr,
        aliases   = [ $($Aliases:tt)* ],
        methods   = { $($Methods:tt)* },
        static_methods = { $($StaticMethods:tt)* },
        fields    = { $($Fields:tt)* },
        static_fields = { $($StaticFields:tt)* },
        $(,)?
    ) => {
        $crate::__drt__with_api_ident!(
            $ApiName,
            $Type,
            __drt__emit_with_api,
            $Type,
            $Class,
            $RawTy,
            $InitKind,
            $Init,
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
macro_rules! __drt__emit_init_wrapper {
    (
        $InitKind:ident,
        $InitExpr:expr,
        type   = $Type:ident,
        class  = $Class:expr,
        raw    = $RawIdent:ident,
        api    = $Api:ident,
        aliases = [ $($Aliases:tt)* ],
        methods = { $($Methods:tt)* },
        static_methods = { $($StaticMethods:tt)* },
        fields = { $($Fields:tt)* },
        static_fields = { $($StaticFields:tt)* },
    ) => {
        $crate::__define_reference_type_gen! {
            type      = $Type,
            class     = $Class,
            raw       = $RawIdent,
            api       = $Api,
            init_kind = $InitKind,
            init      = $InitExpr,
            aliases   = [ $($Aliases)* ],
            methods   = { $($Methods)* },
            static_methods = { $($StaticMethods)* },
            fields    = { $($Fields)* },
            static_fields = { $($StaticFields)* },
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __drt__dispatch_init {
    (__drt_Closure(()), __drt_Closure(()), $emit:ident, { $($args:tt)* }) => {
        compile_error!("define_reference_type!: expected exactly one of `init` or `init_with_loader`");
    };
    (__drt_Closure($Init:tt), __drt_Closure(()), $emit:ident, { $($args:tt)* }) => {
        $crate::$emit! {
            __drt__InitKindEnvClass,
            $Init,
            $($args)*
        }
    };
    (__drt_Closure(()), __drt_Closure($Init:tt), $emit:ident, { $($args:tt)* }) => {
        $crate::$emit! {
            __drt__InitKindLoader,
            $Init,
            $($args)*
        }
    };
    (__drt_Closure($Init:tt), __drt_Closure($InitWithLoader:tt), $emit:ident, { $($args:tt)* }) => {
        compile_error!(concat!("define_reference_type!: expected exactly one of `init` or `init_with_loader`, but both were provided, init = ", stringify!($Init), ", init_with_loader = ", stringify!($InitWithLoader)));
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
        init   = __drt_Closure($Init:tt),
        init_with_loader = __drt_Closure($InitWithLoader:tt),
        aliases = [ $($Aliases:tt)* ],
        methods = { $($Methods:tt)* },
        static_methods = { $($StaticMethods:tt)* },
        fields = { $($Fields:tt)* },
        static_fields = { $($StaticFields:tt)* },
    ) => {
        $crate::__drt__dispatch_init!(
            __drt_Closure($Init),
            __drt_Closure($InitWithLoader),
            __drt__emit_init_wrapper,
            {
                type   = $Type,
                class  = $Class,
                raw    = $RawIdent,
                api    = $Api,
                aliases = [ $($Aliases)* ],
                methods = { $($Methods)* },
                static_methods = { $($StaticMethods)* },
                fields = { $($Fields)* },
                static_fields = { $($StaticFields)* },
            }
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
            (init, (__drt_Closure(())))
            (init_with_loader, (__drt_Closure(())))
            (aliases, [])
            (methods, {})
            (static_methods, {})
            (fields, {})
            (static_fields, {})
        ]);
    };

    // Error cases for missing required fields (should be caught in finalize, but keep strictness)
    (@parse_tokens { type = (), class = ($Class:expr), pairs = [$($pairs:tt)*] }) => {
        compile_error!("define_reference_type!: missing required `type` field");
    };
    (@parse_tokens { type = ($Type:ident), class = (), pairs = [$($pairs:tt)*] }) => {
        compile_error!("define_reference_type!: missing required `class` field");
    };
    (@parse_tokens { type = (), class = (), pairs = [$($pairs:tt)*] }) => {
        compile_error!("define_reference_type!: missing required `type` and `class` fields");
    };

    // Tokenization rules (top-level):
    (@parse_tokens { type = (), class = $Class:tt, pairs = [$($acc:tt)*] } type = $value:ident $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = ($value), class = $Class, pairs = [$($acc)*] } $($rest)* }
    };
    (@parse_tokens { type = ($set:ident), class = $Class:tt, pairs = [$($acc:tt)*] } type = $value:ident $($rest:tt)*) => {
        compile_error!("define_reference_type!: duplicate `type` key");
    };

    (@parse_tokens { type = $Type:tt, class = (), pairs = [$($acc:tt)*] } class = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = ($value), pairs = [$($acc)*] } $($rest)* }
    };
    (@parse_tokens { type = $Type:tt, class = (), pairs = [$($acc:tt)*] } class = $value:expr) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = ($value), pairs = [$($acc)*] } }
    };
    (@parse_tokens { type = $Type:tt, class = ($set:expr), pairs = [$($acc:tt)*] } class = $value:expr, $($rest:tt)*) => {
        compile_error!("define_reference_type!: duplicate `class` key");
    };
    (@parse_tokens { type = $Type:tt, class = ($set:expr), pairs = [$($acc:tt)*] } class = $value:expr) => {
        compile_error!("define_reference_type!: duplicate `class` key");
    };

    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } raw = $value:ident $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (raw, $value)] } $($rest)* }
    };
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } api = $value:ident $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (api, $value)] } $($rest)* }
    };

    // wrap closures to avoid comma ambiguity during parsing
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } init = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (init, (__drt_Closure(($value))))] } $($rest)* }
    };
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } init = $value:expr) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (init, (__drt_Closure(($value))))] } }
    };

    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } init_with_loader = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (init_with_loader, (__drt_Closure(($value))))] } $($rest)* }
    };
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } init_with_loader = $value:expr) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (init_with_loader, (__drt_Closure(($value))))] } }
    };

    // aliases
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } as = [$($value:tt)*] $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (aliases, [$($value)*])] } $($rest)* }
    };

    // direct members keys
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } methods = {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (methods, {$($value)*})] } $($rest)* }
    };
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } static_methods = {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (static_methods, {$($value)*})] } $($rest)* }
    };
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } fields = {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (fields, {$($value)*})] } $($rest)* }
    };
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } static_fields = {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (static_fields, {$($value)*})] } $($rest)* }
    };

    // skip commas
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } , $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)*] } $($rest)* }
    };

    // unexpected
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } $bad:tt $($rest:tt)*) => {
        compile_error!(concat!("Unexpected token in define_reference_type: ", stringify!($bad)));
    };
}

// Lookups for pairs (order-agnostic)
#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_type {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!($($args)*); };
    ([(type, $Type:ident) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!($Type $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_type!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_class {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!($($args)*); };
    ([(class, $Class:expr) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!($Class $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_class!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_raw {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!($($args)*); };
    ([(raw, $Raw:ident) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!($Raw $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_raw!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_api {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!($($args)*); };
    ([(api, $Api:ident) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!($Api $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_api!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_init {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!($($args)*); };
    ([(init, (__drt_Closure(($Init:tt)))) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!($Init $(, $args)*); };
    ([(init, (__drt_Closure($Init:tt))) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!($Init $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_init!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_init_with_loader {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!($($args)*); };
    ([(init_with_loader, (__drt_Closure(($Init:tt)))) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!($Init $(, $args)*); };
    ([(init_with_loader, (__drt_Closure($Init:tt))) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!($Init $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_init_with_loader!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_aliases {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!($($args)*); };
    ([(aliases, [$($Aliases:tt)*]) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!([$($Aliases)*] $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_aliases!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_methods {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!($($args)*); };
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
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!($($args)*); };
    ([(fields, {$($Fields:tt)*}) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!({$($Fields)*} $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_fields!([$($rest)*], $found, $not $(, $args)*); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_static_fields {
    ([], $found:path, $not:path $(, $args:tt)*) => { $not!($($args)*); };
    ([(static_fields, {$($StaticFields:tt)*}) $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $found!({$($StaticFields)*} $(, $args)*); };
    ([$_h:tt $($rest:tt)*], $found:path, $not:path $(, $args:tt)*) => { $crate::__def_ref_lookup_static_fields!([$($rest)*], $found, $not $(, $args)*); };
}

// Finalize extraction and emit

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
        $crate::__def_ref_lookup_raw!([$($pairs)*], $crate::__def_ref_found_raw, $crate::__def_ref_unreachable, $Type, $Class, [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_unreachable {
    ($($any:tt)*) => {
        compile_error!("internal error: unreachable not-found branch taken")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_raw {
    ($Raw:ident, $Type:ident, $Class:expr, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_api!([$($pairs)*], $crate::__def_ref_found_api, $crate::__def_ref_unreachable, $Type, $Class, $Raw, [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_api {
    ($Api:ident, $Type:ident, $Class:expr, $Raw:ident, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_init!([$($pairs)*], $crate::__def_ref_found_init, $crate::__def_ref_unreachable, $Type, $Class, $Raw, $Api, [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_init {
    ($Init:tt, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_init_with_loader!([$($pairs)*], $crate::__def_ref_found_init_with_loader, $crate::__def_ref_unreachable, $Type, $Class, $Raw, $Api, $Init, [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_init_with_loader {
    ($InitWithLoader:tt, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $Init:tt, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_aliases!([$($pairs)*], $crate::__def_ref_found_aliases, $crate::__def_ref_unreachable, $Type, $Class, $Raw, $Api, $Init, $InitWithLoader, [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_aliases {
    ([$($Aliases:tt)*], $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $Init:tt, $InitWithLoader:tt, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_methods!([$($pairs)*], $crate::__def_ref_found_methods, $crate::__def_ref_unreachable, $Type, $Class, $Raw, $Api, $Init, $InitWithLoader, [$($Aliases)*], [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_methods {
    ({$($Methods:tt)*}, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $Init:tt, $InitWithLoader:tt, [$($Aliases:tt)*], [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_static_methods!([$($pairs)*], $crate::__def_ref_found_static_methods, $crate::__def_ref_unreachable, $Type, $Class, $Raw, $Api, $Init, $InitWithLoader, [$($Aliases)*], {$($Methods)*}, [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_static_methods {
    ({$($StaticMethods:tt)*}, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $Init:tt, $InitWithLoader:tt, [$($Aliases:tt)*], {$($Methods:tt)*}, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_fields!([$($pairs)*], $crate::__def_ref_found_fields, $crate::__def_ref_unreachable, $Type, $Class, $Raw, $Api, $Init, $InitWithLoader, [$($Aliases)*], {$($Methods)*}, {$($StaticMethods)*}, [$($pairs)*]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_fields {
    ({$($Fields:tt)*}, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $Init:tt, $InitWithLoader:tt, [$($Aliases:tt)*], {$($Methods:tt)*}, {$($StaticMethods:tt)*}, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_static_fields!([$($pairs)*], $crate::__def_ref_found_static_fields, $crate::__def_ref_unreachable, $Type, $Class, $Raw, $Api, $Init, $InitWithLoader, [$($Aliases)*], {$($Methods)*}, {$($StaticMethods)*}, {$($Fields)*});
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_static_fields {
    ({$($StaticFields:tt)*}, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $Init:tt, $InitWithLoader:tt, [$($Aliases:tt)*], {$($Methods:tt)*}, {$($StaticMethods:tt)*}, {$($Fields:tt)*}) => {
        $crate::__def_ref_emit! {
            type   = $Type,
            class  = $Class,
            raw    = $Raw,
            api    = $Api,
            init   = __drt_Closure($Init),
            init_with_loader = __drt_Closure($InitWithLoader),
            aliases = [$($Aliases)*],
            methods = {$($Methods)*},
            static_methods = {$($StaticMethods)*},
            fields = {$($Fields)*},
            static_fields = {$($StaticFields)*},
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_finalize {
    ([$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_type!([$($pairs)*], $crate::__def_ref_found_type, $crate::__def_ref_missing_type, [$($pairs)*]);
    };
}

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
