use crate::{bundle_dir, print_path, Directories};
use colored::Colorize;
use semver::Version;
use std::{error::Error, path::PathBuf};

pub enum File {
    Ratmand,
    Ratcat,
    Ratctl,
    RatmandMan,
    SystemdUnit,
}

pub enum Status {
    Missing,
    Exists,
}

impl File {
    pub fn bundle_path(&self) -> &str {
        match self {
            Self::Ratmand => "bin/ratmand",
            Self::Ratcat => "bin/ratcat",
            Self::Ratctl => "bin/ratctl",
            Self::RatmandMan => "man/ratmand.1",
            Self::SystemdUnit => "dist/ratman.service",
        }
    }

    pub fn get_target(&self, dirs: &Directories) -> PathBuf {
        match self {
            Self::Ratmand => dirs.bin_dir.join("ratmand"),
            Self::Ratcat => dirs.bin_dir.join("ratcat"),
            Self::Ratctl => dirs.bin_dir.join("ratctl"),
            Self::RatmandMan => dirs.ratmand_man_path(),
            Self::SystemdUnit => dirs.systemd_unit(),
        }
    }

    pub fn install_state(&self, dirs: &Directories) -> Status {
        let target = self.get_target(dirs);
        let bundle_version = crate::check_version(&bundle_dir().join(self.bundle_path()))
            .map(|v| Version::parse(&v).ok());
        let target_version =
            crate::check_version(&self.get_target(dirs)).map(|v| Version::parse(&v).ok());

        match (target.exists(), bundle_version, target_version) {
            (true, Some(v1), Some(v2)) if v1 > v2 => {
                println!(
                    "({}) {} -> {}",
                    "UPGRADE".cyan(),
                    self.bundle_path(),
                    print_path(&target)
                );
                Status::Exists
            }
            (true, Some(v1), Some(v2)) if v1 < v2 => {
                println!(
                    "({}) {} -> {}",
                    "DOWNGRADE".yellow(),
                    self.bundle_path(),
                    print_path(&target)
                );
                Status::Exists
            }
            (true, _, _) => {
                println!(
                    "({}) {} -> {}",
                    "REPLACE".cyan(),
                    self.bundle_path(),
                    print_path(&target)
                );
                Status::Exists
            }
            (false, _, _) => {
                println!(
                    "({}) {} -> {}",
                    "NEW".green(),
                    self.bundle_path(),
                    print_path(&target)
                );

                Status::Missing
            }
        }
    }

    pub fn install(&self, dirs: &Directories, bundle_dir: &PathBuf) {
        match std::fs::copy(bundle_dir.join(self.bundle_path()), self.get_target(dirs)) {
            Ok(_) => {
                println!(
                    "Install {}: {}",
                    print_path(&self.get_target(dirs)),
                    "OK".bright_green()
                )
            }
            Err(e) => {
                eprintln!(
                    "Install {}: {}",
                    print_path(&self.get_target(dirs)),
                    format!(
                        "{} ({})",
                        "FAILED".bright_red(),
                        e.to_string().split("(").nth(0).unwrap().trim()
                    )
                );
            }
        }
    }

    pub fn install_unitfile(&self, dirs: &Directories, bundle_dir: &PathBuf) {
        crate::systemd::install_unitfile(
            bundle_dir.join(self.bundle_path()),
            &Self::Ratmand.get_target(dirs),
            &self.get_target(dirs),
        );
    }
}
