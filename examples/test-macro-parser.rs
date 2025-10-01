/// Standalone test for define_reference_type! macro parsing
use std::ops::Deref;

#[derive(Default)]
struct Env;
#[allow(unused)]
impl Env {
    fn new_global_ref(&self, class: &JClass) -> Result<String, String> {
        Ok(format!("global_ref::{}", class.name()))
    }
    fn get_method_id(&self, class: &JClass, _name: &str, _sig: &str) -> Result<String, String> {
        Ok(format!("method_id::{}", class.name()))
    }
}

#[derive(Default)]
struct LoaderContext;
impl LoaderContext {
    fn load_class_for_type<T>(&self, _initialize: bool, _env: &mut Env) -> Result<JClass, String> {
        Ok(JClass::null())
    }
}

#[derive(Clone)]
struct JClass(&'static str);

impl JClass {
    fn new(name: &'static str) -> Self {
        Self(name)
    }

    fn name(&self) -> &'static str {
        self.0
    }
    fn null() -> Self {
        Self("<null>")
    }
}

impl Deref for JClass {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __drt__expand_init {
    ($Type:ident, $env:expr, $loader:expr, __drt__InitKindEnvClass, $Init:expr) => {{
        let class = $loader.load_class_for_type::<$Type>(true, $env)?;
        Self::call_init($env, &class, $Init)
    }};
    ($Type:ident, $env:expr, $loader:expr, __drt__InitKindLoader, $Init:expr) => {
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
            // Minimal demo: provide a get() that prints diagnostics
            impl $ApiTy {
                #[allow(unused)]
                fn call_init<F, R>(env: &mut $crate::Env, class: &JClass, init: F) -> Result<R, String>
                where
                    F: FnOnce(&mut $crate::Env, &JClass) -> Result<R, String>,
                {
                    init(env, class)
                }

                #[allow(unused)]
                fn call_init_with_loader<F, R>(env: &mut $crate::Env, loader: &LoaderContext, init: F) -> Result<R, String>
                where
                    F: FnOnce(&mut $crate::Env, &LoaderContext) -> Result<R, String>,
                {
                    init(env, loader)
                }

                pub fn get() -> Result<&'static Self, String> {
                    println!("Generated API for type: {}", stringify!($Type));
                    println!("  class: {}", $Class);
                    println!("  raw: {}", stringify!($RawTy));
                    println!("  init kind: {}", stringify!($init_kind));
                    println!("  aliases: {}", stringify!([$($Aliases)*]));
                    println!("  methods: {}", stringify!({$($Methods)*}));
                    println!("  static_methods: {}", stringify!({$($StaticMethods)*}));
                    println!("  fields: {}", stringify!({$($Fields)*}));
                    println!("  static_fields: {}", stringify!({$($StaticFields)*}));
                    static CELL: once_cell::sync::OnceCell<$ApiTy> = once_cell::sync::OnceCell::new();
                    CELL.get_or_try_init(|| {
                        let mut env = Env::default();
                        let loader = LoaderContext::default();
                        $crate::__drt__expand_init!($Type, &mut env, &loader, $InitKind, $Init)
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

// Determine which init variant is provided and tail-call with the decision
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

    // Parse tokens: init = <expr>, with comma - wrap with __drt_Closure to avoid comma ambiguity
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } init = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (init, (__drt_Closure(($value))))] } $($rest)* }
    };
    // Parse tokens: init = <expr>, no comma (end)
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } init = $value:expr) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (init, (__drt_Closure(($value))))] } }
    };

    // Parse tokens: init_with_loader = <expr>, with comma
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } init_with_loader = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (init_with_loader, (__drt_Closure(($value))))] } $($rest)* }
    };
    // Parse tokens: init_with_loader = <expr>, no comma (end)
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } init_with_loader = $value:expr) => {
        $crate::__def_ref_parse! { @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (init_with_loader, (__drt_Closure(($value))))] } }
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

