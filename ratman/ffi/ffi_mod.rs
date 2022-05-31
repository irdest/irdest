use jni::{
    objects::{JString, JValue, Jclass, Jobject},
    sys::jboolean,
    JNIEnv,
};
use std::ffi::{CStr, CString};

/// Convert Java Strings to Rust
pub(self) fn jstr_to_rs(env: &JNIEnv, sample_str: JString) {
    CString::from(unsafe { CStr::from_ptr(env.get_string(sample_str).unwrap().as_ptr()) })
        .to_str()
        .unwrap()
        .into()
}
