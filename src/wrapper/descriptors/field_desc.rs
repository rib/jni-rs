use crate::{
    descriptors::FromEnvId,
    errors::*,
    objects::{JClass, JFieldID, JStaticFieldID},
    strings::JNIString,
    JNIEnv,
};

use super::FromEnvObject;

impl<'env, 'b, T, U, V> FromEnvId<'env, JFieldID> for (T, U, V)
where
    T: FromEnvObject<'env, 'b, JClass<'env>>,
    U: Into<JNIString>,
    V: Into<JNIString>,
{
    fn lookup(self, env: &JNIEnv<'env>) -> Result<JFieldID> {
        env.get_field_id(self.0, self.1, self.2)
    }
}

impl<'env, 'b, T, U, V> FromEnvId<'env, JStaticFieldID> for (T, U, V)
where
    T: FromEnvObject<'env, 'b, JClass<'env>>,
    U: Into<JNIString>,
    V: Into<JNIString>,
{
    fn lookup(self, env: &JNIEnv<'env>) -> Result<JStaticFieldID> {
        env.get_static_field_id(self.0, self.1, self.2)
    }
}