// Finalizer chain: extract required, then optional, then emit
#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_finalize {
    ([$($pairs:tt)*]) => {
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

fn main() {
    struct Test0API;
    struct Test0;
    define_reference_type!(
        type = Test0,
        class = "java.lang.Object",
        raw = jstring,
        init = |_env, _class| {
            println!("Test0: Custom init called");
            Ok(Test0API)
        }
    );
    let _ = Test0API::get();

    struct Custom0API;
    struct Test1;
    define_reference_type!(
        type = Test1,
        class = "java.lang.Object",
        init = |_env, _class| {
            println!("Custom init called");
            Ok(Custom0API)
        },
        raw = jstring,
        api = Custom0API
    );

    struct Custom1API;
    struct Test2;
    define_reference_type!(
        type = Test2,
        class = "java.lang.Object",
        api = Custom1API,
        init = |_env, _class| {
            println!("Custom init called");
            Ok(Custom1API)
        },
        raw = jstring,
    );

    struct Test3API;
    struct Test3;
    define_reference_type!(
        type = Test3,
        class = "java.lang.Object",
        init = |_env, _class| {
            println!("Custom init called");
            Ok(Test3API)
        },
        raw = jstring,
    );

    struct Test4API;
    struct Test4;
    define_reference_type!(
        type = Test4,
        class = "java.lang.Object",
        raw = jstring,
        init = |_env, _class| {
            println!("Custom init called");
            Ok(Test4API)
        }
    );

    struct Test5API;
    struct Test5;
    define_reference_type!(
        type = Test5,
        class = "java.lang.Object",
        raw = jstring,
        init = |_env, _class| {
            println!("Custom init called");
            Ok(Test5API)
        },
        as = [Test2, Test3],
    );

    struct Test6API;
    struct Test6;
    define_reference_type!(
        type = Test6,
        class = "java.lang.Object",
        raw = jstring,
        init = |_env, _class| {
            println!("Custom init called");
            Ok(Test6API)
        },
        as = [Test2, Test3],
        methods = {
            get_message = {
                name = "getMessage",
                sig = "()Ljava/lang/String;",
                ret = JString,
            },
            set_message = {
                name = "setMessage",
                sig = "(Ljava/lang/String;)V",
                ret = void
            }
        },
        static_methods = {
            example_static = {
                name = "exampleStatic",
                sig = "(I)I",
                ret = jint,
            }
        },
        fields {
            example_field = {
                name = "exampleField",
                sig = "I",
                ty = jint
            }
        },
        static_fields {
            example_static_field = {
                name = "exampleStaticField",
                sig = "I",
                ty = jint,
            }
        }
    );

    struct Test7API;
    struct Test7;
    define_reference_type!(
        type = Test7,
        class = "java.lang.Object",
        raw = jstring,
        init = |_env, _class| {
            println!("Test7: Custom init called");
            Ok(Test7API)
        },
        as = [Test2, Test3],
        fields {
            example_field = {
                name = "exampleField",
                sig = "I",
                ty = jint,
            }
        },
        methods = {
            get_message = {
                name = "getMessage",
                sig = () -> java.lang.String,
                ret = JString
            },
            set_message = {
                name = "setMessage",
                sig = "(Ljava/lang/String;)V",
                ret = void,
            }
        },
        static_methods = {
            example_static = {
                name = "exampleStatic",
                sig = "(I)I",
                ret = jint,
            }
        },
    );

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
        init = |env, class| {
            println!("JThrowable: Custom init called");
            Ok(JThrowableAPI {
                class: env.new_global_ref(class)?,
                get_message_method: env.get_method_id(class, "getMessage", "()Ljava/lang/String;")?,
                get_cause_method: env.get_method_id(class, "getCause", "()Ljava/lang/Throwable;")?,
                get_stack_trace_method: env.get_method_id(class, "getStackTrace", "()[Ljava/lang/StackTraceElement;")?,
            })
        }
    );

    struct Test8API;
    struct Test8;
    define_reference_type!(
        type = Test8,
        class = "java.lang.Object",
        raw = jstring,
        init_with_loader = |env, loader_context| {
            let _class = loader_context.load_class_for_type::<Test8>(false, env).unwrap();
            println!("Test8: Custom init called");
            Ok(Test8API)
        }
    );

    Test6API::get();
    Test7API::get();
    Test8API::get();

    println!("All macro parsing tests passed!");
}
