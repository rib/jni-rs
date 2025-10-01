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
    fn load_class_for_type(&self, _env: &mut Env) -> Result<String, String> {
        Ok("loaded_class".to_string())
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
}

impl Deref for JClass {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

fn call_init<F, R>(init: F, env: &mut Env, class: &JClass) -> Result<R, String>
where
    F: FnOnce(&mut Env, &JClass) -> Result<R, String>,
{
    init(env, class)
}

fn call_init_with_loader<F, R>(init: F, env: &mut Env, loader: &LoaderContext) -> Result<R, String>
where
    F: FnOnce(&mut Env, &LoaderContext) -> Result<R, String>,
{
    init(env, loader)
}

// Dummy name mappings for testing that avoids paste crate dependency
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__with_api_ident {
    (__auto_api, Test0, $callback:ident $(, $args:tt)*) => {
        $callback!(Test0API $(, $args)*)
    };
    (__auto_api, Test1, $callback:ident $(, $args:tt)*) => {
        $callback!(Test1API $(, $args)*)
    };
    (__auto_api, Test2, $callback:ident $(, $args:tt)*) => {
        $callback!(Test2API $(, $args)*)
    };
    (__auto_api, Test3, $callback:ident $(, $args:tt)*) => {
        $callback!(Test3API $(, $args)*)
    };
    (__auto_api, Test4, $callback:ident $(, $args:tt)*) => {
        $callback!(Test4API $(, $args)*)
    };
    (__auto_api, Test5, $callback:ident $(, $args:tt)*) => {
        $callback!(Test5API $(, $args)*)
    };
    (__auto_api, Test6, $callback:ident $(, $args:tt)*) => {
        $callback!(Test6API $(, $args)*)
    };
    (__auto_api, Test7, $callback:ident $(, $args:tt)*) => {
        $callback!(Test7API $(, $args)*)
    };
    (__auto_api, Test8, $callback:ident $(, $args:tt)*) => {
        $callback!(Test8API $(, $args)*)
    };
    (__auto_api, JThrowable, $callback:ident $(, $args:tt)*) => {
        $callback!(JThrowableAPI $(, $args)*)
    };
    (Custom0API, $Type:ident, $callback:ident $(, $args:tt)*) => {
        $callback!(Custom0API $(, $args)*)
    };
    (Custom1API, $Type:ident, $callback:ident $(, $args:tt)*) => {
        $callback!(Custom1API $(, $args)*)
    };
    ($Api:ident, $Type:ident, $callback:ident $(, $args:tt)*) => {
        $callback!($Api $(, $args)*)
    };
}

// Decide which init variant is provided and tail-call with the decision
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__dispatch_init {
    (init = [()], init_with_loader = [()], $callback:ident, $($rest:tt)*) => {
        compile_error!("define_reference_type!: exactly one of `init` or `init_with_loader` must be provided")
    };
    (init = [$($Init:tt)+], init_with_loader = [()], $callback:ident, $($rest:tt)*) => {
        $callback!(init, [$($Init)+], $($rest)*)
    };
    (init = [()], init_with_loader = [$($InitWithLoader:tt)+], $callback:ident, $($rest:tt)*) => {
        $callback!(init_with_loader, [$($InitWithLoader)+], $($rest)*)
    };
    (init = [$($I:tt)+], init_with_loader = [$($J:tt)+], $callback:ident, $($rest:tt)*) => {
        compile_error!("define_reference_type!: `init` and `init_with_loader` are mutually exclusive")
    };
}

// Final emission with concrete API ident and init decision
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_api_with_kind {
    ($init_kind:ident, [$($InitTokens:tt)*], $Api:ident, type = $Type:ident, class = $Class:expr, raw = $Raw:ident,
     aliases = [$($Aliases:tt)*], methods = {$($Methods:tt)*}, static_methods = {$($StaticMethods:tt)*}, fields = {$($Fields:tt)*}, static_fields = {$($StaticFields:tt)*}) => {
        // Minimal demo: provide a get() that prints diagnostics
        impl $Api {
            pub fn get() {
                println!("Generated API for type: {}", stringify!($Type));
                println!("  class: {}", $Class);
                println!("  raw: {}", stringify!($Raw));
                println!("  init kind: {}", stringify!($init_kind));
                let _ = stringify!($($InitTokens)*);
                println!("  aliases: {}", stringify!([$($Aliases)*]));
                println!("  methods: {}", stringify!({$($Methods)*}));
                println!("  static_methods: {}", stringify!({$($StaticMethods)*}));
                println!("  fields: {}", stringify!({$($Fields)*}));
                println!("  static_fields: {}", stringify!({$($StaticFields)*}));
            }
        }
    };
}

// Resolve API ident then dispatch on init/init_with_loader
#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_with_api {
    ($Api:ident, ($($rest:tt)*)) => {
        $crate::__drt__emit_with_api_parse!($Api, $($rest)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __drt__emit_with_api_parse {
    ($Api:ident, init = [$Init:tt], init_with_loader = [$InitWithLoader:tt], type = $Type:ident, class = $Class:expr, raw = $Raw:ident,
     aliases = [$($Aliases:tt)*], methods = {$($Methods:tt)*}, static_methods = {$($StaticMethods:tt)*}, fields = {$($Fields:tt)*}, static_fields = {$($StaticFields:tt)*}) => {
        $crate::__drt__dispatch_init!(
            init = [$Init],
            init_with_loader = [$InitWithLoader],
            __drt__emit_api_with_kind,
            $Api,
            type = $Type, class = $Class, raw = $Raw,
            aliases = [$($Aliases)*], methods = {$($Methods)*}, static_methods = {$($StaticMethods)*}, fields = {$($Fields)*}, static_fields = {$($StaticFields)*}
        );
    };
}

// Entry to emission: normalize API ident and forward to emitter
#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_emit {
    (
        type   = $Type:ident,
        class  = $Class:expr,
        raw    = $Raw:ident,
        api    = $Api:ident,
        init   = $Init:tt,
        init_with_loader = $InitWithLoader:tt,
        aliases = [$($Aliases:tt)*],
        methods = {$($Methods:tt)*},
        static_methods = {$($StaticMethods:tt)*},
        fields = {$($Fields:tt)*},
        static_fields = {$($StaticFields:tt)*},
    ) => {
        $crate::__drt__with_api_ident!(
            $Api, $Type, __drt__emit_with_api,
            (
                init = [$Init], init_with_loader = [$InitWithLoader],
                type = $Type, class = $Class, raw = $Raw,
                aliases = [$($Aliases)*], methods = {$($Methods)*}, static_methods = {$($StaticMethods)*}, fields = {$($Fields)*}, static_fields = {$($StaticFields)*}
            )
        );
    };
}

// Defaults + lookup approach (order-agnostic, append-only)
#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_append_defaults {
    ([$($pairs:tt)*]) => {
        [
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
        ]
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_type {
    ([], $found:ident, $not:ident $(, $args:tt)*) => { $not!($($args)*) };
    ([(type, $Type:ident) $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $found!($Type $(, $args)*)
    };
    ([$_head:tt $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $crate::__def_ref_lookup_type!([$($rest)*], $found, $not $(, $args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_class {
    ([], $found:ident, $not:ident $(, $args:tt)*) => { $not!($($args)*) };
    ([(class, $Class:expr) $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $found!($Class $(, $args)*)
    };
    ([$_head:tt $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $crate::__def_ref_lookup_class!([$($rest)*], $found, $not $(, $args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_raw {
    ([], $found:ident, $not:ident $(, $args:tt)*) => { $not!($($args)*) };
    ([(raw, $Raw:ident) $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $found!($Raw $(, $args)*)
    };
    ([$_head:tt $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $crate::__def_ref_lookup_raw!([$($rest)*], $found, $not $(, $args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_api {
    ([], $found:ident, $not:ident $(, $args:tt)*) => { $not!($($args)*) };
    ([(api, $Api:ident) $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $found!($Api $(, $args)*)
    };
    ([$_head:tt $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $crate::__def_ref_lookup_api!([$($rest)*], $found, $not $(, $args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_init {
    ([], $found:ident, $not:ident $(, $args:tt)*) => { $not!($($args)*) };
    ([(init, (__drt_Closure(($Init:tt)))) $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $found!($Init $(, $args)*)
    };
    ([(init, (__drt_Closure($Init:tt))) $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $found!($Init $(, $args)*)
    };
    ([$_head:tt $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $crate::__def_ref_lookup_init!([$($rest)*], $found, $not $(, $args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_init_with_loader {
    ([], $found:ident, $not:ident $(, $args:tt)*) => { $not!($($args)*) };
    ([(init_with_loader, (__drt_Closure(($Init:tt)))) $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $found!($Init $(, $args)*)
    };
    ([(init_with_loader, (__drt_Closure($Init:tt))) $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $found!($Init $(, $args)*)
    };
    ([$_head:tt $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $crate::__def_ref_lookup_init_with_loader!([$($rest)*], $found, $not $(, $args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_aliases {
    ([], $found:ident, $not:ident $(, $args:tt)*) => { $not!($($args)*) };
    ([(aliases, [$($Aliases:tt)*]) $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $found!([$($Aliases)*] $(, $args)*)
    };
    ([$_head:tt $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $crate::__def_ref_lookup_aliases!([$($rest)*], $found, $not $(, $args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_methods {
    ([], $found:ident, $not:ident $(, $args:tt)*) => { $not!($($args)*) };
    ([(methods, {$($Methods:tt)*}) $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $found!({$($Methods)*} $(, $args)*)
    };
    ([$_head:tt $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $crate::__def_ref_lookup_methods!([$($rest)*], $found, $not $(, $args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_static_methods {
    ([], $found:ident, $not:ident $(, $args:tt)*) => { $not!($($args)*) };
    ([(static_methods, {$($StaticMethods:tt)*}) $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $found!({$($StaticMethods)*} $(, $args)*)
    };
    ([$_head:tt $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $crate::__def_ref_lookup_static_methods!([$($rest)*], $found, $not $(, $args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_fields {
    ([], $found:ident, $not:ident $(, $args:tt)*) => { $not!($($args)*) };
    ([(fields, {$($Fields:tt)*}) $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $found!({$($Fields)*} $(, $args)*)
    };
    ([$_head:tt $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $crate::__def_ref_lookup_fields!([$($rest)*], $found, $not $(, $args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_lookup_static_fields {
    ([], $found:ident, $not:ident $(, $args:tt)*) => { $not!($($args)*) };
    ([(static_fields, {$($StaticFields:tt)*}) $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $found!({$($StaticFields)*} $(, $args)*)
    };
    ([$_head:tt $($rest:tt)*], $found:ident, $not:ident $(, $args:tt)*) => {
        $crate::__def_ref_lookup_static_fields!([$($rest)*], $found, $not $(, $args)*)
    };
}

// Finalizer chain: extract required, then optional, then emit
#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_finalize {
    ([$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_type!(
            [$($pairs)*],
            __def_ref_found_type,
            __def_ref_missing_type,
            [$($pairs)*]
        );
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
        $crate::__def_ref_lookup_class!(
            [$($pairs)*],
            __def_ref_found_class,
            __def_ref_missing_class,
            $Type, [$($pairs)*]
        );
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
        $crate::__def_ref_lookup_raw!(
            [$($pairs)*],
            __def_ref_found_raw,
            __def_ref_unreachable,
            $Type, $Class, [$($pairs)*]
        );
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
        $crate::__def_ref_lookup_api!(
            [$($pairs)*],
            __def_ref_found_api,
            __def_ref_unreachable,
            $Type, $Class, $Raw, [$($pairs)*]
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_api {
    ($Api:ident, $Type:ident, $Class:expr, $Raw:ident, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_init!(
            [$($pairs)*],
            __def_ref_found_init,
            __def_ref_unreachable,
            $Type, $Class, $Raw, $Api, [$($pairs)*]
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_init {
    ($Init:tt, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_init_with_loader!(
            [$($pairs)*],
            __def_ref_found_init_with_loader,
            __def_ref_unreachable,
            $Type, $Class, $Raw, $Api, $Init, [$($pairs)*]
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_init_with_loader {
    ($InitWithLoader:tt, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $Init:tt, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_aliases!(
            [$($pairs)*],
            __def_ref_found_aliases,
            __def_ref_unreachable,
            $Type, $Class, $Raw, $Api, $Init, $InitWithLoader, [$($pairs)*]
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_aliases {
    ([$($Aliases:tt)*], $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $Init:tt, $InitWithLoader:tt, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_methods!(
            [$($pairs)*],
            __def_ref_found_methods,
            __def_ref_unreachable,
            $Type, $Class, $Raw, $Api, $Init, $InitWithLoader, [$($Aliases)*], [$($pairs)*]
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_methods {
    ({$($Methods:tt)*}, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $Init:tt, $InitWithLoader:tt, [$($Aliases:tt)*], [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_static_methods!(
            [$($pairs)*],
            __def_ref_found_static_methods,
            __def_ref_unreachable,
            $Type, $Class, $Raw, $Api, $Init, $InitWithLoader, [$($Aliases)*], {$($Methods)*}, [$($pairs)*]
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_static_methods {
    ({$($StaticMethods:tt)*}, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $Init:tt, $InitWithLoader:tt, [$($Aliases:tt)*], {$($Methods:tt)*}, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_fields!(
            [$($pairs)*],
            __def_ref_found_fields,
            __def_ref_unreachable,
            $Type, $Class, $Raw, $Api, $Init, $InitWithLoader, [$($Aliases)*], {$($Methods)*}, {$($StaticMethods)*}, [$($pairs)*]
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_found_fields {
    ({$($Fields:tt)*}, $Type:ident, $Class:expr, $Raw:ident, $Api:ident, $Init:tt, $InitWithLoader:tt, [$($Aliases:tt)*], {$($Methods:tt)*}, {$($StaticMethods:tt)*}, [$($pairs:tt)*]) => {
        $crate::__def_ref_lookup_static_fields!(
            [$($pairs)*],
            __def_ref_found_static_fields,
            __def_ref_unreachable,
            $Type, $Class, $Raw, $Api, $Init, $InitWithLoader, [$($Aliases)*], {$($Methods)*}, {$($StaticMethods)*}, {$($Fields)*}
        );
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
            init   = $Init,
            init_with_loader = $InitWithLoader,
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
macro_rules! __def_ref_parse {
    // Done: emit with resolved required fields and accumulated option pairs
    (@finish type = ($Type:ident), class = ($Class:expr), pairs = [$($pairs:tt)*]) => {{
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
    }};
    // Error cases
    (@finish type = (), class = ($Class:expr), pairs = [$($pairs:tt)*]) => {
        compile_error!("define_reference_type!: missing required `type` field")
    };
    (@finish type = ($Type:ident), class = (), pairs = [$($pairs:tt)*]) => {
        compile_error!("define_reference_type!: missing required `class` field")
    };
    (@finish type = (), class = (), pairs = [$($pairs:tt)*]) => {
        compile_error!("define_reference_type!: missing required `type` and `class` fields")
    };

    // Finished parsing tokens, move to extraction
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] }) => {
        $crate::__def_ref_parse! {
            @finish type = $Type, class = $Class, pairs = [$($acc)*]
        }
    };

    // Parse tokens: type = <ident>
    (@parse_tokens { type = (), class = $Class:tt, pairs = [$($acc:tt)*] } type = $value:ident $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = ($value), class = $Class, pairs = [$($acc)*] } $($rest)*
        }
    };
    // If type already set, updating it is an error to keep semantics simple
    (@parse_tokens { type = ($set:ident), class = $Class:tt, pairs = [$($acc:tt)*] } type = $value:ident $($rest:tt)*) => {
        compile_error!("define_reference_type!: duplicate `type` key")
    };

    // Parse tokens: class = <expr>, with comma
    (@parse_tokens { type = $Type:tt, class = (), pairs = [$($acc:tt)*] } class = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = $Type, class = ($value), pairs = [$($acc)*] } $($rest)*
        }
    };
    // Parse tokens: class = <expr>, no comma (end)
    (@parse_tokens { type = $Type:tt, class = (), pairs = [$($acc:tt)*] } class = $value:expr) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = $Type, class = ($value), pairs = [$($acc)*] }
        }
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
        $crate::__def_ref_parse! {
            @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (raw, $value)] } $($rest)*
        }
    };

    // Parse tokens: api = <ident>
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } api = $value:ident $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (api, $value)] } $($rest)*
        }
    };

    // Parse tokens: init = <expr>, with comma - wrap with __drt_Closure to avoid comma ambiguity
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } init = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (init, (__drt_Closure(($value))))] } $($rest)*
        }
    };
    // Parse tokens: init = <expr>, no comma (end)
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } init = $value:expr) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (init, (__drt_Closure(($value))))] }
        }
    };

    // Parse tokens: init_with_loader = <expr>, with comma
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } init_with_loader = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (init_with_loader, (__drt_Closure(($value))))] } $($rest)*
        }
    };
    // Parse tokens: init_with_loader = <expr>, no comma (end)
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } init_with_loader = $value:expr) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (init_with_loader, (__drt_Closure(($value))))] }
        }
    };

    // Parse tokens: as = [aliases]
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } as = [$($value:tt)*] $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (aliases, [$($value)*])] } $($rest)*
        }
    };

    // Parse tokens: methods = {methods}
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } methods = {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (methods, {$($value)*})] } $($rest)*
        }
    };

    // Parse tokens: static_methods = {static_methods}
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } static_methods = {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (static_methods, {$($value)*})] } $($rest)*
        }
    };

    // Parse tokens: fields {fields}
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } fields {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (fields, {$($value)*})] } $($rest)*
        }
    };

    // Parse tokens: static_fields {static_fields}
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } static_fields {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)* (static_fields, {$($value)*})] } $($rest)*
        }
    };

    // Parse tokens: Skip commas
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } , $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens { type = $Type, class = $Class, pairs = [$($acc)*] } $($rest)*
        }
    };

    // Parse tokens: Error on unexpected tokens
    (@parse_tokens { type = $Type:tt, class = $Class:tt, pairs = [$($acc:tt)*] } $bad:tt $($rest:tt)*) => {
        compile_error!(concat!("Unexpected token in define_reference_type: ", stringify!($bad)));
    };

    // Append type = <ident>
    (@parse [$($acc:tt)*] type = $value:ident $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (type, $value)] $($rest)*
        }
    };

    // Append class = <expr>, with comma
    (@parse [$($acc:tt)*] class = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (class, $value)] $($rest)*
        }
    };

    // Append class = <expr>, no comma (end)
    (@parse [$($acc:tt)*] class = $value:expr) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (class, $value)]
        }
    };

    // Append raw = <ident>
    (@parse [$($acc:tt)*] raw = $value:ident $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (raw, $value)] $($rest)*
        }
    };

    // Append api = <ident>
    (@parse [$($acc:tt)*] api = $value:ident $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (api, $value)] $($rest)*
        }
    };

    // Append init = <expr>, with comma - wrap with __drt_Closure to avoid comma ambiguity
    (@parse [$($acc:tt)*] init = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (init, (__drt_Closure(($value))))] $($rest)*
        }
    };

    // Append init = <expr>, no comma (end) - wrap with __drt_Closure to avoid comma ambiguity
    (@parse [$($acc:tt)*] init = $value:expr) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (init, (__drt_Closure(($value))))]
        }
    };

    // Append init_with_loader = <expr>, with comma - wrap with __drt_Closure to avoid comma ambiguity
    (@parse [$($acc:tt)*] init_with_loader = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (init_with_loader, (__drt_Closure(($value))))] $($rest)*
        }
    };

    // Append init_with_loader = <expr>, no comma (end) - wrap with __drt_Closure to avoid comma ambiguity
    (@parse [$($acc:tt)*] init_with_loader = $value:expr) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (init_with_loader, (__drt_Closure(($value))))]
        }
    };

    // Append as = [aliases]
    (@parse [$($acc:tt)*] as = [$($value:tt)*] $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (aliases, [$($value)*])] $($rest)*
        }
    };

    // Append methods = {methods}
    (@parse [$($acc:tt)*] methods = {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (methods, {$($value)*})] $($rest)*
        }
    };

    // Append static_methods = {static_methods}
    (@parse [$($acc:tt)*] static_methods = {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (static_methods, {$($value)*})] $($rest)*
        }
    };

    // Append fields {fields}
    (@parse [$($acc:tt)*] fields {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (fields, {$($value)*})] $($rest)*
        }
    };

    // Append static_fields {static_fields}
    (@parse [$($acc:tt)*] static_fields {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (static_fields, {$($value)*})] $($rest)*
        }
    };

    // Skip commas
    (@parse [$($acc:tt)*] , $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)*] $($rest)*
        }
    };

    // Error on unexpected tokens
    (@parse [$($acc:tt)*] $bad:tt $($rest:tt)*) => {
        compile_error!(concat!("Unexpected token in define_reference_type: ", stringify!($bad)));
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
    define_reference_type!(
        type = Test8,
        class = "java.lang.Object",
        raw = jstring,
        init_with_loader = |env, loader_context| {
            let _class = loader_context.load_class_for_type(env).unwrap();
            println!("Test8: Custom init called");
            Ok(Test8API)
        }
    );

    Test6API::get();
    Test7API::get();
    Test8API::get();

    println!("All macro parsing tests passed!");
}
