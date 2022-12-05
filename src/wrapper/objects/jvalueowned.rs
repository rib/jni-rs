use std::convert::TryFrom;

use log::trace;

use crate::{errors::*, objects::JObject, signature::Primitive, sys::*};

/// Rusty version of the JNI C `jvalue` enum. Used in Java method call arguments
/// and returns.
#[allow(missing_docs)]
#[derive(Debug)]
pub enum JValueOwned<'a> {
    Object(JObject<'a>),
    Byte(jbyte),
    Char(jchar),
    Short(jshort),
    Int(jint),
    Long(jlong),
    Bool(jboolean),
    Float(jfloat),
    Double(jdouble),
    Void,
}

impl<'a> From<JValueOwned<'a>> for jvalue {
    fn from(other: JValueOwned) -> jvalue {
        other.to_jni()
    }
}

impl<'a> JValueOwned<'a> {
    /// Convert the enum to its jni-compatible equivalent.
    pub fn to_jni(self) -> jvalue {
        let val: jvalue = match self {
            JValueOwned::Object(obj) => jvalue { l: obj.internal },
            JValueOwned::Byte(byte) => jvalue { b: byte },
            JValueOwned::Char(char) => jvalue { c: char },
            JValueOwned::Short(short) => jvalue { s: short },
            JValueOwned::Int(int) => jvalue { i: int },
            JValueOwned::Long(long) => jvalue { j: long },
            JValueOwned::Bool(boolean) => jvalue { b: boolean as i8 },
            JValueOwned::Float(float) => jvalue { f: float },
            JValueOwned::Double(double) => jvalue { d: double },
            JValueOwned::Void => jvalue {
                l: ::std::ptr::null_mut(),
            },
        };
        trace!("converted {:?} to jvalue {:?}", self, unsafe {
            ::std::mem::transmute::<_, u64>(val)
        });
        val
    }

    /// Get the type name for the enum variant.
    pub fn type_name(&self) -> &'static str {
        match *self {
            JValueOwned::Void => "void",
            JValueOwned::Object(_) => "object",
            JValueOwned::Byte(_) => "byte",
            JValueOwned::Char(_) => "char",
            JValueOwned::Short(_) => "short",
            JValueOwned::Int(_) => "int",
            JValueOwned::Long(_) => "long",
            JValueOwned::Bool(_) => "bool",
            JValueOwned::Float(_) => "float",
            JValueOwned::Double(_) => "double",
        }
    }

    /// Get the primitive type for the enum variant. If it's not a primitive
    /// (i.e. an Object), returns None.
    pub fn primitive_type(&self) -> Option<Primitive> {
        Some(match *self {
            JValueOwned::Object(_) => return None,
            JValueOwned::Void => Primitive::Void,
            JValueOwned::Byte(_) => Primitive::Byte,
            JValueOwned::Char(_) => Primitive::Char,
            JValueOwned::Short(_) => Primitive::Short,
            JValueOwned::Int(_) => Primitive::Int,
            JValueOwned::Long(_) => Primitive::Long,
            JValueOwned::Bool(_) => Primitive::Boolean,
            JValueOwned::Float(_) => Primitive::Float,
            JValueOwned::Double(_) => Primitive::Double,
        })
    }

    /// Try to unwrap to an Object.
    pub fn l(self) -> Result<JObject<'a>> {
        match self {
            JValueOwned::Object(obj) => Ok(obj),
            _ => Err(Error::WrongJValueType("object", self.type_name())),
        }
    }

    /// Try to unwrap to a boolean.
    pub fn z(self) -> Result<bool> {
        match self {
            JValueOwned::Bool(b) => Ok(b == JNI_TRUE),
            _ => Err(Error::WrongJValueType("bool", self.type_name())),
        }
    }

    /// Try to unwrap to a byte.
    pub fn b(self) -> Result<jbyte> {
        match self {
            JValueOwned::Byte(b) => Ok(b),
            _ => Err(Error::WrongJValueType("jbyte", self.type_name())),
        }
    }

    /// Try to unwrap to a char.
    pub fn c(self) -> Result<jchar> {
        match self {
            JValueOwned::Char(b) => Ok(b),
            _ => Err(Error::WrongJValueType("jchar", self.type_name())),
        }
    }

    /// Try to unwrap to a double.
    pub fn d(self) -> Result<jdouble> {
        match self {
            JValueOwned::Double(b) => Ok(b),
            _ => Err(Error::WrongJValueType("jdouble", self.type_name())),
        }
    }

    /// Try to unwrap to a float.
    pub fn f(self) -> Result<jfloat> {
        match self {
            JValueOwned::Float(b) => Ok(b),
            _ => Err(Error::WrongJValueType("jfloat", self.type_name())),
        }
    }

    /// Try to unwrap to an int.
    pub fn i(self) -> Result<jint> {
        match self {
            JValueOwned::Int(b) => Ok(b),
            _ => Err(Error::WrongJValueType("jint", self.type_name())),
        }
    }

    /// Try to unwrap to a long.
    pub fn j(self) -> Result<jlong> {
        match self {
            JValueOwned::Long(b) => Ok(b),
            _ => Err(Error::WrongJValueType("jlong", self.type_name())),
        }
    }

    /// Try to unwrap to a short.
    pub fn s(self) -> Result<jshort> {
        match self {
            JValueOwned::Short(b) => Ok(b),
            _ => Err(Error::WrongJValueType("jshort", self.type_name())),
        }
    }

    /// Try to unwrap to a void.
    pub fn v(self) -> Result<()> {
        match self {
            JValueOwned::Void => Ok(()),
            _ => Err(Error::WrongJValueType("void", self.type_name())),
        }
    }
}

