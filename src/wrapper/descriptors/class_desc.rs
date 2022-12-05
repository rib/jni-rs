use crate::{
    descriptors::FromEnvObject,
    errors::*,
    objects::{AutoLocal, GlobalRef, JClass, JObject},
    strings::JNIString,
    JNIEnv,
};

use super::FromEnvValue;

impl<'env, 'b, T> FromEnvObject<'env, 'b, JClass<'env>> for T
where
    T: Into<JNIString>,
{
    fn lookup(self, env: &JNIEnv<'env>) -> Result<FromEnvValue<'env, 'b, JClass<'env>>> {
        let class_obj = env.find_class(self)?;
        Ok(FromEnvValue::Owned(env.auto_local(class_obj)))
    }
}

impl<'env, 'b> FromEnvObject<'env, 'b, JClass<'env>> for JObject<'env> {
    fn lookup(self, env: &JNIEnv<'env>) -> Result<FromEnvValue<'env, 'b, JClass<'env>>> {
        let class_obj = env.get_object_class(self)?;
        Ok(FromEnvValue::Owned(env.auto_local(class_obj)))
    }
}

/// This conversion assumes that the `GlobalRef` is a pointer to a class object.
impl<'env, 'b, 'c> FromEnvObject<'env, 'b, JClass<'env>> for &'c GlobalRef<JClass<'env>> {
    fn lookup(self, _: &JNIEnv<'env>) -> Result<FromEnvValue<'env, 'b, JClass<'env>>> {
        Ok(FromEnvValue::Reference(self.as_ref()))
    }
}

/// This conversion assumes that the `AutoLocal` is a pointer to a class object.
impl<'env, 'b, 'c> FromEnvObject<'env, 'b, JClass<'env>> for &'c AutoLocal<'env, 'b, JClass<'env>>
{
    fn lookup<'d>(self, _: &'d JNIEnv<'env>) -> Result<FromEnvValue<'env, 'b, JClass<'env>>> {
        Ok(FromEnvValue::Reference(self.as_ref()))
    }
}