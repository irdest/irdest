use crate::daemon;
use jni::{
    objects::{JClass, JObject, JString, JValue},
    JNIEnv,
};
use ratman_client::ffi;

#[no_mangle]
pub unsafe extern "C" fn Java_irde_st_app_ffi_RatmanNative_initRatman<'a>(
    jni_env: &'a JNIEnv,
    j_obj: JObject,
) {
    daemon::init_daemon();
}

#[no_mangle]
pub unsafe extern "C" fn Java_irde_st_app_ffi_RatmanNative_setupLogging<'a>(
    jni_env: &'a JNIEnv,
    j_obj: JObject,
    level: JString,
    syslog: jni::sys::jboolean,
) {
    let lv_str = ffi::conv_jstring(jni_env, level);
    let slog_bool = (syslog > 0);
    daemon::setup_logging(&lv_str, slog_bool)
}