impl<'a, T: Into<JObject<'a>>> From<T> for JValueOwned<'a> {
    fn from(other: T) -> Self {
        JValueOwned::Object(other.into())
    }
}

impl<'a> TryFrom<JValueOwned<'a>> for JObject<'a> {
    type Error = Error;

    fn try_from(value: JValueOwned<'a>) -> Result<Self> {
        match value {
            JValueOwned::Object(o) => Ok(o),
            _ => Err(Error::WrongJValueType("object", value.type_name())),
        }
    }
}

impl<'a> From<bool> for JValueOwned<'a> {
    fn from(other: bool) -> Self {
        JValueOwned::Bool(if other { JNI_TRUE } else { JNI_FALSE })
    }
}

// jbool
impl<'a> From<jboolean> for JValueOwned<'a> {
    fn from(other: jboolean) -> Self {
        JValueOwned::Bool(other)
    }
}

impl<'a> TryFrom<JValueOwned<'a>> for jboolean {
    type Error = Error;

    fn try_from(value: JValueOwned<'a>) -> Result<Self> {
        match value {
            JValueOwned::Bool(b) => Ok(b),
            _ => Err(Error::WrongJValueType("bool", value.type_name())),
        }
    }
}

// jchar
impl<'a> From<jchar> for JValueOwned<'a> {
    fn from(other: jchar) -> Self {
        JValueOwned::Char(other)
    }
}

impl<'a> TryFrom<JValueOwned<'a>> for jchar {
    type Error = Error;

    fn try_from(value: JValueOwned<'a>) -> Result<Self> {
        match value {
            JValueOwned::Char(c) => Ok(c),
            _ => Err(Error::WrongJValueType("char", value.type_name())),
        }
    }
}

// jshort
impl<'a> From<jshort> for JValueOwned<'a> {
    fn from(other: jshort) -> Self {
        JValueOwned::Short(other)
    }
}

impl<'a> TryFrom<JValueOwned<'a>> for jshort {
    type Error = Error;

    fn try_from(value: JValueOwned<'a>) -> Result<Self> {
        match value {
            JValueOwned::Short(s) => Ok(s),
            _ => Err(Error::WrongJValueType("short", value.type_name())),
        }
    }
}

// jfloat
impl<'a> From<jfloat> for JValueOwned<'a> {
    fn from(other: jfloat) -> Self {
        JValueOwned::Float(other)
    }
}

impl<'a> TryFrom<JValueOwned<'a>> for jfloat {
    type Error = Error;

    fn try_from(value: JValueOwned<'a>) -> Result<Self> {
        match value {
            JValueOwned::Float(f) => Ok(f),
            _ => Err(Error::WrongJValueType("float", value.type_name())),
        }
    }
}

// jdouble
impl<'a> From<jdouble> for JValueOwned<'a> {
    fn from(other: jdouble) -> Self {
        JValueOwned::Double(other)
    }
}

impl<'a> TryFrom<JValueOwned<'a>> for jdouble {
    type Error = Error;

    fn try_from(value: JValueOwned<'a>) -> Result<Self> {
        match value {
            JValueOwned::Double(d) => Ok(d),
            _ => Err(Error::WrongJValueType("double", value.type_name())),
        }
    }
}

// jint
impl<'a> From<jint> for JValueOwned<'a> {
    fn from(other: jint) -> Self {
        JValueOwned::Int(other)
    }
}

impl<'a> TryFrom<JValueOwned<'a>> for jint {
    type Error = Error;

    fn try_from(value: JValueOwned<'a>) -> Result<Self> {
        match value {
            JValueOwned::Int(i) => Ok(i),
            _ => Err(Error::WrongJValueType("int", value.type_name())),
        }
    }
}

// jlong
impl<'a> From<jlong> for JValueOwned<'a> {
    fn from(other: jlong) -> Self {
        JValueOwned::Long(other)
    }
}

impl<'a> TryFrom<JValueOwned<'a>> for jlong {
    type Error = Error;

    fn try_from(value: JValueOwned<'a>) -> Result<Self> {
        match value {
            JValueOwned::Long(l) => Ok(l),
            _ => Err(Error::WrongJValueType("long", value.type_name())),
        }
    }
}

// jbyte
impl<'a> From<jbyte> for JValueOwned<'a> {
    fn from(other: jbyte) -> Self {
        JValueOwned::Byte(other)
    }
}

impl<'a> TryFrom<JValueOwned<'a>> for jbyte {
    type Error = Error;

    fn try_from(value: JValueOwned<'a>) -> Result<Self> {
        match value {
            JValueOwned::Byte(b) => Ok(b),
            _ => Err(Error::WrongJValueType("byte", value.type_name())),
        }
    }
}

// jvoid
impl<'a> From<()> for JValueOwned<'a> {
    fn from(_: ()) -> Self {
        JValueOwned::Void
    }
}

impl<'a> TryFrom<JValueOwned<'a>> for () {
    type Error = Error;

    fn try_from(value: JValueOwned<'a>) -> Result<Self> {
        match value {
            JValueOwned::Void => Ok(()),
            _ => Err(Error::WrongJValueType("void", value.type_name())),
        }
    }
}
