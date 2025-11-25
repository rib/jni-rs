mod util;

use jni::objects::{JClass, JPrimitiveArray, JString};
use jni::strings::JNIString;
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jint, jlong};
use jni::{Env, EnvUnowned};
use jni::{bind_java_type, jni_str};
use rusty_fork::rusty_fork_test;
use std::fs;
use std::path::{Path, PathBuf};

// Raw JNI function implementations for testing unsafe fn feature
extern "system" fn raw_add_impl<'local>(
    _env: EnvUnowned<'local>,
    _this: TestNativeRaw<'local>,
    a: jint,
    b: jint,
) -> jint {
    a + b
}

extern "system" fn raw_is_positive_impl<'local>(
    _env: EnvUnowned<'local>,
    _this: TestNativeRaw<'local>,
    value: jint,
) -> jboolean {
    if value > 0 { JNI_TRUE } else { JNI_FALSE }
}

extern "system" fn raw_process_string_impl<'local>(
    mut env: EnvUnowned<'local>,
    _this: TestNativeRaw<'local>,
    input: JString<'local>,
) -> JString<'local> {
    env.with_env(|env| {
        let input_str = input.to_string();
        let result = format!("raw: {}", input_str);
        JString::from_str(env, result.as_str())
    })
    .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

extern "system" fn raw_multiply_impl<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    a: jint,
    b: jint,
) -> jint {
    a * b
}

extern "system" fn raw_is_even_impl<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    value: jint,
) -> jboolean {
    if value % 2 == 0 { JNI_TRUE } else { JNI_FALSE }
}

// Create bindings for TestNativeMethods class
bind_java_type! {
    rust_type = TestNativeMethods,
    java_type = "com.example.TestNativeMethods",
    constructors {
        fn new(),
    },
    methods {
        fn get_counter() -> jint,
        fn set_counter(value: jint) -> void,
        fn get_message() -> JString,
        fn set_message(msg: JString) -> void,
        // Wrapper methods that call native methods
        fn call_native_add(a: jint, b: jint) -> jint,
        fn call_native_process_string(input: JString) -> JString,
        fn call_native_log(msg: JString) -> void,
        fn call_native_sum_array(arr: jint[]) -> jint,
        fn call_native_set_counter(value: jint) -> void,
        fn call_native_get_counter() -> jint,
        fn call_native_set_message(msg: JString) -> void,
        fn call_native_get_message() -> JString,
        static fn call_native_get_version() -> jint,
        static fn call_native_initialize(config: JString, flags: jint) -> jboolean,
        static fn call_native_concat_static(a: JString, b: JString) -> JString,
        static fn call_native_multiply(a: jlong, b: jlong) -> jlong,
    },
    native_methods {
        fn native_add {
            sig = (a: jint, b: jint) -> jint,
        },
        fn native_process_string {
            sig = (input: JString) -> JString,
        },
        fn native_log {
            sig = (message: JString) -> void,
        },
        fn native_sum_array {
            sig = (arr: jint[]) -> jint,
        },
        fn native_set_counter {
            sig = (value: jint) -> void,
        },
        fn native_get_counter {
            sig = () -> jint,
        },
        fn native_set_message {
            sig = (msg: JString) -> void,
        },
        fn native_get_message {
            sig = () -> JString,
        },
        static fn native_get_version {
            sig = () -> jint,
        },
        static fn native_initialize {
            sig = (config: JString, flags: jint) -> jboolean,
        },
        static fn native_concat_static {
            sig = (a: JString, b: JString) -> JString,
        },
        static fn native_multiply {
            sig = (a: jlong, b: jlong) -> jlong,
        },
    }
}

// Implement the native methods trait
impl TestNativeMethodsNativeInterface for TestNativeMethodsAPI {
    type Error = jni::errors::Error;

