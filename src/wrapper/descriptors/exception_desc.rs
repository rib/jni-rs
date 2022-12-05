use crate::{
    descriptors::FromEnvObject,
    errors::*,
    objects::{JClass, JObject, JThrowable, JValue},
    strings::JNIString,
    JNIEnv,
};

use super::FromEnvValue;

const DEFAULT_EXCEPTION_CLASS: &str = "java/lang/RuntimeException";

impl<'env, 'b, C, M> FromEnvObject<'env, 'b, JThrowable<'env>> for (C, M)
where
    C: FromEnvObject<'env, 'b, JClass<'env>>,
    M: Into<JNIString>,
{
    fn lookup(self, env: &JNIEnv<'env>) -> Result<FromEnvValue<'env, 'b, JThrowable<'env>>> {
        let jmsg: JObject = env.new_string(self.1)?.into();
        let obj: JThrowable = env
            .new_object(self.0, "(Ljava/lang/String;)V", &[JValue::from(&jmsg)])?
            .into();
        Ok(FromEnvValue::Owned(env.auto_local(obj)))
    }
}

impl<'env, 'b> FromEnvObject<'env, 'b, JThrowable<'env>> for Exception {
    fn lookup(self, env: &JNIEnv<'env>) -> Result<FromEnvValue<'env, 'b, JThrowable<'env>>> {
        (self.class, self.msg).lookup(env)
    }
}

impl<'env, 'b, 'c> FromEnvObject<'env, 'b, JThrowable<'env>> for &'c str {
    fn lookup(self, env: &JNIEnv<'env>) -> Result<FromEnvValue<'env, 'b, JThrowable<'env>>> {
        (DEFAULT_EXCEPTION_CLASS, self).lookup(env)
    }
}

impl<'env, 'b> FromEnvObject<'env, 'b, JThrowable<'env>> for String {
    fn lookup(self, env: &JNIEnv<'env>) -> Result<FromEnvValue<'env, 'b, JThrowable<'env>>> {
        (DEFAULT_EXCEPTION_CLASS, self).lookup(env)
    }
}

impl<'env, 'b> FromEnvObject<'env, 'b, JThrowable<'env>> for JNIString {
    fn lookup(self, env: &JNIEnv<'env>) -> Result<FromEnvValue<'env, 'b, JThrowable<'env>>> {
        (DEFAULT_EXCEPTION_CLASS, self).lookup(env)
    }
}
