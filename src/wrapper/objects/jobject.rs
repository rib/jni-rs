use std::marker::PhantomData;

use crate::sys::jobject;

pub(crate) trait IsObject {}

/// Wrapper around `sys::jobject` that adds a lifetime.
///
/// The lifetime helps ensure that the underlying pointer can't be held by
/// Rust code for longer than the local reference remains valid.
///
/// Furthermore a `JObject` is also _non-_`Copy` because some APIs are able
/// to delete a local reference earlier than a lifetime would otherwise
/// imply. Those APIs need to drop the `JObject` to further ensure that
/// Rust code won't try to use an invalid local reference (pointer).
///
/// It matches C's representation of the raw pointer, so it can be used in any
/// of the extern function argument positions that would take a `jobject`.
///
/// Most other types in the `objects` module deref to this, as they do in the C
/// representation.
#[repr(transparent)]
#[derive(Debug)]
pub struct JObject<'a> {
    pub(crate) internal: jobject,
    lifetime: PhantomData<&'a ()>,
}

impl<'a> IsObject for JObject<'a> {}

impl<'a> ::std::ops::Deref for JObject<'a> {
    type Target = jobject;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl<'a> AsRef<JObject<'a>> for JObject<'a> {
    fn as_ref(&'_ self) -> &'_ JObject<'a> {
        self
    }
}

impl<'a> JObject<'a> {
    /// Creates a [`JObject`] that wraps the given `raw` [`jobject`]
    ///
    /// # Safety
    ///
    /// Expects a valid pointer or `null`
    pub unsafe fn from_raw(raw: jobject) -> Self {
        Self {
            internal: raw,
            lifetime: PhantomData,
        }
    }

    /// Unwrap to the internal jni type.
    pub fn into_raw(self) -> jobject {
        self.internal
    }

    /// Creates a new null object
    pub fn null() -> JObject<'a> {
        unsafe { Self::from_raw(std::ptr::null_mut() as jobject) }
    }
}

impl<'a> std::default::Default for JObject<'a> {
    fn default() -> Self {
        Self::null()
    }
}
