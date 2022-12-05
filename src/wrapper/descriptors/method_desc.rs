use crate::{
    descriptors::FromEnvId,
    errors::*,
    objects::{JClass, JMethodID, JStaticMethodID},
    strings::JNIString,
    JNIEnv,
};

use super::FromEnvObject;

impl<'env, 'b, T, U, V> FromEnvId<'env, JMethodID> for (T, U, V)
where
    T: FromEnvObject<'env, 'b, JClass<'env>>,
    U: Into<JNIString>,
    V: Into<JNIString>,
{
    fn lookup(self, env: &JNIEnv<'env>) -> Result<JMethodID> {
        env.get_method_id(self.0, self.1, self.2)
    }
}

impl<'env, 'b, T, Signature> FromEnvId<'env, JMethodID> for (T, Signature)
where
    T: FromEnvObject<'env, 'b, JClass<'env>>,
    Signature: Into<JNIString>,
{
    fn lookup(self, env: &JNIEnv<'env>) -> Result<JMethodID> {
        (self.0, "<init>", self.1).lookup(env)
    }
}

impl<'env, 'b, T, U, V> FromEnvId<'env, JStaticMethodID> for (T, U, V)
where
    T: FromEnvObject<'env, 'b, JClass<'env>>,
    U: Into<JNIString>,
    V: Into<JNIString>,
{
    fn lookup(self, env: &JNIEnv<'env>) -> Result<JStaticMethodID> {
        env.get_static_method_id(self.0, self.1, self.2)
    }
}