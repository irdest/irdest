use jni::{
    objects::{JClass, JObject, JString, JValue},
    JNIEnv,
};

#[no_mangle]
pub unsafe extern "C" fn Java_irde_st_app_ffi_RatmanNative_connect<'a>(
    jni_env: &'a JNIEnv,
    j_obj: JObject,
    socket: JString,
    socket_addr: JString,
) {
    let skt = conv_jstring(jni_env, socket);
    let skt_addr = JavaId::from_obj(jni_env, j_obj).into_identity();
    ratman_client::RatmanIpc::connect(&skt, Some(skt_addr));
}

#[no_mangle]
pub unsafe extern "C" fn Java_irde_st_app_ffi_RatmanNative_anonymous<'a>(
    jni_env: &'a JNIEnv,
    j_obj: JObject,
    socket_addr: JString,
) {
    let skt_addr = conv_jstring(jni_env, socket_addr);
    ratman_client::RatmanIpc::anonymous(&skt_addr);
}

#[no_mangle]
pub unsafe extern "C" fn Java_irde_st_app_ffi_RatmanNative_default_connect_with_addr<'a>(
    jni_env: &'a JNIEnv,
    j_obj: JObject,
    addr: JString,
) {
    ratman_client::RatmanIpc::default_with_addr(JavaId::from_obj(jni_env, j_obj).into_identity());
}