    fn native_add<'local>(
        _env: &mut Env<'local>,
        _this: TestNativeMethods<'local>,
        a: jint,
        b: jint,
    ) -> Result<jint, Self::Error> {
        Ok(a + b)
    }

    fn native_process_string<'local>(
        env: &mut Env<'local>,
        _this: TestNativeMethods<'local>,
        input: JString<'local>,
    ) -> Result<JString<'local>, Self::Error> {
        let input_str = input.to_string();
        let result = format!("processed: {}", input_str);
        JString::from_str(env, result.as_str())
    }

    fn native_log<'local>(
        _env: &mut Env<'local>,
        _this: TestNativeMethods<'local>,
        message: JString<'local>,
    ) -> Result<(), Self::Error> {
        println!("Native log: {}", message);
        Ok(())
    }

    fn native_sum_array<'local>(
        env: &mut Env<'local>,
        _this: TestNativeMethods<'local>,
        arr: JPrimitiveArray<'local, jint>,
    ) -> Result<jint, Self::Error> {
        unsafe {
            let elements = arr.get_elements_critical(env, jni::objects::ReleaseMode::NoCopyBack)?;
            let sum: jint = elements.iter().sum();
            Ok(sum)
        }
    }

    fn native_set_counter<'local>(
        env: &mut Env<'local>,
        this: TestNativeMethods<'local>,
        value: jint,
    ) -> Result<(), Self::Error> {
        this.set_counter(env, value)
    }

    fn native_get_counter<'local>(
        env: &mut Env<'local>,
        this: TestNativeMethods<'local>,
    ) -> Result<jint, Self::Error> {
        this.get_counter(env)
    }

    fn native_set_message<'local>(
        env: &mut Env<'local>,
        this: TestNativeMethods<'local>,
        msg: JString<'local>,
    ) -> Result<(), Self::Error> {
        this.set_message(env, &msg)
    }

    fn native_get_message<'local>(
        env: &mut Env<'local>,
        this: TestNativeMethods<'local>,
    ) -> Result<JString<'local>, Self::Error> {
        this.get_message(env)
    }

    fn native_get_version<'local>(
        _env: &mut Env<'local>,
        _class: JClass<'local>,
    ) -> Result<jint, Self::Error> {
        Ok(100)
    }

    fn native_initialize<'local>(
        _env: &mut Env<'local>,
        _class: JClass<'local>,
        config: JString<'local>,
        flags: jint,
    ) -> Result<jboolean, Self::Error> {
        let config_str = config.to_string();
        println!("Initializing with config: {}, flags: {}", config_str, flags);
        Ok(true)
    }

    fn native_concat_static<'local>(
        env: &mut Env<'local>,
        _class: JClass<'local>,
        a: JString<'local>,
        b: JString<'local>,
    ) -> Result<JString<'local>, Self::Error> {
        let a_str = a.to_string();
        let b_str = b.to_string();
        let result = format!("{}{}", a_str, b_str);
        JString::from_str(env, result)
    }

    fn native_multiply<'local>(
        _env: &mut Env<'local>,
        _class: JClass<'local>,
        a: jlong,
        b: jlong,
    ) -> Result<jlong, Self::Error> {
        Ok(a * b)
    }
}

rusty_fork_test! {
#[test]
fn test_native_method_basic_arithmetic() {
    let out_dir = setup_test_output("bind_native_methods_arithmetic");

    // Compile Java class
    javac::Build::new()
        .file("tests/java/com/example/TestNativeMethods.java")
        .output_dir(&out_dir)
        .compile();

    util::attach_current_thread(|env| {
        load_test_native_methods_class(env, &out_dir)?;

        // Initialize the API (this registers native methods automatically)
        let loader = jni::refs::LoaderContext::default();
        TestNativeMethodsAPI::get(env, &loader)?;

        let obj = TestNativeMethods::new(env)?;

        // Call Java method which calls native method
        let result = obj.call_native_add(env, 10, 20)?;
        assert_eq!(result, 30);

        let result = obj.call_native_add(env, 5, 7)?;
        assert_eq!(result, 12);

        Ok(())
    })
    .expect("Basic arithmetic test failed");
}
}

rusty_fork_test! {
#[test]
fn test_native_method_string_processing() {
    let out_dir = setup_test_output("bind_native_methods_strings");

    javac::Build::new()
        .file("tests/java/com/example/TestNativeMethods.java")
        .output_dir(&out_dir)
        .compile();

    util::attach_current_thread(|env| {
        load_test_native_methods_class(env, &out_dir)?;
        let loader = jni::refs::LoaderContext::default();
        TestNativeMethodsAPI::get(env, &loader)?;

        let obj = TestNativeMethods::new(env)?;

        // Test native_process_string via wrapper
        let input = JString::from_str(env, "test input")?;
        let result = obj.call_native_process_string(env, &input)?;
        assert_eq!(result.to_string(), "processed: test input");

        Ok(())
    })
    .expect("String processing test failed");
}
}

