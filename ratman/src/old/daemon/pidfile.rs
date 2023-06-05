//! Pid file logic for daemonized ratmand instances.

/// A pid file that automatically unlocks and closes the file descriptor on `Drop`.
pub struct PidFile(pub i32, std::os::unix::prelude::RawFd, String);

impl PidFile {
    pub fn new(path: &str) -> std::result::Result<Self, nix::errno::Errno> {
        let pid = nix::unistd::getpid().as_raw();
        let pid_file = nix::fcntl::open(
            std::path::Path::new(path),
            nix::fcntl::OFlag::O_RDWR | nix::fcntl::OFlag::O_CREAT,
            nix::sys::stat::Mode::from_bits(0o0777).unwrap(),
        )?;
        let flock = nix::libc::flock {
            l_type: nix::libc::F_WRLCK as _,
            l_whence: nix::libc::SEEK_SET as _,
            l_start: 0,
            l_len: 0,
            l_pid: 0,
        };
        nix::fcntl::fcntl(pid_file, nix::fcntl::FcntlArg::F_SETLK(&flock))?;
        Ok(Self(pid, pid_file, path.to_string()))
    }

    pub fn write_pid(self) -> std::result::Result<Self, nix::errno::Errno> {
        nix::unistd::write(self.1, format!("{}", self.0).as_bytes())?;
        Ok(self)
    }
}

/* Note: this won't run if the daemon is killed by a signal. */
impl Drop for PidFile {
    fn drop(&mut self) {
        let flock = nix::libc::flock {
            l_type: nix::libc::F_UNLCK as _,
            l_whence: nix::libc::SEEK_SET as _,
            l_start: 0,
            l_len: 0,
            l_pid: 0,
        };
        let _ = nix::fcntl::fcntl(self.1, nix::fcntl::FcntlArg::F_SETLK(&flock));
        let _ = nix::unistd::close(self.1);
        let _ = nix::unistd::unlink(self.2.as_str());
    }
}
