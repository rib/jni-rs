//! Copy of JNI call macros from src/macros.rs

/// Directly calls a Env FFI function, nothing else
///
/// # Safety
///
/// When calling any function added after JNI 1.1 you must know that it's valid
/// for the current JNI version.
#[doc(hidden)]
#[macro_export]
macro_rules! jni_call_unchecked {
    ( $jnienv:expr, $version:tt, $name:ident $(, $args:expr )*) => {{
        // Safety: we know that the Env pointer can't be null, since that's
        // checked in `from_raw()`
        let env: *mut jni::sys::JNIEnv = $jnienv.get_raw();
        let interface: *const jni_sys::JNINativeInterface_ = *env;
        ((*interface).$version.$name)(env $(, $args)*)
    }};
}

/// Calls a Env function, then checks for a pending exception
///
/// This only checks for an exception, it doesn't clear the exception and so the
/// exception will be thrown if the native code returns to the JVM.
///
/// Returns `Err` if there is a pending exception after the call.
#[doc(hidden)]
#[macro_export]
macro_rules! jni_call_check_ex {
    ( $jnienv:expr, $version:tt, $name:ident $(, $args:expr )* ) => ({
        let ret = $crate::jni_call_unchecked!($jnienv, $version, $name $(, $args)*);
        if $jnienv.exception_check() {
            Err($crate::errors::Error::JavaException)
        } else {
            Ok(ret)
        }
    })
}