rusty_fork_test! {
#[test]
fn test_native_method_void_return() {
    let out_dir = setup_test_output("bind_native_methods_void");

    javac::Build::new()
        .file("tests/java/com/example/TestNativeMethods.java")
        .output_dir(&out_dir)
        .compile();

    util::attach_current_thread(|env| {
        load_test_native_methods_class(env, &out_dir)?;
        let loader = jni::refs::LoaderContext::default();
        TestNativeMethodsAPI::get(env, &loader)?;

        let obj = TestNativeMethods::new(env)?;

        // Test native_log (void return) via wrapper
        let message = JString::from_str(env, "test log message")?;
        obj.call_native_log(env, &message)?;

        Ok(())
    })
    .expect("Void return test failed");
}
}

rusty_fork_test! {
#[test]
fn test_native_method_array_argument() {
    let out_dir = setup_test_output("bind_native_methods_arrays");

    javac::Build::new()
        .file("tests/java/com/example/TestNativeMethods.java")
        .output_dir(&out_dir)
        .compile();

    util::attach_current_thread(|env| {
        load_test_native_methods_class(env, &out_dir)?;
        let loader = jni::refs::LoaderContext::default();
        TestNativeMethodsAPI::get(env, &loader)?;

        let obj = TestNativeMethods::new(env)?;

        // Create an int array
        let arr = env.new_int_array(5)?;
        let data = [1, 2, 3, 4, 5];
        arr.set_region(env, 0, &data)?;

        // Test native_sum_array via wrapper
        let sum = obj.call_native_sum_array(env, arr)?;
        assert_eq!(sum, 15);

        Ok(())
    })
    .expect("Array argument test failed");
}
}

rusty_fork_test! {
#[test]
fn test_native_method_state_mutation() {
    let out_dir = setup_test_output("bind_native_methods_state");

    javac::Build::new()
        .file("tests/java/com/example/TestNativeMethods.java")
        .output_dir(&out_dir)
        .compile();

    util::attach_current_thread(|env| {
        load_test_native_methods_class(env, &out_dir)?;
        let loader = jni::refs::LoaderContext::default();
        TestNativeMethodsAPI::get(env, &loader)?;

        let obj = TestNativeMethods::new(env)?;

        // Test initial state via native methods
        let counter = obj.call_native_get_counter(env)?;
        assert_eq!(counter, 0);

        let message = obj.call_native_get_message(env)?;
        assert_eq!(message.to_string(), "initial");

        // Modify state via native methods
        obj.call_native_set_counter(env, 42)?;
        let counter = obj.call_native_get_counter(env)?;
        assert_eq!(counter, 42);

        // Verify the change is visible via non-native methods
        let counter = obj.get_counter(env)?;
        assert_eq!(counter, 42);

        // Modify message via native method
        let new_msg = JString::from_str(env, "updated")?;
        obj.call_native_set_message(env, &new_msg)?;
        let message = obj.call_native_get_message(env)?;
        assert_eq!(message.to_string(), "updated");

        // Verify the change is visible via non-native methods
        let message = obj.get_message(env)?;
        assert_eq!(message.to_string(), "updated");

        Ok(())
    })
    .expect("State mutation test failed");
}
}

rusty_fork_test! {
#[test]
fn test_static_native_methods() {
    let out_dir = setup_test_output("bind_native_methods_static");

    javac::Build::new()
        .file("tests/java/com/example/TestNativeMethods.java")
        .output_dir(&out_dir)
        .compile();

    util::attach_current_thread(|env| {
        load_test_native_methods_class(env, &out_dir)?;
        let loader = jni::refs::LoaderContext::default();
        TestNativeMethodsAPI::get(env, &loader)?;

        // Test static native methods (no object instance needed)

        // Test native_get_version
        let version = TestNativeMethods::call_native_get_version(env)?;
        assert_eq!(version, 100);

        // Test native_initialize
        let config = JString::from_str(env, "test config")?;
        let result = TestNativeMethods::call_native_initialize(env, &config, 42)?;
        assert!(result);

        // Test native_concat_static
        let a = JString::from_str(env, "Hello, ")?;
        let b = JString::from_str(env, "World!")?;
        let result = TestNativeMethods::call_native_concat_static(env, &a, &b)?;
        assert_eq!(result.to_string(), "Hello, World!");

        // Test native_multiply
        let result = TestNativeMethods::call_native_multiply(env, 1000, 2000)?;
        assert_eq!(result, 2_000_000);

        Ok(())
    })
    .expect("Static native methods test failed");
}
}

