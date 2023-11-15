// SPDX-FileCopyrightText: 2022-2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Various platform abstractions

use crate::util::{env_xdg_config, env_xdg_data};
use directories::ProjectDirs;
use libratman::{tokio::fs::File, NonfatalError, RatmanError, Result};
use std::{os::fd::AsRawFd, path::PathBuf};

/// OS specific support
pub enum Os {
    Android,
    Unix,
    Unknown,
    Ios,
    Windows,
}

/// Represents a state-directory lock which must be kept alive for the
/// runtime of the daemon
pub struct StateDirectoryLock(File);

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

    /// Attempt to lock the state directory to ensure only one ratman
    /// instance runs in it
    pub async fn lock_state_directory(path: Option<PathBuf>) -> Result<Option<StateDirectoryLock>> {
        #[cfg(target_family = "unix")]
        {
            let path: PathBuf = path.unwrap_or_else(|| Self::match_os().data_path());
            let f = File::open(&path).await?;
            match nix::fcntl::flock(f.as_raw_fd(), nix::fcntl::FlockArg::LockExclusiveNonblock)
                .map(|_| Some(StateDirectoryLock(f)))
            {
                Ok(lock) => Ok(lock),
                Err(_) => Err(RatmanError::StateDirectoryAlreadyLocked),
            }
        }

        #[cfg(target_family = "windows")]
        {
            warn!(
                "Directory locks are not implemented on Windows! \
                 please make sure only one ratmand instance runs at a time"
            );
            Ok(None)
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

    /// Get the XDG_CONFIG path for the current system
    pub fn xdg_config_path() -> PathBuf {
        let dirs = ProjectDirs::from("org", "irdest", "ratmand")
            .expect("Failed to initialise project directories");
        let config_dir = env_xdg_config()
            .map(|path| PathBuf::new().join(path))
            .unwrap_or_else(|| dirs.config_dir().to_path_buf());

        trace!("Ensure data directory exists: {:?}", config_dir);
        let _ = std::fs::create_dir_all(&config_dir);
        config_dir
    }

    /// Get the XDG_DATA path for the current system
    pub fn xdg_data_path() -> PathBuf {
        let dirs = ProjectDirs::from("org", "irdest", "ratmand")
            .expect("Failed to initialise project directories");
        let data_dir = env_xdg_data()
            .map(|path| PathBuf::new().join(path))
            .unwrap_or_else(|| dirs.data_dir().to_path_buf());

        trace!("Ensure data directory exists: {:?}", data_dir);
        let _ = std::fs::create_dir_all(&data_dir);
        data_dir
    }

    /// Return the IrdestVPN data path on Android
    // TODO: make this somehow configurable?  Does that makes sense?
    pub fn android_data_path() -> PathBuf {
        PathBuf::new().join("/data/user/0/org.irdest.IrdestVPN/files/")
    }
}
