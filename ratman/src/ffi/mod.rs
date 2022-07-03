use jni::{
    objects::{JClass, JObject, JString, JValue},
    JNIEnv,
};

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
    syslog: bool,
) {
    let lv_str = ratman_client::ffi::conv_jstring(jni_env, level).to_string();
    let slog_bool = conv_jbool(jni_env, syslog);
    daemon::setup_logging(lv_str, syslog)
}
