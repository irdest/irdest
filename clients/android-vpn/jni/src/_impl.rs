use android_logger::Config;
use async_std;
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use log::{debug, info, Level};
use std::error::Error;

// use netmod_mem::MemMod;
// use ratman::Router; // FIXME

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Java_org_irdest_IrdestVPN_Ratmand_receiveLog(env: JNIEnv, _: JClass) {
    android_logger::init_once(
        Config::default()
            .with_tag("ratmand-android-logger")
            .with_min_level(Level::Trace),
    );
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Java_org_irdest_IrdestVPN_RatmandRunnable_runRouter(env: JNIEnv, _: JClass) {
    // let mut cfg = daemon::config::Config::new();
    // cfg.accept_unknown_peers = true;
    // info!("@android-dev#: config => {:?}", cfg);

    // FIXME !!!

    // Run ratmand.
    // async_std::task::block_on(ratmand::start_with_configuration(cfg)).unwrap();
}

// Run ratman for the simple android test.
async fn router_testing() -> Result<(), Box<dyn Error>> {
    // Build a simple channel in memory
    // let mm1 = MemMod::new();

    // FIXME !!!

    // // Initialise one router
    // let r1 = Router::new();

    // // Add channel endpoint to router
    // r1.add_endpoint(mm1).await;

    // // Create a user and add them to the router
    // let u1 = r1.add_user().await?;

    // // And mark router "online"
    // r1.online(u1).await?;

    // // The routers will now start announcing their new users on the
    // // micro-network.  You can now poll for new user discoveries.
    // r1.discover().await;

    // This test needs two android devices that are connected
    // via Wifi-Direct.
    // device1 needs to install .apk [android-vpn app]
    // which contains this library.[libratman_android.so]
    //
    // Device2 should be able to register ratcat to the r1(router)
    // via termux or adb by ./ratcat --register
    // * expected output on the device2:
    // $ Registered address: [...]
    // $ Registered a new address!  You may now run `ratcat` to send data
    Ok(())
}
