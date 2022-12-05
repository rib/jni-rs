use crate::{errors::*, JNIEnv, objects::{AutoLocal, JObject}};

use crate::sys::jobject;

pub enum FromEnvValue<'env, 'b, T: AsRef<JObject<'env>>> {
    Reference(&'b T),
    Owned(AutoLocal<'env, 'b, T>)
}

impl<'env, 'b, T: AsRef<JObject<'env>>> FromEnvValue<'env, 'b, T> {
    pub(crate) fn as_raw(&self) -> jobject {
        match self {
            FromEnvValue::Owned(auto) => {
                auto.as_ref().internal
            }
            FromEnvValue::Reference(r) => {
                let obj: &JObject = r.as_ref();
                obj.internal
            }
        }
    }
}

impl<'env, 'b, T: AsRef<JObject<'env>>> AsRef<JObject<'env>> for FromEnvValue<'env, 'b, T> {
    fn as_ref(&self) -> &JObject<'env> {
        match self {
            FromEnvValue::Owned(auto) => {
                auto.as_ref()
            }
            FromEnvValue::Reference(r) => {
                r.as_ref()
            }
        }
    }
}

impl<'env, 'b, T: AsRef<JObject<'env>>> ::std::ops::Deref for FromEnvValue<'env, 'b, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            FromEnvValue::Owned(auto) => {
                auto.deref()
            }
            FromEnvValue::Reference(r) => {
                r
            }
        }
    }
}

/// Trait for things that can be looked up through the JNI via a descriptor.
/// This will be something like the fully-qualified class name
/// `java/lang/String` or a tuple containing a class descriptor, method name,
/// and method signature. For convenience, this is also implemented for the
/// concrete types themselves in addition to their descriptors.
pub trait FromEnvObject<'env, 'b, T: AsRef<JObject<'env>>> {
    /// Look up the concrete type from the JVM.
    fn lookup<'c>(self, _: &'c JNIEnv<'env>) -> Result<FromEnvValue<'env, 'b, T>>;
}

impl<'env, 'b, T: AsRef<JObject<'env>>> FromEnvObject<'env, 'b, T> for &T {
    fn lookup<'c>(self, _: &'c JNIEnv<'env>) -> Result<FromEnvValue<'env, 'b, T>> {
        Ok(FromEnvValue::Reference(self))
    }
}

/// Trait for things that can be looked up through the JNI via a descriptor.
/// This will be something like the fully-qualified class name
/// `java/lang/String` or a tuple containing a class descriptor, method name,
/// and method signature. For convenience, this is also implemented for the
/// concrete types themselves in addition to their descriptors.
pub trait FromEnvId<'env, T> {
    /// Look up the concrete type from the JVM.
    fn lookup(self, _: &JNIEnv<'env>) -> Result<T>;
}

impl<'env, T> FromEnvId<'env, T> for T
{
    fn lookup(self, env: &JNIEnv<'env>) -> Result<T> {
        Ok(self)
    }
}

/*
impl<'env, T> IntoEnvId<'env, T> for &T {
    fn lookup(self, _: &JNIEnv<'env>) -> Result<T> {
        Ok()
    }
}*/