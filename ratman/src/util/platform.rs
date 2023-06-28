// SPDX-FileCopyrightText: 2022-2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore


//! Various platform abstractions

use crate::util::env_xdg_data;
use directories::ProjectDirs;
use std::path::PathBuf;

/// OS specific support
pub enum Os {
    Android,
    Unix,
    Unknown,
    Ios,
    Windows,
}

impl Os {
    pub fn match_os() -> Os {
        match std::env::consts::OS.as_ref() {
            "linux" | "macos" | "freebsd" | "dragonfly" | "netbsd" | "openbsd" | "solaris" => {
                Self::Unix
            }
            "android" => Self::Android,
            "ios" => Self::Ios,
            "windows" => Self::Windows,
            _ => Self::Unknown, // Found ailian Os write log
        }
    }

    /// A wrapper for both xdg_data_path and android_data_path
    ///
    pub fn data_path(&self) -> PathBuf {
        match self {
            Self::Android => Self::android_data_path(),
            Self::Unix | Self::Windows => Self::xdg_data_path(),
            _ => Self::xdg_data_path(), // Maybe try
        }
    }

    /// Get the XDG_DATA path for the current system
    pub fn xdg_data_path() -> PathBuf {
        let dirs = ProjectDirs::from("org", "irdest", "ratmand")
            .expect("Failed to initialise project directories");
        let data_dir = env_xdg_data()
            .map(|path| PathBuf::new().join(path))
            .unwrap_or_else(|| dirs.data_dir().to_path_buf());
        trace!("Ensure data directory exists: {:?}", data_dir);
        let _ = std::fs::create_dir(&data_dir);

        PathBuf::new().join(data_dir).join("users.json")
    }

    /// Return the IrdestVPN data path on Android
    // TODO: make this somehow configurable?  Does that makes sense?
    pub fn android_data_path() -> PathBuf {
        PathBuf::new().join("/data/user/0/org.irdest.IrdestVPN/files/users.json")
    }
}
