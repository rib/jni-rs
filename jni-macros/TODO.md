# Update rustoc jni_sig examples in lib.rs

# Check TODOs in jni-macros/examples/bind_java_type_native_methods_test.rs

# Optionally allow `fn` after a visibility specifier in the syntax for method bindings

# Add some runtime binding tests

# Add some runtime tests of native method bindings

# Add some runtime tests of class initialization depending on native methods

# Allow specification of default native_error_policy

# the jni_mangle macro should use #[unsafe(export_name = )]

Using `export_name =` means that the unmangled name remains usable in Rust code.

That makes it more practical to use with the `native_method_struct` macro

# Add export_native_method and export_static_native_method macros

These should expand to a wrapper function with name mangling that calls some other given implementation function.

These should also output a `const MY_FOO__NATIVE_METHOD: NativeMethod` that can be
used with `env.register_native_methods()`

These should support a `raw` or safe `fn` function where the `raw` version will
will accept an `EnvUnowned` and a safe `fn` will accept an `&mut Env` and the
wrapper will automatically handle calling `EnvUnowned::with_env` and `resolve()`
the return value based on a configurable `ErrorPolicy`.

# Add support for `jlong` type aliases in `type_map`

Support smart pointer wrappers like:

```
type_map {
    JHandle<Custom> => jlong,
}
```

Allow anything in the form `Path < Path, Path >`

TypeMappings will probably need some other adaptations to allow tracking of
primitive type aliases.

Code for mapping signature types to Rust types will probably also need some
special case handling now for the possibility that we map to a primitive type
that does not require a lifetime.

Related to this, the codegen should be preserving the name of primitive type
aliases instead of always normalizing to jni::sys types.

This matters for this specific case, but also allows for nicer bindings that
prefer to expose Rust types instead of jni::sys types in the Rust api.


