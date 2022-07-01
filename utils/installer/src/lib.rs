mod file;
mod systemd;

pub use file::*;

use directories::BaseDirs;
use std::{path::PathBuf, process::Command};

#[derive(Debug)]
pub struct Directories {
    pub bin_dir: PathBuf,
    pub cfg_dir: PathBuf,
    pub dat_dir: PathBuf,
}

impl Directories {
    pub fn new() -> Self {
        let base = BaseDirs::new().expect("failed to determine directories");

        Directories {
            bin_dir: base
                .executable_dir()
                .expect("failed to determine BIN path")
                .to_path_buf(),
            cfg_dir: base.config_dir().to_path_buf(),
            dat_dir: base.data_dir().to_path_buf(),
        }
    }

    pub fn systemd_unit(&self) -> PathBuf {
        self.cfg_dir
            .join("systemd")
            .join("user")
            .join("ratman.service")
    }

    pub fn ratmand_man_path(&self) -> PathBuf {
        self.dat_dir.join("man").join("man1").join("ratmand.1")
    }
}

pub fn print_path(p: &PathBuf) -> String {
    p.to_str().unwrap().to_string()
}

pub fn bundle_dir() -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Try to run --version on something
pub fn check_version(bin: &PathBuf) -> Option<String> {
    let out = Command::new(bin).args(&["--version"]).output().ok()?.stdout;
    Some(String::from_utf8(out).unwrap())
}