rusty_fork_test! {
#[test]
fn test_native_methods_multiple_instances() {
    let out_dir = setup_test_output("bind_native_methods_instances");

    javac::Build::new()
        .file("tests/java/com/example/TestNativeMethods.java")
        .output_dir(&out_dir)
        .compile();

    util::attach_current_thread(|env| {
        load_test_native_methods_class(env, &out_dir)?;
        let loader = jni::refs::LoaderContext::default();
        TestNativeMethodsAPI::get(env, &loader)?;

        // Create multiple instances
        let obj1 = TestNativeMethods::new(env)?;
        let obj2 = TestNativeMethods::new(env)?;

        // Set different states via native methods
        obj1.call_native_set_counter(env, 10)?;
        obj2.call_native_set_counter(env, 20)?;

        // Verify each instance maintains its own state
        let counter1 = obj1.call_native_get_counter(env)?;
        let counter2 = obj2.call_native_get_counter(env)?;
        assert_eq!(counter1, 10);
        assert_eq!(counter2, 20);

        // Set messages
        let msg1 = JString::from_str(env, "first")?;
        let msg2 = JString::from_str(env, "second")?;
        obj1.call_native_set_message(env, &msg1)?;
        obj2.call_native_set_message(env, &msg2)?;

        // Verify messages are independent
        let result1 = obj1.call_native_get_message(env)?;
        let result2 = obj2.call_native_get_message(env)?;
        assert_eq!(result1.to_string(), "first");
        assert_eq!(result2.to_string(), "second");

        Ok(())
    })
    .expect("Multiple instances test failed");
}
}

rusty_fork_test! {
#[test]
fn test_native_methods_registration_idempotent() {
    let out_dir = setup_test_output("bind_native_methods_reregister");

    javac::Build::new()
        .file("tests/java/com/example/TestNativeMethods.java")
        .output_dir(&out_dir)
        .compile();

    util::attach_current_thread(|env| {
        load_test_native_methods_class(env, &out_dir)?;

        let loader = jni::refs::LoaderContext::default();

        // Get API multiple times (should be safe and use cached version)
        TestNativeMethodsAPI::get(env, &loader)?;
        TestNativeMethodsAPI::get(env, &loader)?;
        TestNativeMethodsAPI::get(env, &loader)?;

        let obj = TestNativeMethods::new(env)?;

        // Test that methods still work after multiple get() calls
        let result = obj.call_native_add(env, 5, 10)?;
        assert_eq!(result, 15);

        Ok(())
    })
    .expect("Registration idempotent test failed");
}
}

rusty_fork_test! {
#[test]
fn test_native_method_in_static_initializer() {
    let out_dir = setup_test_output("bind_native_methods_static_init");

    javac::Build::new()
        .file("tests/java/com/example/TestNativeStaticInit.java")
        .output_dir(&out_dir)
        .compile();

    util::attach_current_thread(|env| {
        // Step 1: Define the class (but don't initialize it yet)
        let class_path = out_dir.join("com/example/TestNativeStaticInit.class");
        assert!(
            class_path.exists(),
            "TestNativeStaticInit.class not found at {:?}",
            class_path
        );

        let class_bytes = fs::read(&class_path).expect("Failed to read TestNativeStaticInit.class");
        let class_loader = jni::objects::JClassLoader::get_system_class_loader(env)
            .expect("Failed to get system class loader");

        // Define the class - this does NOT run the static initializer
        env.define_class(
            Some(jni_str!("com/example/TestNativeStaticInit")),
            &class_loader,
            &class_bytes,
        )
        .expect("Failed to define TestNativeStaticInit class");

        // Step 2: Register native methods BEFORE the class is initialized
        // This is the critical step - we must register the native methods
        // before the static initializer runs
        let loader = jni::refs::LoaderContext::default();
        TestNativeStaticInitAPI::get(env, &loader)?;

        println!("Native methods registered, now creating instance...");

        // Step 3: Create an instance - this will trigger class initialization
        // The static initializer will run and call our native methods
        let obj = TestNativeStaticInit::new(env)?;

        // Step 4: Verify that the static initializer ran successfully
        // and our native methods were called
        let static_value = TestNativeStaticInit::get_static_value(env)?;
        assert_eq!(static_value, 42, "Static value should be set by native method");

        let static_message = TestNativeStaticInit::get_static_message(env)?;
        assert_eq!(
            static_message.to_string(),
            "Initialized from native",
            "Static message should be set by native method"
        );

        // Verify instance methods work too
        let instance_message = obj.get_message(env)?;
        assert!(
            instance_message.to_string().contains("static value was: 42"),
            "Instance should see the static value"
        );

        Ok(())
    })
    .expect("Static initializer native method test failed");
}
}

