// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Java/JNI utilities.

use jni::errors::Error as JniError;
use jni::objects::{AutoLocal, GlobalRef, JObject};
use jni::sys::{jobject, jsize};
use jni::{AttachGuard, JNIEnv, JavaVM};
use std::os::raw::c_void;

/// Result returning JNI errors
pub type JniResult<T> = Result<T, JniError>;

/// Tries to get the `JNIEnv` structure. If we happen to execute in the context
/// of a Java thread, we just reuse it (`Auto`). If we are in the context of a
/// native thread, then we will attach it to JVM by calling `attach_current_thread`
/// and it will be automatically detached when it goes out of scope (`Manual`).
pub enum EnvGuard<'a> {
    /// Automatically obtained `JNIEnv`. We do not need to detach it.
    Auto(JNIEnv<'a>),
    /// `JNIEnv` obtained through `attach_current_thread`.
    /// It will be automatically detached from the current thread when it gets out
    /// of its scope.
    Manual(AttachGuard<'a>),
}

impl<'a> EnvGuard<'a> {
    /// Initialise `EnvGuard` out of a `JavaVM` reference.
    /// We also check if the reference is valid and return an error if it is not.
    pub fn new(vm: Option<&'a JavaVM>) -> JniResult<Self> {
        let vm = vm.ok_or_else(|| JniError::from("no JVM reference found"))?;
        Ok(match vm.get_env() {
            Ok(env) => EnvGuard::Auto(env),
            Err(_) => EnvGuard::Manual(vm.attach_current_thread()?),
        })
    }

    /// Return `JNIEnv` that we obtained.
    pub fn env(&self) -> &JNIEnv {
        match self {
            EnvGuard::Auto(env) => &env,
            EnvGuard::Manual(guard) => &*guard,
        }
    }
}

/// Unwraps the results and checks for Java exceptions or other errors.
/// Returns from the function call and passes the exception handling to
/// Java in case of an exception.
/// Required for exceptions pass-through (simplifies debugging).
#[macro_export]
macro_rules! jni_unwrap {
    ($res:expr) => {{
        let res: Result<_, JniError> = $res;
        match res {
            Ok(val) => val,
            Err(e) => {
                error!("{:?}", e);
                return;
            }
        }
    }};
}

/// Generates a `user_data` context containing a reference to a single or several Java callbacks
#[macro_export]
macro_rules! gen_ctx {
    ($env:ident, $cb:ident) => {
        {
            let ctx = jni_unwrap!($env.new_global_ref($cb));
            let ptr = *ctx.as_obj() as *mut c_void;
            mem::forget(ctx);
            ptr
        }
    };

    ($env:ident, $cb0:ident, $($cb_rest:ident),+ ) => {
        {
            let ctx = [
                Some(jni_unwrap!($env.new_global_ref($cb0))),
                $(
                    Some(jni_unwrap!($env.new_global_ref($cb_rest))),
                )+
            ];
            let ctx = Box::into_raw(Box::new(ctx)) as *mut c_void;
            ctx
        }
    }
}

/// Generates primitive type converters
#[macro_export]
macro_rules! gen_primitive_type_converter {
    ($native_type:ty, $java_type:ty) => {
        impl FromJava<$java_type> for $native_type {
            fn from_java(_env: &JNIEnv, input: $java_type) -> JniResult<Self> {
                Ok(input as Self)
            }
        }

        impl<'a> ToJava<'a, $java_type> for $native_type {
            fn to_java(&self, _env: &JNIEnv) -> JniResult<$java_type> {
                Ok(*self as $java_type)
            }
        }
    };
}

/// Generate a `ToJava` impl that converts a slice of structures (`&[Foo]`) into a Java object array (`Foo[]`).
#[macro_export]
macro_rules! gen_object_array_converter {
    ($class_loader:expr, $native_type:ident, $java_ty_name:expr) => {
        impl<'a, 'b> ToJava<'a, JObject<'a>> for &'b [$native_type] {
            fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
                unsafe {
                    object_array_to_java(
                        $class_loader,
                        $native_type::to_java,
                        self,
                        env,
                        $java_ty_name,
                    )
                }
            }
        }
    };
}

/// Generate a `ToJava` impl that converts a byte array (`[u8; 32]`) into a Java byte array (`byte[]`).
#[macro_export]
macro_rules! gen_byte_array_converter {
    ($arr_type:ty, $size:expr) => {
        impl<'a> FromJava<JObject<'a>> for [$arr_type; $size] {
            fn from_java(env: &JNIEnv, input: JObject) -> JniResult<Self> {
                let input = input.into_inner() as jbyteArray;
                let mut output = [0; $size];

                let len = env.get_array_length(input)? as usize;
                env.get_byte_array_region(input, 0, &mut output[0..cmp::min(len, $size)])?;

                Ok(unsafe { mem::transmute(output) })
            }
        }

        impl<'a> ToJava<'a, JObject<'a>> for [$arr_type; $size] {
            fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
                let output = env.new_byte_array(self.len() as jsize)?;
                env.set_byte_array_region(output, 0, unsafe {
                    slice::from_raw_parts(self.as_ptr() as *const i8, self.len())
                })?;
                Ok(JObject::from(output as jobject))
            }
        }
    };
}

/// Converts object arrays into Java arrays
pub unsafe fn object_array_to_java<'a, T, U: Into<JObject<'a>> + 'a>(
    class_loader: unsafe fn(&'a JNIEnv, &str) -> JniResult<AutoLocal<'a>>,
    transform_fn: fn(&T, &'a JNIEnv) -> JniResult<U>,
    list: &[T],
    env: &'a JNIEnv,
    class: &str,
) -> JniResult<JObject<'a>> {
    let cls = class_loader(env, class)?;
    let output = env.new_object_array(list.len() as jsize, &cls, JObject::null())?;

    for (idx, entry) in list.iter().enumerate() {
        let jentry = transform_fn(entry, env)?.into();
        env.set_object_array_element(output, idx as i32, jentry)?;
        env.delete_local_ref(jentry)?;
    }

    Ok(JObject::from(output))
}

/// Converts `user_data` back into a Java callback object
pub unsafe fn convert_cb_from_java(env: &JNIEnv, ctx: *mut c_void) -> JniResult<GlobalRef> {
    Ok(GlobalRef::from_raw(env.get_java_vm()?, ctx as jobject))
}
