// SPDX-FileCopyrightText: 2022 Manos Pitsidianakis <el13635@mail.ntua.gr>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Daemonise via fork and pidfiles for systems that are into it

use crate::{config::ConfigTree, start_with_configuration};
use nix::sys::{
    resource::{getrlimit, Resource},
    signal::{signal, sigprocmask, SigHandler, SigSet, SigmaskHow, Signal},
    stat,
};
use nix::unistd::{chdir, close, dup2, fork, pipe, read, setsid, write, ForkResult};
use nix::{
    env::clearenv,
    errno::Errno,
    fcntl::{open, OFlag},
    libc,
};
use std::{convert::TryInto, os::unix::io::RawFd, path::PathBuf};
use std::{error::Error, path::Path, result::Result as StdResult};

use super::pidfile::PidFile;

/// Start Ratman as a daemonised process
pub fn sysv_daemonize_app(cfg: ConfigTree, state_path: PathBuf) -> StdResult<(), Box<dyn Error>> {
    //  1. Close all open file descriptors except standard input,
    //     output, and error (i.e. the first three file descriptors 0,
    //     1, 2). This ensures that no accidentally passed file
    //     descriptor stays around in the daemon process. On Linux, this
    //     is best implemented by iterating through /proc/self/fd, with
    //     a fallback of iterating from file descriptor 3 to the value
    //     returned by getrlimit() for RLIMIT_NOFILE.
    if let (_, Some(max_fds)) = getrlimit(Resource::RLIMIT_NOFILE)? {
        for i in 3..=(max_fds.try_into().unwrap_or(RawFd::MAX)) {
            let _ = close(i);
        }
    }
    let (read_pipe_fd, write_pipe_fd) = pipe()?;

    //  2. Reset all signal handlers to their default. This is best done
    //     by iterating through the available signals up to the limit of
    //     _NSIG and resetting them to SIG_DFL.
    for signum in Signal::iterator() {
        if signum == Signal::SIGKILL || signum == Signal::SIGSTOP {
            continue;
        }
        unsafe { signal(signum, SigHandler::SigDfl)? };
    }

    //  3. Reset the signal mask using sigprocmask().
    sigprocmask(SigmaskHow::SIG_SETMASK, Some(&SigSet::empty()), None)?;

    //  4. Sanitize the environment block, removing or resetting
    //     environment variables that might negatively impact daemon
    //     runtime.
    unsafe { clearenv()? };

    if let ForkResult::Parent { child: _ } = unsafe { fork()? } {
        close(write_pipe_fd)?;
        let mut buf = [0u8; 4];
        read(read_pipe_fd, &mut buf)?;
        let pid = i32::from_be_bytes(buf);
        if pid < 0 {
            if pid == -libc::EAGAIN {
                println!("ratmand PID file is locked by another process.");
            } else {
                println!(
                    "ratmand daemonization failed with errno {}.",
                    Errno::from_i32(pid)
                );
            }
        } else {
            println!("ratmand PID: {}", pid);
        }
        return Ok(());
    }
    close(read_pipe_fd)?;

    /* Become session leader */
    setsid()?;
    if let ForkResult::Parent { child: _ } = unsafe { fork()? } {
        return Ok(());
    }

    //  9. In the daemon process, connect /dev/null to standard input,
    //     output, and error.

    let secondary_fd = open(Path::new("/dev/null"), OFlag::O_RDWR, stat::Mode::empty())?;

    // assign stdin, stdout, stderr to the process
    dup2(secondary_fd, libc::STDIN_FILENO)?;
    dup2(secondary_fd, libc::STDOUT_FILENO)?;
    dup2(secondary_fd, libc::STDERR_FILENO)?;

    close(secondary_fd)?;

    // 10. In the daemon process, reset the umask to 0, so that the file
    //     modes passed to open(), mkdir() and suchlike directly control
    //     the access mode of the created files and directories.

    stat::umask(stat::Mode::empty());

    // 11. In the daemon process, change the current directory to the
    //     root directory (/), in order to avoid that the daemon
    //     involuntarily blocks mount points from being unmounted.

    chdir("/")?;

    // 12. In the daemon process, write the daemon PID (as returned by
    //     getpid()) to a PID file, for example /run/foobar.pid (for a
    //     hypothetical daemon "foobar") to ensure that the daemon
    //     cannot be started more than once. This must be implemented in
    //     race-free fashion so that the PID file is only updated when
    //     it is verified at the same time that the PID previously
    //     stored in the PID file no longer exists or belongs to a
    //     foreign process.
    let pid_file = match PidFile::new(
        // We can rely on these settings existing because otherwise we
        // would not call this function.  "Trust me bro", basically.
        &cfg.get_subtree("ratmand")
            .and_then(|tree| tree.get_string_value("pidfile"))
            .unwrap(),
    )
    .and_then(|f| f.write_pid())
    {
        Ok(v) => v,
        Err(err) => {
            let err = err as i32;
            let error_bytes: [u8; 4] = (-err).to_be_bytes();
            write(write_pipe_fd, &error_bytes)?;
            close(write_pipe_fd)?;
            return Ok(());
        }
    };

    let pid_bytes: [u8; 4] = pid_file.0.to_be_bytes();
    write(write_pipe_fd, &pid_bytes)?;
    close(write_pipe_fd)?;

    let _ = start_with_configuration(cfg, state_path);

    // Unlock and close/delete pid file
    drop(pid_file);
    Ok(())
}
