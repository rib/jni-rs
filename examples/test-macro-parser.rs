//! Standalone test for define_reference_type! macro parsing

/// The actual emitter, parameterized by the resolved API ident.
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_with_api {
    (
        $ApiTy:ident,
        $Type:ident,
        $Class:expr,
        $RawTy:ident,
        $LoadClass:expr,
        [ $($Aliases:tt)* ],
        { $($Methods:tt)* },
        { $($StaticMethods:tt)* },
        { $($Fields:tt)* },
        { $($StaticFields:tt)* } $(,)?
    ) => {
        paste::paste!{
            struct $ApiTy {
                class: $crate::refs::Global<$crate::objects::JClass>,
            }

            // Minimal demo: provide a get() that prints diagnostics
            impl $ApiTy {
                #[allow(unused)]
                fn _load_class_wrapper<F, R>(env: &mut $crate::Env, loader: &$crate::refs::LoaderContext, initialize: bool, load_class: F) -> $crate::errors::Result<R>
                where
                    F: FnOnce(&mut $crate::Env, &$crate::refs::LoaderContext, bool) -> $crate::errors::Result<R>,
                {
                    load_class(env, loader, initialize)
                }

                pub fn get(env: &Env, loader: &LoaderContext) -> $crate::errors::Result<&'static Self> {
                    println!("Generated API for type: {}", stringify!($Type));
                    println!("  class: {}", $Class);
                    println!("  raw: {}", stringify!($RawTy));
                    println!("  load_class: {}", stringify!($LoadClass));
                    println!("  aliases: {}", stringify!([$($Aliases)*]));
                    println!("  methods: {}", stringify!({$($Methods)*}));
                    println!("  static_methods: {}", stringify!({$($StaticMethods)*}));
                    println!("  fields: {}", stringify!({$($Fields)*}));
                    println!("  static_fields: {}", stringify!({$($StaticFields)*}));
                    static CELL: once_cell::sync::OnceCell<$ApiTy> = once_cell::sync::OnceCell::new();
                    CELL.get_or_try_init(|| {
                        env.with_local_frame(4, |env| {
                            let class = Self::_load_class_wrapper(env, &loader, false, $LoadClass)?;
                            // TODO: generate code to lookup method IDs
                            Ok(Self {
                                class: env.new_global_ref(&class)?,
                            })
                        })
                    })
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
        load_class = $LoadClass:expr,
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
            $LoadClass,
            [ $($Aliases)* ],
            { $($Methods)* },
            { $($StaticMethods)* },
            { $($Fields)* },
            { $($StaticFields)* }
        );
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
        $crate::__define_reference_type_gen!(
            type   = $Type,
            class  = $Class,
            raw    = $RawIdent,
            api    = $Api,
            load_class = $LoadClass,
            aliases = [ $($Aliases)* ],
            methods = { $($Methods)* },
            static_methods = { $($StaticMethods)* },
            fields = { $($Fields)* },
            static_fields = { $($StaticFields)* },
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

// ----------- Minimal placeholders so this compiles in isolation -----------

type JMethodID = *mut jni::sys::_jmethodID;

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
    use crate::objects::JClass;
    use crate::Env;
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

pub use objects::{JClass, JObject, JObjectArray, JPrimitiveArray, JString};

use crate::refs::LoaderContext;

fn main() {
    let mut env = Env::default();
    let env = &mut env;

    struct Test0;
    define_reference_type!(
        type = Test0,
        class = "java.lang.Object",
        raw = jstring,
    );
    let _ = Test0API::get(env, &LoaderContext::None);

    struct Test1;
    define_reference_type!(
        type = Test1,
        class = "java.lang.Object",
        raw = jstring,
        api = Custom0API
    );

    struct Test2;
    define_reference_type!(
        type = Test2,
        class = "java.lang.Object",
        api = Custom1API,
        raw = jstring,
    );

    struct Test3;
    define_reference_type!(
        type = Test3,
        class = "java.lang.Object",
        load_class = |env, loader_context, initialize| {
            loader_context.load_class_for_type::<Test3>(initialize, env)
        },
        raw = jstring,
    );

    struct Test4;
    define_reference_type!(
        type = Test4,
        class = "java.lang.Object",
        raw = jstring,
    );

    struct Test5;
    define_reference_type!(
        type = Test5,
        class = "java.lang.Object",
        raw = jstring,
        as = [Test2, Test3],
    );

    struct Test6;
    define_reference_type!(
        type = Test6,
        class = "java.lang.Object",
        raw = jstring,
        as = [Test2, Test3],
        methods = {
            get_message = {
                name = "getMessage",
                sig = () -> java.lang.String as JString,
            },
            set_message = {
                name = "setMessage",
                sig = (msg: java.lang.String) -> void,
            }
        },
        static_methods = {
            example_static = {
                name = "exampleStatic",
                sig = (val: jint) -> jint,
            }
        },
        fields {
            example_field = {
                name = "exampleField",
                sig = jint,
            }
        },
        static_fields {
            example_static_field = {
                name = "exampleStaticField",
                sig = jint,
            }
        }
    );

    struct Test7;
    define_reference_type!(
        type = Test7,
        class = "java.lang.Object",
        raw = jstring,
        as = [Test2, Test3],
        fields {
            example_field = {
                name = "exampleField",
                sig = jint,
            }
        },
        methods = {
            get_message = {
                name = "getMessage",
                sig = () -> java.lang.String as JString,
            },
            set_message = {
                name = "setMessage",
                sig = (msg: java.lang.String) -> void,
            }
        },
        static_methods = {
            example_static = {
                name = "exampleStatic",
                sig = (val: jint) -> jint,
            }
        },
    );

    /*
    #[allow(dead_code)]
    struct JThrowableAPI {
        class: String,
        get_message_method: String,
        get_cause_method: String,
        get_stack_trace_method: String,
    }
    struct JThrowable;
    define_reference_type!(
        type = JThrowable,
        class = "java.lang.Throwable",
        methods {
            get_message_method = { name = "getMessage", sig = () -> java.lang.String },
            get_cause_method = { name = "getCause", sig = () -> java.lang.Throwable },
            get_stack_trace_method = { name = "getStackTrace", sig = () -> java.lang.StackTraceElement },
        }
    );
    */

    struct Test8;
    define_reference_type!(
        type = Test8,
        class = "java.lang.Object",
        raw = jstring,
        load_class = |env, _loader_context, _initialize| {
            env.find_class("java/lang/Object")
        }
    );

    Test6API::get(env, &LoaderContext::None);
    Test7API::get(env, &LoaderContext::None);
    Test8API::get(env, &LoaderContext::None);

    println!("All macro parsing tests passed!");
}