// Bindings for TestNativeStaticInit class
bind_java_type! {
    rust_type = TestNativeStaticInit,
    java_type = "com.example.TestNativeStaticInit",
    constructors {
        fn new(),
    },
    methods {
        static fn get_static_value() -> jint,
        static fn get_static_message() -> JString,
        fn get_message() -> JString,
    },
    native_methods {
        static fn native_initialize_static {
            sig = () -> jint,
        },
        static fn native_get_static_message {
            sig = () -> JString,
        },
    }
}

// Implement the native methods trait for static initializer test
impl TestNativeStaticInitNativeInterface for TestNativeStaticInitAPI {
    type Error = jni::errors::Error;

    fn native_initialize_static<'local>(
        _env: &mut Env<'local>,
        _class: JClass<'local>,
    ) -> Result<jint, Self::Error> {
        println!("native_initialize_static called from static initializer!");
        Ok(42)
    }

    fn native_get_static_message<'local>(
        env: &mut Env<'local>,
        _class: JClass<'local>,
    ) -> Result<JString<'local>, Self::Error> {
        println!("native_get_static_message called from static initializer!");
        JString::from_str(env, "Initialized from native")
    }
}

// Bindings for TestNativeRaw class (testing raw unsafe function pointers)
bind_java_type! {
    rust_type = TestNativeRaw,
    java_type = "com.example.TestNativeRaw",
    constructors {
        fn new(),
    },
    methods {
        fn call_raw_add(a: jint, b: jint) -> jint,
        fn call_raw_is_positive(value: jint) -> jboolean,
        fn call_raw_process_string(input: JString) -> JString,
        fn call_regular_method(value: jint) -> jint,
        static fn call_raw_multiply(a: jint, b: jint) -> jint,
        static fn call_raw_is_even(value: jint) -> jboolean,
    },
    native_methods {
        // Raw unsafe function pointers
        fn raw_add {
            sig = (a: jint, b: jint) -> jint,
            fn = raw_add_impl,
            raw = true,
        },
        fn raw_is_positive {
            sig = (value: jint) -> jboolean,
            fn = raw_is_positive_impl,
            raw = true,
        },
        fn raw_process_string {
            sig = (input: JString) -> JString,
            fn = raw_process_string_impl,
            raw = true,
        },
        // Regular trait-based method
        fn regular_method {
            sig = (value: jint) -> jint,
        },
        // Raw unsafe function pointers for static methods
        static fn raw_multiply {
            sig = (a: jint, b: jint) -> jint,
            fn = raw_multiply_impl,
            raw = true,
        },
        static fn raw_is_even {
            sig = (value: jint) -> jboolean,
            fn = raw_is_even_impl,
            raw = true,
        },
    }
}

// Implement only the regular (non-raw) method
impl TestNativeRawNativeInterface for TestNativeRawAPI {
    type Error = jni::errors::Error;

    fn regular_method<'local>(
        _env: &mut Env<'local>,
        _this: TestNativeRaw<'local>,
        value: jint,
    ) -> Result<jint, Self::Error> {
        Ok(value * 10)
    }
}

rusty_fork_test! {
#[test]
fn test_raw_native_functions() {
    let out_dir = setup_test_output("bind_native_methods_raw");

    javac::Build::new()
        .file("tests/java/com/example/TestNativeRaw.java")
        .output_dir(&out_dir)
        .compile();

    util::attach_current_thread(|env| {
        load_test_class(env, &out_dir, "TestNativeRaw")?;
        let loader = jni::refs::LoaderContext::default();
        TestNativeRawAPI::get(env, &loader)?;

        let obj = TestNativeRaw::new(env)?;

        // Test raw add function
        let result = obj.call_raw_add(env, 10, 20)?;
        assert_eq!(result, 30, "Raw add should work");

        // Test raw is_positive function
        let result = obj.call_raw_is_positive(env, 5)?;
        assert!(result, "Raw is_positive should return true for positive");

        let result = obj.call_raw_is_positive(env, -5)?;
        assert!(!result, "Raw is_positive should return false for negative");

        // Test raw string processing
        let input = JString::from_str(env, "test")?;
        let result = obj.call_raw_process_string(env, &input)?;
        assert_eq!(result.to_string(), "raw: test", "Raw string processing should work");

        // Test raw static multiply
        let result = TestNativeRaw::call_raw_multiply(env, 7, 8)?;
        assert_eq!(result, 56, "Raw static multiply should work");

        // Test raw static is_even
        let result = TestNativeRaw::call_raw_is_even(env, 4)?;
        assert!(result, "Raw is_even should return true for even");

        let result = TestNativeRaw::call_raw_is_even(env, 5)?;
        assert!(!result, "Raw is_even should return false for odd");

        // Test regular trait-based method (mixed with raw methods)
        let result = obj.call_regular_method(env, 5)?;
        assert_eq!(result, 50, "Regular method should work alongside raw methods");

        Ok(())
    })
    .expect("Raw native functions test failed");
}
}

