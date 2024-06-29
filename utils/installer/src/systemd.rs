use colored::Colorize;
use std::{
    boxed::Box,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    process::Command,
};

pub fn install_unitfile(path: PathBuf, ratmand_path: &PathBuf, target: &PathBuf) {
    let mut buf = String::new();
    let mut f = File::open(path).unwrap();
    f.read_to_string(&mut buf).unwrap();

    let new_str = buf.replace("/path/to/ratmand", &crate::print_path(&ratmand_path));

    let mut new_file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(target)
        .unwrap();
    new_file.write_all(new_str.as_bytes()).unwrap();

    if Command::new("systemctl")
        .arg("--user")
        .arg("daemon-reload")
        .output()
        .is_ok()
    {
        println!("systemctl daemon-reload: {}", "OK".bright_green())
    }
}

pub fn uninstall_unitfile(target: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::remove_file(target)?;

    Command::new("systemctl")
        .arg("--user")
        .arg("daemon-reload")
        .output()?;

    Ok(())
}
