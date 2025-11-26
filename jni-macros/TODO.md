# Update rustoc jni_sig examples in lib.rs

# Check TODOs in jni-macros/examples/bind_java_type_native_methods_test.rs

# Add some runtime binding tests

# Add some runtime tests of native method bindings

# Add some runtime tests of class initialization depending on native methods

# Allow specification of default native_error_policy

# Add export_native_method and export_static_native_method macros

These should expand to a wrapper function with name mangling that calls some other given implementation function.

These should also output a `const MY_FOO__NATIVE_METHOD: NativeMethod` that can be
used with `env.register_native_methods()`

These should support a `raw` or safe `fn` function where the `raw` version will
will accept an `EnvUnowned` and a safe `fn` will accept an `&mut Env` and the
wrapper will automatically handle calling `EnvUnowned::with_env` and `resolve()`
the return value based on a configurable `ErrorPolicy`.
