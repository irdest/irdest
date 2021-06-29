//! Base android (java ffi) API
//!
//! This module provides some simple utilities for setting up the
//! irdest-core and router state, and adding new TCP routes to the driver.

use crate::utils::{self, GcWrapped};
use async_std::{
    sync::{Arc, RwLock},
    task::block_on,
};
use jni::{
    objects::{JClass, JObject, JString, JValue},
    sys::{jboolean, jint, jlong, jobject},
    JNIEnv,
};

use android_logger::{Config, FilterBuilder};
use irdest_core::{users::UserAuth, helpers::Directories, Irdest};
use log::Level;
use ratman::Router;
use ratman_netmod::{Frame, Target};

/// Setup the main database and router state
#[no_mangle]
pub unsafe extern "C" fn Java_st_irde_app_ffi_NativeIrdest_setupState(
    _: JNIEnv,
    _: JObject,
    port: jint,
) -> jlong {
    info!("Rust FFI setupState");
    println!("Setting up android logger and panic hook!");
    android_logger::init_once(
        Config::default()
            .with_tag("rust")
            .with_min_level(Level::Debug),
    );
    utils::init_panic_handling_once();

    // tracing::subscriber::set_global_default(subscriber).unwrap();
    // let subscriber = android_tracing::AndroidSubscriber::new(true)
    //     .with(EnvFilter::new("android_support=trace,[]=warn"));

    info!("Running ratman-configure and irdest-core bootstrap code...");

    let router = Router::new();

    let tcp = block_on(async {
        use netmod_tcp::Endpoint;
        let ep = Endpoint::new("0.0.0.0", port as u16, "qauld", netmod_tcp::Mode::Static)
            .await
            .unwrap();
        router.add_endpoint(Arc::clone(&ep)).await;
        ep
    });

    let wd = block_on(async {
        use netmod_wd::WdMod;
        let ep = WdMod::new();
        router.add_endpoint(Arc::clone(&ep)).await;
        ep
    });
    info!("Router init...done");

    let irdest = Irdest::new(router, Directories::android("st.irde.app"));
    info!("lib-irdest init...done");

    // Uncomment it and then fix, there is some referencing error, most probably we're using incorrect import of `UserUpdate` here
    // block_on(async {
    //     use irdest_core::users::UserUpdate;
    //     let auth = irdest_core::java::ffi::users::create("1234").await.unwrap();
    //     irdest
    //         .users()
    //         .update(
    //             auth.clone(),
    //             UserUpdate::RealName(Some("Alice Anonymous".into())),
    //         )
    //         .await;
    //     irdest
    //         .users()
    //         .update(auth.clone(), UserUpdate::DisplayName(Some("alice".into())))
    //         .await;
    // });

    info!("Service init: done");

    // We just return the state pointer here because for some reason
    // storing the state directly in the instance variable doesn't
    // work, or didn't work when I last tried it.  Patches to change
    // this very welcome, if they work!
    GcWrapped::new(tcp, wd, irdest).into_ptr()
}

/// Check if an auth token is still valid
#[no_mangle]
pub unsafe extern "C" fn Java_st_irde_app_ffi_NativeIrdest_checkLogin(
    _: JNIEnv,
    _: JObject,
    irdest: jlong,
) -> jboolean {
    info!("Rust FFI checkLogin");
    let state = GcWrapped::from_ptr(irdest as i64);
    match state.get_auth() {
        None => false,
        Some(auth) => block_on(async {
            let w = state.get_inner();
            w.irdest()
                .users()
                .is_authenticated(auth)
                .await
                .map(|_| true)
                .unwrap_or(false)
        }),
    }
    .into()
}

/// Check if an auth token is still valid
#[no_mangle]
pub unsafe extern "C" fn Java_st_irde_app_ffi_NativeIrdest_connectTcp(
    env: JNIEnv,
    _: JObject,
    irdest: jlong,
    addr: JString,
    port: jint,
) {
    info!("Rust FFI connectTcp");
    let state = GcWrapped::from_ptr(irdest as i64);
    let tcp = state.get_tcp();

    let addr: Vec<u8> = utils::conv_jstring(&env, addr)
        .split(".")
        .map(|s| s.parse().unwrap())
        .collect();
    let port = port as u16;

    // block_on(async {
    //     use std::net::{Ipv4Addr, SocketAddrV4};
    //     tcp.load_peers(vec![SocketAddrV4::new(
    //         Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]),
    //         port,
    //     )])
    //     .await;
    // });
}

fn frame_to_jframe<'env>(env: &'env JNIEnv, f: Frame, t: Target) -> JObject<'env> {
    let vec = bincode::serialize(&f).unwrap();
    let array = env.new_byte_array(vec.len() as i32).unwrap();

    env.set_byte_array_region(
        array,
        0,
        vec.into_iter()
            .map(|u| u as i8)
            .collect::<Vec<_>>()
            .as_slice(),
    )
    .unwrap();

    let target: i32 = match t {
        Target::Flood => -1,
        Target::Single(id) => id as i32,
    };

    let class: JClass<'env> = env.find_class("st/irde/app/ffi/models/Frame").unwrap();
    env.new_object(
        class,
        "([BI)V",
        &[JValue::Object(array.into()), JValue::Int(target)],
    )
    .unwrap()
}

fn from_jframe<'env>(env: &'env JNIEnv, jframe: JObject<'env>) -> (Frame, Target) {
    let bytes = match env.get_field(jframe, "data", "[B").unwrap() {
        JValue::Object(o) => o,
        _ => unreachable!(),
    };
    let target = env.get_field(jframe, "target", "I").unwrap();

    let len = env.get_array_length(*bytes).unwrap() as usize;
    let mut buf: Vec<i8> = (0..).take(len).map(|_| 0_i8).collect();
    env.get_byte_array_region(*bytes, 0, buf.as_mut_slice())
        .unwrap();

    (
        bincode::deserialize(&buf.into_iter().map(|u| u as u8).collect::<Vec<u8>>()).unwrap(),
        match target {
            JValue::Int(-1) => Target::Flood,
            JValue::Int(id) => Target::Single(id as u16),
            _ => unreachable!(),
        },
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_st_irde_app_ffi_NativeIrdest_wdToSend(
    env: JNIEnv,
    _: JObject,
    irdest: jlong,
) -> jobject {
    info!("Rust FFI wdToSend");
    let state = GcWrapped::from_ptr(irdest as i64);
    let wd = state.get_wd();

    // Blocks until a new frame should be sent
    let (frame, target) = wd.take();

    // Convert to Java types
    frame_to_jframe(&env, frame, target).into_inner()
}

#[no_mangle]
pub unsafe extern "C" fn Java_st_irde_app_ffi_NativeIrdest_wdReceived(
    env: JNIEnv,
    _: JObject,
    irdest: jlong,
    frame: JObject,
) {
    info!("Rust FFI wdReceived");
    let state = GcWrapped::from_ptr(irdest as i64);
    let wd = state.get_wd();
    let (f, t) = from_jframe(&env, frame);

    // Pass the frame to the router - not our problem anymore :)
    wd.give(f, t);
}
