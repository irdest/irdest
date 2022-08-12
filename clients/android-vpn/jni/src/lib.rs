use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use std::error::Error;

use ratman::daemon;

async fn ratrun() {
    let configuration = daemon::config::Config::default();
    let m = daemon::startup::build_cli();

    daemon::startup::run_app(m, configuration);
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Java_org_irdest_IrdestVPN_Ratmand_ratrun(
    env: JNIEnv,
    _: JClass,
    _test_string: JString,
) -> jstring {
    // Ignoring the test_string which comes from the android application.

    ratrun();

    env.new_string("Testing is running 🐲")
        .expect("Error: can't not make java string!")
        .into_inner()
}