// Bindings for TestNativeExported class (testing exported native methods)
bind_java_type! {
    rust_type = TestNativeExported,
    java_type = "com.example.TestNativeExported",
    export_native_methods = true,  // Enable export for auto-discovery
    native_methods {
        static fn native_init_value {
            sig = () -> jint,
            export = true,  // Explicitly export for JVM discovery
        },
        static fn native_get_version {
            sig = () -> JString,
            export = true,
        },
        fn native_double_value {
            sig = (value: jint) -> jint,
            export = true,
        },
    }
}

impl TestNativeExportedNativeInterface for TestNativeExportedAPI {
    type Error = jni::errors::Error;

    fn native_init_value<'local>(
        _env: &mut Env<'local>,
        _class: JClass<'local>,
    ) -> Result<jint, Self::Error> {
        println!("native_init_value called via exported function!");
        Ok(999)
    }

    fn native_get_version<'local>(
        env: &mut Env<'local>,
        _class: JClass<'local>,
    ) -> Result<JString<'local>, Self::Error> {
        JString::from_str(env, "v1.0-exported")
    }

    fn native_double_value<'local>(
        _env: &mut Env<'local>,
        _this: TestNativeExported<'local>,
        value: jint,
    ) -> Result<jint, Self::Error> {
        Ok(value * 2)
    }
}

#[test]
fn test_exported_native_methods_exist() {
    // Test that the exported mangled function names exist
    // by assigning them to variables. This will cause a compile-time
    // error if the functions don't exist.

    // These are the expected JNI mangled function names for the exported methods
    /*
    unsafe extern "system" {
        fn Java_com_example_TestNativeExported_nativeInitValue__(
            env: jni::sys::JNIEnv,
            class: jni::sys::jclass,
        ) -> jni::sys::jint;

        fn Java_com_example_TestNativeExported_nativeGetVersion__(
            env: jni::sys::JNIEnv,
            class: jni::sys::jclass,
        ) -> jni::sys::jstring;

        fn Java_com_example_TestNativeExported_nativeDoubleValue__I(
            env: jni::sys::JNIEnv,
            this: jni::sys::jobject,
            value: jni::sys::jint,
        ) -> jni::sys::jint;
    }
    */

    // Assign to variables to verify they exist
    let _init_fn = Java_com_example_TestNativeExported_nativeInitValue__;
    let _version_fn = Java_com_example_TestNativeExported_nativeGetVersion__;
    let _double_fn = Java_com_example_TestNativeExported_nativeDoubleValue__I;

    println!("All exported native method functions exist!");
}

// Helper function to load the TestNativeMethods class
fn load_test_native_methods_class(env: &mut Env, out_dir: &Path) -> jni::errors::Result<()> {
    load_test_class(env, out_dir, "TestNativeMethods")
}

// Generic helper function to load any test class
fn load_test_class(env: &mut Env, out_dir: &Path, class_name: &str) -> jni::errors::Result<()> {
    let class_path = out_dir.join(format!("com/example/{}.class", class_name));
    assert!(
        class_path.exists(),
        "{}.class not found at {:?}",
        class_name,
        class_path
    );

    let class_bytes =
        fs::read(&class_path).unwrap_or_else(|_| panic!("Failed to read {}.class", class_name));

    let class_loader = jni::objects::JClassLoader::get_system_class_loader(env)
        .expect("Failed to get system class loader");

    let class_internal_name = format!("com/example/{}", class_name);
    let class_jni = JNIString::new(class_internal_name.as_str());

    env.define_class(Some(&class_jni), &class_loader, &class_bytes)
        .unwrap_or_else(|_| panic!("Failed to define {} class", class_name));

    Ok(())
}

// Helper function to set up test output directory
fn setup_test_output(test_name: &str) -> PathBuf {
    let out_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("jni_macros_tests")
        .join(test_name);

    // Clean up any existing output
    let _ = fs::remove_dir_all(&out_dir);
    fs::create_dir_all(&out_dir).expect("Failed to create test output directory");

    out_dir
}
