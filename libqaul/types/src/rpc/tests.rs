#[cfg(test)]
use super::*;

use crate::diff::ItemDiff;
use crate::users::{UserAuth, UserUpdate};

#[test]
fn create_user() {
    let cap = Capabilities::Users(UserCapabilities::Create {
        pw: "car horse battery staple".into(),
    });

    let json = cap.to_json();
    let cap2 = Capabilities::from_json(&json).unwrap();

    assert_eq!(cap, cap2);
    println!("{}", json);
}

#[test]
fn list_users() {
    let cap = Capabilities::Users(UserCapabilities::List);
    let json = cap.to_json();

    let cap2 = Capabilities::from_json(&json).unwrap();

    assert_eq!(cap, cap2);
    println!("{}", json);
}

#[test]
fn user_update() {
    let auth = UserAuth(Identity::random(), "<invalid>".into());

    let cap = Capabilities::Users(UserCapabilities::Update {
        auth,
        update: UserUpdate {
            handle: ItemDiff::set("@alice"),
            ..Default::default()
        },
    });
    let json = cap.to_json();

    println!("{}", json);
}

#[test]
fn reply_auth() {
    let reply = Reply::Users(UserReply::Auth(UserAuth(
        Identity::random(),
        "<invalid>".into(),
    )));
    let json = reply.to_json();

    let reply2 = Reply::from_json(&json).unwrap();

    assert_eq!(reply, reply2);
    println!("{}", json);
}
