// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 embr <hi@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

// We gate this builder on the inclusion of the "proto" feature, which
// is disabled on docs.rs.  In that case we include an empty main

#[cfg(feature = "proto")]
fn build_proto() {
    use protoc_rust::Customize;
    use std::{env, fs, path::Path};

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = format!("{}/proto_gen", out_dir);

    if Path::new(&out_path).exists() {
        let _ = fs::remove_dir_all(&out_path);
    }
    fs::create_dir_all(&out_path).unwrap();

    protoc_rust::Codegen::new()
        .customize(Customize {
            gen_mod_rs: Some(true),
            ..Default::default()
        })
        .out_dir(out_path)
        .input("proto/feed.proto")
        .input("proto/db.proto")
        .run()
        .expect("Failed to compile protobuf schemas!");
}

#[cfg(not(feature = "proto"))]
fn build_proto() {}

#[cfg(feature = "mblog-gtk")]
fn build_resources() {
    gtk::gio::compile_resources(
        "resources",
        "resources/resources.gresources.xml",
        "resources.gresource",
    );
}

#[cfg(not(feature = "mblog-gtk"))]
fn build_resources() {}

fn main() {
    build_proto();
    build_resources();
}
