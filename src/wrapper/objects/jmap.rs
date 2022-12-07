use crate::{
    errors::*,
    objects::{AutoLocal, JMethodID, JObject, JValue},
    signature::{Primitive, ReturnType},
    JNIEnv,
};

use super::JClass;

/// Wrapper for JObjects that implement `java/util/Map`. Provides methods to get
/// and set entries and a way to iterate over key/value pairs.
///
/// Looks up the class and method ids on creation rather than for every method
/// call.
pub struct JMap<'a: 'b, 'b> {
    internal: &'b JObject<'a>,
    class: AutoLocal<'a, 'b, JClass<'a>>,
    get: JMethodID,
    put: JMethodID,
    remove: JMethodID,
    env: &'b JNIEnv<'a>,
}

impl<'a: 'b, 'b> ::std::ops::Deref for JMap<'a, 'b> {
    type Target = JObject<'a>;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl<'a: 'b, 'b> From<JMap<'a, 'b>> for &'b JObject<'a> {
    fn from(other: JMap<'a, 'b>) -> &'b JObject<'a> {
        other.internal
    }
}

impl<'a: 'b, 'b> JMap<'a, 'b> {
    /// Create a map from the environment and an object. This looks up the
    /// necessary class and method ids to call all of the methods on it so that
    /// exra work doesn't need to be done on every method call.
    pub fn from_env(env: &'b JNIEnv<'a>, obj: &'b JObject<'a>) -> Result<JMap<'a, 'b>> {
        let class = env.auto_local(env.find_class("java/util/Map")?);

        let get = env.get_method_id(&class, "get", "(Ljava/lang/Object;)Ljava/lang/Object;")?;
        let put = env.get_method_id(
            &class,
            "put",
            "(Ljava/lang/Object;Ljava/lang/Object;\
             )Ljava/lang/Object;",
        )?;

        let remove =
            env.get_method_id(&class, "remove", "(Ljava/lang/Object;)Ljava/lang/Object;")?;

        Ok(JMap {
            internal: obj,
            class,
            get,
            put,
            remove,
            env,
        })
    }

    /// Look up the value for a key. Returns `Some` if it's found and `None` if
    /// a null pointer would be returned.
    pub fn get(&self, key: &'a JObject<'a>) -> Result<Option<JObject<'a>>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a JObject/null, rather than another primitive type.
        let result = unsafe {
            self.env.call_method_unchecked(
                self.internal,
                self.get,
                ReturnType::Object,
                &[JValue::from(key).to_jni()],
            )
        };

        match result {
            Ok(val) => Ok(Some(val.l()?)),
            Err(e) => match e {
                Error::NullPtr(_) => Ok(None),
                _ => Err(e),
            },
        }
    }

    /// Look up the value for a key. Returns `Some` with the old value if the
    /// key already existed and `None` if it's a new key.
    pub fn put(&self, key: &'a JObject<'a>, value: &'a JObject<'a>) -> Result<Option<JObject<'a>>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a JObject/null, rather than another primitive type.
        let result = unsafe {
            self.env.call_method_unchecked(
                self.internal,
                self.put,
                ReturnType::Object,
                &[JValue::from(key).to_jni(), JValue::from(value).to_jni()],
            )
        };

        match result {
            Ok(val) => Ok(Some(val.l()?)),
            Err(e) => match e {
                Error::NullPtr(_) => Ok(None),
                _ => Err(e),
            },
        }
    }

    /// Remove a value from the map. Returns `Some` with the removed value and
    /// `None` if there was no value for the key.
    pub fn remove(&self, key: &'a JObject<'a>) -> Result<Option<JObject<'a>>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a JObject/null, rather than another primitive type.
        let result = unsafe {
            self.env.call_method_unchecked(
                self.internal,
                self.remove,
                ReturnType::Object,
                &[JValue::from(key).to_jni()],
            )
        };

        match result {
            Ok(val) => Ok(Some(val.l()?)),
            Err(e) => match e {
                Error::NullPtr(_) => Ok(None),
                _ => Err(e),
            },
        }
    }

    /// Get key/value iterator for the map. This is done by getting the
    /// `EntrySet` from java and iterating over it.
    pub fn iter(&self) -> Result<JMapIter<'a, 'b, '_>> {
        let iter_class = self
            .env
            .auto_local(self.env.find_class("java/util/Iterator")?);

        let has_next = self.env.get_method_id(&iter_class, "hasNext", "()Z")?;

        let next = self
            .env
            .get_method_id(&iter_class, "next", "()Ljava/lang/Object;")?;

        let entry_class = self
            .env
            .auto_local(self.env.find_class("java/util/Map$Entry")?);

        let get_key = self
            .env
            .get_method_id(&entry_class, "getKey", "()Ljava/lang/Object;")?;

        let get_value = self
            .env
            .get_method_id(&entry_class, "getValue", "()Ljava/lang/Object;")?;

        // Get the iterator over Map entries.
        // Use the local frame till #109 is resolved, so that implicitly looked-up
        // classes are freed promptly.
        let iter = self.env.with_local_frame(16, || {
            // SAFETY: We keep the class loaded, and fetched the method ID for this function. Arg list is known empty.
            let entry_set = unsafe {
                self.env.call_method_unchecked(
                    self.internal,
                    (&self.class, "entrySet", "()Ljava/util/Set;"),
                    ReturnType::Object,
                    &[],
                )
            }?
            .l()?;

            // SAFETY: We keep the class loaded, and fetched the method ID for this function. Arg list is known empty.
            let iter = unsafe {
                self.env.call_method_unchecked(
                    &entry_set,
                    ("java/util/Set", "iterator", "()Ljava/util/Iterator;"),
                    ReturnType::Object,
                    &[],
                )
            }?
            .l()?;

            Ok(iter)
        })?;
        let iter = self.env.auto_local(iter);

        Ok(JMapIter {
            map: self,
            has_next,
            next,
            get_key,
            get_value,
            iter,
        })
    }
}

