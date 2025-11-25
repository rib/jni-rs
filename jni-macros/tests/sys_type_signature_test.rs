#![allow(unused)]

// Test that verifies the generated method signatures with __sys_type
//
// This test demonstrates that:
// 1. __sys_type = jclass generates from_raw(jclass) and into_raw() -> jclass
// 2. __sys_type = jthrowable generates from_raw(jthrowable) and into_raw() -> jthrowable
// 3. Default (no __sys_type) generates from_raw(jobject) and into_raw() -> jobject

use jni::sys::{jclass, jobject, jthrowable};

// Binding with __sys_type = jclass
jni_macros::bind_java_type! {
    rust_type = MyClass,
    java_type = "java.lang.Class",
    __jni_core = true,
    __sys_type = jclass,
}

// Binding with __sys_type = jthrowable
jni_macros::bind_java_type! {
    rust_type = MyThrowable,
    java_type = "java.lang.Throwable",
    __jni_core = true,
    __sys_type = jthrowable,
}

// Binding without __sys_type (uses default jobject)
jni_macros::bind_java_type! {
    rust_type = MyObject,
    java_type = "java.lang.Object",
    __jni_core = true,
}

// Test that from_raw accepts jclass and into_raw returns jclass
fn test_signature_jclass(env: &jni::Env, raw: jclass) {
    let my_class = unsafe { MyClass::from_raw(env, raw) };

    // Verify that into_raw returns jclass
    let raw2: jclass = my_class.into_raw();
    let _ = raw2;
}

// Test that from_raw accepts jthrowable and into_raw returns jthrowable
fn test_signature_jthrowable(env: &jni::Env, raw: jthrowable) {
    let my_throwable = unsafe { MyThrowable::from_raw(env, raw) };

    // Verify that into_raw returns jthrowable
    let raw2: jthrowable = my_throwable.into_raw();
    let _ = raw2;
}

// Test that from_raw accepts jobject and into_raw returns jobject (default)
fn test_signature_jobject(env: &jni::Env, raw: jobject) {
    let my_object = unsafe { MyObject::from_raw(env, raw) };

    // Verify that into_raw returns jobject
    let raw2: jobject = my_object.into_raw();
    let _ = raw2;
}

// Test that null() works for all types
fn test_null_constructors() {
    let _class: MyClass = MyClass::null();
    let _throwable: MyThrowable = MyThrowable::null();
    let _object: MyObject = MyObject::null();
}

fn main() {
    // All the test functions compile successfully, which proves that:
    // - MyClass uses jclass as its sys type
    // - MyThrowable uses jthrowable as its sys type
    // - MyObject uses jobject as its sys type (default)
}
