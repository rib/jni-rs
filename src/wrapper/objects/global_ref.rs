use std::{convert::From, sync::Arc, mem::transmute};

use log::{debug, warn};

use crate::{errors::Result, objects::JObject, sys, JNIEnv, JavaVM};

use super::{JClass, IsObject};

/// A global JVM reference. These are "pinned" by the garbage collector and are
/// guaranteed to not get collected until released. Thus, this is allowed to
/// outlive the `JNIEnv` that it came from and can be used in other threads.
///
/// `GlobalRef` can be cloned to use _the same_ global reference in different
/// contexts. If you want to create yet another global ref to the same java object
/// you may call `JNIEnv#new_global_ref` just like you do when create `GlobalRef`
/// from a local reference.
///
/// Underlying global reference will be dropped, when the last instance
/// of `GlobalRef` leaves its scope.
///
/// It is _recommended_ that a native thread that drops the global reference is attached
/// to the Java thread (i.e., has an instance of `JNIEnv`). If the native thread is *not* attached,
/// the `GlobalRef#drop` will print a warning and implicitly `attach` and `detach` it, which
/// significantly affects performance.

#[derive(Clone, Debug)]
pub struct GlobalRef<T: IsObject + 'static> {
    inner: Arc<GlobalRefGuard<T>>,
}

#[derive(Debug)]
struct GlobalRefGuard<T: IsObject + 'static> {
    obj: T,
    vm: JavaVM,
}

unsafe impl<T: IsObject> Send for GlobalRef<T> {}
unsafe impl<T: IsObject> Sync for GlobalRef<T> {}

impl<T: IsObject> AsRef<JObject<'static>> for GlobalRef<T> {
    fn as_ref(&self) -> &JObject<'static> {
        &*self
    }
}

impl<T: IsObject> ::std::ops::Deref for GlobalRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner.obj
    }
}

impl<T: IsObject> GlobalRef<T> {
    /// Creates a new wrapper for a global reference.
    ///
    /// # Safety
    ///
    /// Expects a valid raw global reference that should be created with `NewGlobalRef` JNI function.
    pub(crate) unsafe fn from_raw(vm: JavaVM, raw_global_ref: sys::jobject) -> Self {
        GlobalRef {
            inner: Arc::new(GlobalRefGuard::from_raw(vm, raw_global_ref)),
        }
    }
}

impl<T: IsObject> GlobalRefGuard<T> {
    /// Creates a new global reference guard. This assumes that `NewGlobalRef`
    /// has already been called.
    unsafe fn from_raw(vm: JavaVM, obj: sys::jobject) -> Self {
        GlobalRefGuard {
            obj: JObject::from_raw(obj),
            vm,
        }
    }
}

impl<T: IsObject> Drop for GlobalRefGuard<T> {
    fn drop(&mut self) {
        fn drop_impl(env: &JNIEnv, global_ref: crate::sys::jobject) -> Result<()> {
            let internal = env.get_native_interface();
            // This method is safe to call in case of pending exceptions (see chapter 2 of the spec)
            jni_unchecked!(internal, DeleteGlobalRef, global_ref);
            Ok(())
        }

        let res = match self.vm.get_env() {
            Ok(env) => drop_impl(&env, self.obj.internal),
            Err(_) => {
                warn!("Dropping a GlobalRef in a detached thread. Fix your code if this message appears frequently (see the GlobalRef docs).");
                self.vm
                    .attach_current_thread()
                    .and_then(|env| drop_impl(&env, self.obj.internal))
            }
        };

        if let Err(err) = res {
            debug!("error dropping global ref: {:#?}", err);
        }
    }
}
