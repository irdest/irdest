use jni::{
    objects::{JClass, JObject, JString, JValue},
    JNIEnv,
};
use std::ffi::{CStr, CString};
use types::{Address, Message};

pub(crate) fn into_jstring<'a>(env: &'a JNIEnv, s: String) -> JString<'a> {
    env.new_string(s).unwrap()
}

pub(crate) fn conv_jstring(env: &JNIEnv, s: JString) -> String {
    CString::from(unsafe { CStr::from_ptr(env.get_string(s).unwrap().as_ptr()) })
        .to_str()
        .unwrap()
        .into()
}

/// Create a jstring from an optional Rust string
pub(crate) fn to_jstring<'env>(env: &'env JNIEnv, s: Option<String>) -> JString<'env> {
    match s {
        Some(s) => env.new_string(s).unwrap(),
        None => JObject::null().into(),
    }
}

pub(crate) fn jval_to_jstring(val: JValue) -> JString {
    match val {
        JValue::Object(o) => o.into(),
        _ => unreachable!(),
    }
}

pub trait ToJObject {
    fn to_jobject<'env>(self, env: &'env JNIEnv) -> JObject<'env>;
}

pub(crate) struct JavaId(pub(crate) String);
// pub(crate) struct JRatmanIpc(pub(crate) ratman_client::RatmanIpc);

impl JavaId {
    pub(crate) fn from_obj(env: &JNIEnv, jobj: JObject) -> Self {
        let jval = env.get_field(jobj, "inner", "Ljava/lang/String;").unwrap();
        let jstring = jval_to_jstring(jval);
        let id = conv_jstring(env, jstring);
        Self(id.to_string())
    }

    pub(crate) fn into_obj<'a>(self, env: &'a JNIEnv) -> JObject<'a> {
        /// arg in `find_class` should point to the file in
        /// android application where Id is defined
        /// currently sample path provided
        let class: JClass<'a> = env.find_class("st/irde/app/ffi/models/Id").unwrap();
        env.new_object(
            class,
            "(Ljava/lang/String;)V",
            &[JValue::Object(into_jstring(env, self.0).into())],
        )
        .unwrap()
    }

    pub(crate) fn from_address(id: Address) -> Self {
        Self(id.to_string())
    }

    pub(crate) fn into_address(self) -> Address {
        Address::from_string(&self.0)
    }
}

// fn to_jpair<'env>(env: &'env JNIEnv, pair: Tuple) -> JObject<'env> {
// }

// fn message_to_jobject<'env>(env: &'env JNIEnv, message: Message) -> JObject<'env> {
//     let id = JavaId::from_identity(message.id).into_obj(env);
//     let sender_id = JavaId::from_identity(message.sender).into_obj(env);
//     let time_pair = jni::objects::JObject::from(message.time).into_inner();
// }

/// For logging the lib status in android device
/// maybe not needed if ratman does tracing internally
pub(crate) fn init_panic_handling_once() {
    use std::sync::Once;
    static INIT_BACKTRACES: Once = Once::new();
    INIT_BACKTRACES.call_once(move || {
        std::panic::set_hook(Box::new(move |panic_info| {
            let (file, line) = if let Some(loc) = panic_info.location() {
                (loc.file(), loc.line())
            } else {
                ("<unknown>", 0)
            };
            let reason = panic_info.to_string();

            let err = format!(
                "### Rust `panic!` hit at file '{}', line {}: `{}`",
                file, line, reason,
            );

            // android_logger::log(<stuff>, <err_msg>, <priority>)
        }));
    });
}
