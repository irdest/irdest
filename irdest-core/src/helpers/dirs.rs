use crate::error::{Error, Result};
use directories::ProjectDirs;
use tempfile::TempDir;
use std::{
    env,
    ffi::{CStr, OsStr},
    mem,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::Arc,
    ptr,
};
use libc::{self, c_char};

/// A config, data, and cache directories helper
///
/// Irdest can run on a lot of platforms that have different
/// requirements for where and how data can be stored.  To make
/// initialisation as easy as possible (for both tests, and clients on
/// different platforms) this structure is meant to be used to
/// initialise the irdest core!
#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Directories {
    pub(crate) data: PathBuf,
    pub(crate) config: PathBuf,
    pub(crate) cache: PathBuf,

    // Tempdir will remote itself if Directories is dropped
    temp: Option<Arc<TempDir>>,
}

pub fn home_dir() -> PathBuf {
    unsafe fn char_ptr_to_path_buf(ptr: *mut c_char) -> PathBuf {
        OsStr::from_bytes(CStr::from_ptr(ptr).to_bytes()).into()
    }

    // Check env var, otherwise, call libc::getpwuid_r
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| {
            let mut buf = [0; 4096];
            let mut result = ptr::null_mut();
            let mut passwd: libc::passwd = unsafe { mem::zeroed() };

            let getpwuid_r_code = unsafe {
                libc::getpwuid_r(
                    libc::getuid(),
                    &mut passwd,
                    buf.as_mut_ptr(),
                    buf.len(),
                    &mut result,
                )
            };
            // If success
            if getpwuid_r_code == 0 && !result.is_null() {
                let home_dir = unsafe { char_ptr_to_path_buf(passwd.pw_dir) };
                Some(home_dir)
            } else {
                None
            }
        })
        .unwrap()
}

impl Directories {
    /// Create a temporary directory tree for tests
    pub fn temp() -> Result<Self> {
        let temp = tempfile::tempdir()?;
        let temp_path = temp.path().to_path_buf();

        Ok(Self {
            data: temp_path.join("data"),
            config: temp_path.join("config"),
            cache: temp_path.join("cache"),
            temp: Some(Arc::new(temp)),
        })
    }

    /// Create directory metadata for the current platform
    ///
    /// Provide a client ID to separate data stored by different
    /// clients.  The 'organisation' of the client is always `irdest`.
    pub fn new(client_id: &str) -> Result<Self> {
        ProjectDirs::from("", "irdest", client_id)
            .ok_or(Error::IoFault)
            .map(|dirs| Self {
                data: dirs.data_dir().into(),
                config: dirs.config_dir().into(),
                cache: dirs.cache_dir().into(),
                temp: None,
            })
    }

    // Accesses the app-specific private directory for saving state
    pub fn android(client_id: &str) -> Directories{ 
    let mut android_home = PathBuf::new();
    android_home.push(home_dir()); // "/data"
    android_home.push("user/0");   // "/data/user/0"
    android_home.push(client_id);
    android_home.push("cache");
    info!("home_dir: {:?} ", android_home);

    Self { 
        data: android_home.clone(),
        config: android_home.clone(),
        cache: android_home.clone(),
        temp: None
    }
}

}