/// An iterator over the keys and values in a map.
///
/// TODO: make the iterator implementation for java iterators its own thing
/// and generic enough to use elsewhere.
pub struct JMapIter<'a, 'b, 'c> {
    map: &'c JMap<'a, 'b>,
    has_next: JMethodID,
    next: JMethodID,
    get_key: JMethodID,
    get_value: JMethodID,
    iter: AutoLocal<'a, 'b, JObject<'a>>,
}

impl<'a: 'b, 'b: 'c, 'c> JMapIter<'a, 'b, 'c> {
    fn get_next(&self) -> Result<Option<(JObject<'a>, JObject<'a>)>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for these functions. We know none expect args.

        let iter = self.iter.as_ref();
        let has_next = unsafe {
            self.map.env.call_method_unchecked(
                iter,
                self.has_next,
                ReturnType::Primitive(Primitive::Boolean),
                &[],
            )
        }?
        .z()?;

        if !has_next {
            return Ok(None);
        }

        let next = unsafe {
            self.map
                .env
                .call_method_unchecked(iter, self.next, ReturnType::Object, &[])
        }?
        .l()?;
        // Since this local reference isn't being returned to the caller we need to
        // make sure it gets deleted
        let next = self.map.env.auto_local(next);

        let key = unsafe {
            self.map
                .env
                .call_method_unchecked(&next, self.get_key, ReturnType::Object, &[])
        }?
        .l()?;

        let value = unsafe {
            self.map
                .env
                .call_method_unchecked(&next, self.get_value, ReturnType::Object, &[])
        }?
        .l()?;

        Ok(Some((key, value)))
    }
}

impl<'a: 'b, 'b: 'c, 'c> Iterator for JMapIter<'a, 'b, 'c> {
    type Item = (JObject<'a>, JObject<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.get_next() {
            Ok(Some(n)) => Some(n),
            _ => None,
        }
    }
}
