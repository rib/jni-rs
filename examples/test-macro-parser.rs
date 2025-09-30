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

// Simplified emit macro that receives individual tokens (not named parameters)
macro_rules! __drt__init_kind_report {
    (__drt__InitKindEnvClass) => {
        println!("Init kind: init");
    };
    (__drt__InitKindLoader) => {
        println!("Init kind: init_with_loader");
    };
}

macro_rules! __drt__call_init_kind {
    (__drt__InitKindEnvClass, $Init:expr, $env:ident, $Class:expr) => {{
        let class = JClass::new($Class);
        call_init($Init, &mut $env, &class)
    }};
    (__drt__InitKindLoader, $Init:expr, $env:ident, $Class:expr) => {{
        let loader = LoaderContext::default();
        call_init_with_loader($Init, &mut $env, &loader)
    }};
}

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
        impl $ApiTy {
            #[allow(unused)]
            fn get() -> Self {
                println!("Generated API for type: {}", stringify!($Type));
                println!("Class: {}", $Class);
                println!("Raw type: {}", stringify!($crate::sys::$RawTy));
                println!("Init tokens: {}", stringify!($Init));
                println!("Aliases: [{}]", stringify!($($Aliases)*));
                println!("Methods: {{ {} }}", stringify!($($Methods)*));
                println!("Static methods: {{ {} }}", stringify!($($StaticMethods)*));
                println!("Fields: {{ {} }}", stringify!($($Fields)*));
                println!("Static fields: {{ {} }}", stringify!($($StaticFields)*));

                let mut env = Env::default();
                __drt__init_kind_report!($InitKind);
                __drt__call_init_kind!($InitKind, $Init, env, $Class).unwrap()
            }
        }
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
macro_rules! __def_ref_extract {
    // Base case: no more pairs to process, emit with current values
    ([] { $($result:tt)* }) => {
        $crate::__def_ref_emit! { $($result)* }
    };

    // Extract type
    ([(type, $value:ident)$($rest:tt)*] {
        type = $old:ident, $($other:tt)*
    }) => {
        $crate::__def_ref_extract! {
            [$($rest)*]
            { type = $value, $($other)* }
        }
    };

    // Extract class
    ([(class, $value:expr)$($rest:tt)*] {
        class = $old:expr, $($other:tt)*
    }) => {
        $crate::__def_ref_extract! {
            [$($rest)*]
            { class = $value, $($other)* }
        }
    };

    // Extract raw
    ([(raw, $value:ident)$($rest:tt)*] {
        raw = $old:ident, $($other:tt)*
    }) => {
        $crate::__def_ref_extract! {
            [$($rest)*]
            { raw = $value, $($other)* }
        }
    };

    // Extract api
    ([(api, $value:ident)$($rest:tt)*] {
        api = $old:ident, $($other:tt)*
    }) => {
        $crate::__def_ref_extract! {
            [$($rest)*]
            { api = $value, $($other)* }
        }
    };

    // Extract init (already wrapped with __drt_Closure)
    ([(init, $value:tt)$($rest:tt)*] {
        init = __drt_Closure($old:tt), $($other:tt)*
    }) => {
        $crate::__def_ref_extract! {
            [$($rest)*]
            { init = $value, $($other)* }
        }
    };

    // Extract init_with_loader (already wrapped with __drt_Closure)
    ([(init_with_loader, $value:tt)$($rest:tt)*] {
        init_with_loader = __drt_Closure($old:tt), $($other:tt)*
    }) => {
        $crate::__def_ref_extract! {
            [$($rest)*]
            { init_with_loader = $value, $($other)* }
        }
    };

    // Extract aliases
    ([(aliases, [$($value:tt)*])$($rest:tt)*] {
        aliases = [$($old:tt)*], $($other:tt)*
    }) => {
        $crate::__def_ref_extract! {
            [$($rest)*]
            { aliases = [$($value)*], $($other)* }
        }
    };

    // Extract methods
    ([(methods, {$($value:tt)*})$($rest:tt)*] {
        methods = {$($old:tt)*}, $($other:tt)*
    }) => {
        $crate::__def_ref_extract! {
            [$($rest)*]
            { methods = {$($value)*}, $($other)* }
        }
    };

    // Extract static_methods
    ([(static_methods, {$($value:tt)*})$($rest:tt)*] {
        static_methods = {$($old:tt)*}, $($other:tt)*
    }) => {
        $crate::__def_ref_extract! {
            [$($rest)*]
            { static_methods = {$($value)*}, $($other)* }
        }
    };

    // Extract fields
    ([(fields, {$($value:tt)*})$($rest:tt)*] {
        fields = {$($old:tt)*}, $($other:tt)*
    }) => {
        $crate::__def_ref_extract! {
            [$($rest)*]
            { fields = {$($value)*}, $($other)* }
        }
    };

    // Extract static_fields
    ([(static_fields, {$($value:tt)*})$($rest:tt)*] {
        static_fields = {$($old:tt)*}, $($other:tt)*
    }) => {
        $crate::__def_ref_extract! {
            [$($rest)*]
            { static_fields = {$($value)*}, $($other)* }
        }
    };

    // Skip unknown keys
    ([($key:tt, $value:tt)$($rest:tt)*] { $($result:tt)* }) => {
        $crate::__def_ref_extract! {
            [$($rest)*]
            { $($result)* }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __def_ref_parse {
    // Done: extract final values and emit  
    (@parse [$($pairs:tt)*]) => {
        $crate::__def_ref_extract! {
            [$($pairs)*]
            {
                type = __PLACEHOLDER__,
                class = "__PLACEHOLDER__",
                raw = jobject,
                api = __auto_api,
                init = __drt_Closure(()),
                init_with_loader = __drt_Closure(()),
                aliases = [],
                methods = {},
                static_methods = {},
                fields = {},
                static_fields = {},
            }
        }
    };



    // Start parsing with empty accumulator
    (@parse_tokens [$($acc:tt)*]) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)*]
        }
    };

    // Parse tokens: type = <ident>
    (@parse_tokens [$($acc:tt)*] type = $value:ident $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)* (type, $value)] $($rest)*
        }
    };

    // Parse tokens: class = <expr>, with comma
    (@parse_tokens [$($acc:tt)*] class = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)* (class, $value)] $($rest)*
        }
    };

    // Parse tokens: class = <expr>, no comma (end)
    (@parse_tokens [$($acc:tt)*] class = $value:expr) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)* (class, $value)]
        }
    };

    // Parse tokens: raw = <ident>
    (@parse_tokens [$($acc:tt)*] raw = $value:ident $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)* (raw, $value)] $($rest)*
        }
    };

    // Parse tokens: api = <ident>
    (@parse_tokens [$($acc:tt)*] api = $value:ident $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)* (api, $value)] $($rest)*
        }
    };

    // Parse tokens: init = <expr>, with comma - wrap with __drt_Closure to avoid comma ambiguity
    (@parse_tokens [$($acc:tt)*] init = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)* (init, __drt_Closure(($value)))] $($rest)*
        }
    };

    // Parse tokens: init = <expr>, no comma (end) - wrap with __drt_Closure to avoid comma ambiguity
    (@parse_tokens [$($acc:tt)*] init = $value:expr) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)* (init, __drt_Closure(($value)))]
        }
    };

    // Parse tokens: init_with_loader = <expr>, with comma - wrap with __drt_Closure to avoid comma ambiguity
    (@parse_tokens [$($acc:tt)*] init_with_loader = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)* (init_with_loader, __drt_Closure(($value)))] $($rest)*
        }
    };

    // Parse tokens: init_with_loader = <expr>, no comma (end) - wrap with __drt_Closure to avoid comma ambiguity
    (@parse_tokens [$($acc:tt)*] init_with_loader = $value:expr) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)* (init_with_loader, __drt_Closure(($value)))]
        }
    };

    // Parse tokens: as = [aliases]
    (@parse_tokens [$($acc:tt)*] as = [$($value:tt)*] $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)* (aliases, [$($value)*])] $($rest)*
        }
    };

    // Parse tokens: methods = {methods}
    (@parse_tokens [$($acc:tt)*] methods = {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)* (methods, {$($value)*})] $($rest)*
        }
    };

    // Parse tokens: static_methods = {static_methods}
    (@parse_tokens [$($acc:tt)*] static_methods = {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)* (static_methods, {$($value)*})] $($rest)*
        }
    };

    // Parse tokens: fields {fields}
    (@parse_tokens [$($acc:tt)*] fields {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)* (fields, {$($value)*})] $($rest)*
        }
    };

    // Parse tokens: static_fields {static_fields}
    (@parse_tokens [$($acc:tt)*] static_fields {$($value:tt)*} $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)* (static_fields, {$($value)*})] $($rest)*
        }
    };

    // Parse tokens: Skip commas
    (@parse_tokens [$($acc:tt)*] , $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse_tokens [$($acc)*] $($rest)*
        }
    };

    // Parse tokens: Error on unexpected tokens
    (@parse_tokens [$($acc:tt)*] $bad:tt $($rest:tt)*) => {
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
            @parse [$($acc)* (init, __drt_Closure(($value)))] $($rest)*
        }
    };

    // Append init = <expr>, no comma (end) - wrap with __drt_Closure to avoid comma ambiguity
    (@parse [$($acc:tt)*] init = $value:expr) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (init, __drt_Closure(($value)))]
        }
    };

    // Append init_with_loader = <expr>, with comma - wrap with __drt_Closure to avoid comma ambiguity
    (@parse [$($acc:tt)*] init_with_loader = $value:expr, $($rest:tt)*) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (init_with_loader, __drt_Closure(($value)))] $($rest)*
        }
    };

    // Append init_with_loader = <expr>, no comma (end) - wrap with __drt_Closure to avoid comma ambiguity
    (@parse [$($acc:tt)*] init_with_loader = $value:expr) => {
        $crate::__def_ref_parse! {
            @parse [$($acc)* (init_with_loader, __drt_Closure(($value)))]
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
            @parse_tokens [(type, $Type) (class, $Class)] $( $($rest)* )?
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
        init = (|_env, _class| {
            println!("Custom init called");
            Ok(Custom0API)
        }),
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
