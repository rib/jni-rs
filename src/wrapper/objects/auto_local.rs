use std::mem;

use log::debug;

use crate::{objects::JObject, JNIEnv};

/// Auto-delete wrapper for local refs.
///
/// Anything passed to a foreign method _and_ returned from JNI methods is considered a local ref
/// unless it is specified otherwise.
/// These refs are automatically deleted once the foreign method exits, but it's possible that
/// they may reach the JVM-imposed limit before that happens.
///
/// This wrapper provides automatic local ref deletion when it goes out of
/// scope.
///
/// NOTE: This comes with some potential safety risks. DO NOT use this to wrap
/// something unless you're SURE it won't be used after this wrapper gets
/// dropped. Otherwise, you'll get a nasty JVM crash.
///
/// See also the [JNI specification][spec-references] for details on referencing Java objects
/// and some [extra information][android-jni-references].
///
/// [spec-references]: https://docs.oracle.com/en/java/javase/12/docs/specs/jni/design.html#referencing-java-objects
/// [android-jni-references]: https://developer.android.com/training/articles/perf-jni#local-and-global-references
pub struct AutoLocal<'env, 'b, T: AsRef<JObject<'env>>> {
    obj: T,
    env: &'b JNIEnv<'env>,
}

impl<'env, 'b, T: AsRef<JObject<'env>>> ::std::ops::Deref for AutoLocal<'env, 'b, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

impl<'env, 'b, T: AsRef<JObject<'env>> + From<JObject<'env>>> AutoLocal<'env, 'b, T> {
    /// Creates a new auto-delete wrapper for a local ref.
    ///
    /// Once this wrapper goes out of scope, the `delete_local_ref` will be
    /// called on the object. While wrapped, the object can be accessed via
    /// the `Deref` impl.
    pub fn new(env: &'b JNIEnv<'env>, obj: T) -> Self {
        AutoLocal { obj, env }
    }

    /// Forget the wrapper, returning the original object.
    ///
    /// This prevents `delete_local_ref` from being called when the `AutoLocal`
    /// gets dropped. You must either remember to delete the local ref manually,
    /// or be ok with it getting deleted once the foreign method returns.
    pub fn forget(self) -> T {
        let obj = unsafe { JObject::from_raw(self.as_ref().internal) };
        mem::forget(self);
        obj.into()
    }
}

impl<'env, 'b, T: AsRef<JObject<'env>>> Drop for AutoLocal<'env, 'b, T> {
    fn drop(&mut self) {
        let obj = unsafe { JObject::from_raw(self.as_ref().internal) };
        let res = self.env.delete_local_ref(obj);
        match res {
            Ok(()) => {}
            Err(e) => debug!("error dropping global ref: {:#?}", e),
        }
    }
}

/*
impl<'env, 'b, T: AsRef<JObject<'env>>> From<&'_ AutoLocal<'env, 'b, T>> for &'b JObject<'env> {
    fn from(other: &'_ AutoLocal<'env, 'b, T>) -> Self {
        other.as_obj()
    }
}
*/

impl<'env, 'b, T: AsRef<JObject<'env>>> AsRef<JObject<'env>> for AutoLocal<'env, 'b, T>
{
    fn as_ref(&self) -> &JObject<'env> {
        self.obj.as_ref()
    }
}