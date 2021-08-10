//! irdest-core users module

use super::ToJObject;
use crate::{
    error::Result,
    helpers::ItemDiff,
    users::{UserAuth, UserUpdate},
    Identity, Irdest,
};

use async_std::task::block_on;
use jni::{
    objects::{JClass, JList, JObject, JString},
    sys::jboolean,
    JNIEnv,
};
use log::info;
use std::sync::Arc;

#[no_mangle]
pub unsafe extern "C" fn create(
    this: &JNIEnv,
    q: Arc<Irdest>,
    handle: JString,
    name: JString,
    pw: JString,
) -> Result<UserAuth> {
    let handle = super::conv_jstring(this, handle);
    let name = super::conv_jstring(this, name);
    let pw = super::conv_jstring(this, pw);
    let auth = block_on(async { q.users().create(&pw).await })?;

    // Update the user handle and display name
    let update = UserUpdate {
        handle: ItemDiff::Set(handle),
        display_name: ItemDiff::Set(name),
        ..Default::default()
    };

    block_on(async { q.users().update(auth.clone(), update).await })?;

    Ok(auth)
}

#[no_mangle]
pub unsafe extern "C" fn login(
    env: &JNIEnv,
    q: Arc<Irdest>,
    id: Identity,
    pw: JString,
) -> Result<UserAuth> {
    let pw = super::conv_jstring(env, pw);
    block_on(async { q.users().login(id, &pw).await })
}

pub fn list<'env>(local: jboolean, env: &'env JNIEnv<'env>, q: Arc<Irdest>) -> JList<'env, 'env> {
    let users = block_on(async {
        if local != 0 {
            // a jboolean false == 0
            q.users().list().await
        } else {
            q.users().list_remote().await
        }
    });

    let class = env.find_class("java/util/ArrayList").unwrap();
    let arraylist = env.new_object(class, "()V", &[]).unwrap();
    let list = JList::from_env(env, arraylist).unwrap();

    users
        .into_iter()
        .map(|user| user.to_jobject(&env))
        .fold(list, |list, jobj| {
            list.add(jobj);
            list
        })
}

pub fn get<'env>(env: &'env JNIEnv<'env>, q: Arc<Irdest>, id: Identity) -> JObject<'env> {
    match block_on(async { q.users().get(id).await }) {
        Ok(u) => u.to_jobject(&env),
        Err(_) => JObject::null(),
    }
}
