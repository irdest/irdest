// SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Lux <lux@lux.name>
// SPDX-FileCopyrightText: 2022 Christopher A. Grant <grantchristophera@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{daemon::config::Config, *};
use netmod_inet::InetEndpoint as Inet;
use netmod_lan::{default_iface, Endpoint as LanDiscovery};

#[cfg(feature = "lora")]
use netmod_lora::LoraEndpoint;

use clap::{App, Arg, ArgMatches};
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
use std::{convert::TryInto, os::unix::io::RawFd};
use std::{error::Error, fs::File, io::Read, path::Path, result::Result as StdResult};

/// Start Ratman in the current process/ thread
pub fn build_cli() -> ArgMatches<'static> {
    App::new("ratmand")
        .about("Decentralised and delay tolerant peer-to-peer packet router.  Part of the Irdest project: https://irde.st")
        .version(env!("CARGO_PKG_VERSION"))
        .after_help("This is ALPHA level software and will include bugs and cause crashes.  If you encounter a reproducible issue, please report it in our issue tracker (https://git.irde.st/we/irdest) or our mailing list: https://lists.irde.st/archives/list/community@lists.irde.st")
        .max_term_width(120)
        .arg(
            Arg::with_name("VERBOSITY")
                .takes_value(true)
                .short("v")
                .long("verbosity")
                .possible_values(&["trace", "debug", "info", "warn", "error", "fatal"])
                .default_value("info")
                .help("Specify the verbosity level at which ratmand logs interactions"),
        )
        .arg(
            Arg::with_name("ACCEPT_UNKNOWN_PEERS")
                .long("accept-unknown-peers")
                .short("d")
                // .required_unless_one(&["PEERS", "PEER_FILE", "NO_INET"])
                .help("Configure ratmand to peer with any incoming connection it may encounter")
        )
        .arg(
            Arg::with_name("API_BIND")
                .takes_value(true)
                .long("bind")
                .short("b")
                .help("Specify the API socket bind address")
                .default_value("127.0.0.1:9020"),
        )
        .arg(
            Arg::with_name("INET_BIND")
                .takes_value(true)
                .long("inet")
                .help("Specify the inet-driver socket bind address.  Make sure this port is open in your firewall")
                .default_value("[::]:9000"),
        )
        .arg(
            Arg::with_name("NO_INET")
                .long("no-inet")
                .help("Disable the inet overlay driver")
        )
        .arg(
            Arg::with_name("DISCOVERY_PORT")
                .long("discovery-port")
                .takes_value(true)
                .default_value("9001")
                .help("Specify the port used for local peer discovery.  Make sure this port is open in your firewall.  WARNING: it's not recommended to change this unless you know this is what you want!")
        )
        .arg(
            Arg::with_name("DISCOVERY_IFACE")
                .takes_value(true)
                .long("discovery-iface")
                .help("Specify the interface on which to bind for local peer discovery.  If none is provided the default interface will be attempted to be determined")
        )
        .arg(
            Arg::with_name("NO_DISCOVERY")
                .long("no-discovery")
                .help("Disable the local multicast peer discovery mechanism")
        )
        .arg(
            {
                let arg = Arg::with_name("NO_LORA")
                .long("no-lora")
                    .help("Disable the lora modem driver");

                #[cfg(not(feature = "lora"))]
                let arg = arg.hidden(true);
                arg
            })
        .arg(
            {
                let arg = Arg::with_name("DATALINK_IFACE")
                    .takes_value(true)
                    .long("datalink-iface")
                    .help("Specify datalink interface.");

                #[cfg(not(feature = "datalink"))]
                let arg = arg.hidden(true);
                arg
            })
        .arg(
            {
                let arg = Arg::with_name("SSID")
                    .takes_value(true)
                    .long("ssid")
                    .help("Specify SSID for wireless interface");

                #[cfg(not(feature = "datalink"))]
                let arg = arg.hidden(true);
                arg
            }
        )
        .arg(
            {
                let arg = Arg::with_name("NO_DATALINK")
                    .long("no-datalink")
                    .help("Disables datalink driver");

                #[cfg(not(feature = "datalink"))]
                let arg = arg.hidden(true);
                arg
            }
            )
        .arg(
            Arg::with_name("PEERS")
                .long("peers")
                .short("p")
                .help("Specify a set of peers via the PEER SYNTAX: <netmod-id>#<address>:<port>[L].  Incompatible with `-f`. Valid netmod-ids are tcp. Example: tcp#10.0.0.10:9000L")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("PEER_FILE")
                .long("peer-file")
                .short("f")
                .help("Provide a set of initial peers to connect to.  Incompatible with `-p`")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("USE_UPNP")
                .long("upnp")
                .hidden(true)
                .help("Attempt to open the port used by the inet driver in your local gateway")
        )
        .arg(
            Arg::with_name("NO_DASHBOARD")
                .long("no-dashboard")
                .help("Stop ratmand from serving a dashboard on port 8090")
        )
        .arg(
            Arg::with_name("DAEMONIZE")
                .long("daemonize")
                .help("Fork ratmand into the background and detach it from the current stdout/stderr/tty")
        )
        .arg(
            Arg::with_name("PID_FILE")
                .takes_value(true)
                .long("pid-file")
                .help("A file which the process PID is written into")
                .default_value("/tmp/ratmand.pid"),
        )
        .get_matches()
}

pub async fn run_app(cfg: Config) -> std::result::Result<(), ()> {
    // Setup logging
    daemon::setup_logging(&cfg.verbosity, cfg.daemonize);

    // Setup metrics collection
    #[cfg(feature = "dashboard")]
    let mut registry = prometheus_client::registry::Registry::default();

    let r = Router::new();

    #[cfg(feature = "dashboard")]
    r.register_metrics(&mut registry);

    if !cfg.no_inet {
        // Load peers or throw an error about missing cli data!
        let peers: Vec<_> = match cfg
            .peers
            .as_ref()
            .map(|s| s.replace(" ", "\n").to_owned())
            .or(cfg.peer_file.as_ref().and_then(|path| {
                let mut f = File::open(path).ok()?;
                let mut buf = String::new();
                f.read_to_string(&mut buf).ok()?;
                Some(buf)
            }))
            .or(if cfg.no_peering {
                Some("".into())
            } else {
                None
            }) {
            Some(peer_str) => peer_str.split("\n").map(|s| s.trim().to_owned()).collect(),
            None if !cfg.accept_unknown_peers => {
                daemon::elog("Failed to initialise ratmand: missing peers data!", 2)
            }
            None => vec![],
        };

        let tcp = match Inet::start(&cfg.inet_bind).await {
            Ok(tcp) => {
                // Open the UPNP port if the user enabled this feature
                if cfg.use_upnp {
                    if let Err(e) = daemon::upnp::open_port(tcp.port()) {
                        error!("UPNP setup failed: {}", e);
                    }
                }

                let peers: Vec<_> = peers.iter().map(|s| s.as_str()).collect();
                match daemon::attach_peers(&tcp, peers).await {
                    Ok(()) => tcp,
                    Err(e) => daemon::elog(format!("failed to parse peer data: {}", e), 1),
                }
            }
            Err(e) => daemon::elog(format!("failed to initialise TCP endpoint: {}", e), 1),
        };

        r.add_endpoint(tcp).await;
    }

    // If local-discovery is enabled
    if !cfg.no_discovery {
        if let Ok(ep) = LanDiscovery::spawn(cfg.discovery_iface, cfg.discovery_port) {
            r.add_endpoint(ep).await;
        }
    }

    #[cfg(feature = "lora")]
    if !cfg.no_lora {
        let lora = LoraEndpoint::spawn(
            cfg.lora_port
                .expect("You must provide a lora serial port path!")
                .as_str(),
            cfg.lora_baud,
        );
        r.add_endpoint(lora).await;
    }

    // If dashboard is enabled
    #[cfg(feature = "dashboard")]
    if !cfg.no_dashboard {
        match daemon::web::start(r.clone(), registry, "127.0.0.1", 8090).await {
            Ok(_) => {}
            Err(e) => warn!("Failed to setup dashboard bind {:?}", e),
        }
    }

    #[cfg(feature = "datalink")]
    if !cfg.no_datalink {
        use netmod_datalink::Endpoint as Datalink;

        r.add_endpoint(Datalink::spawn(
            cfg.datalink_iface.as_ref().map(|s| s.as_str()),
            cfg.ssid.as_ref().map(|s| s.as_str()),
        ))
        .await;
    }

    let api_bind = match cfg.api_bind.parse() {
        Ok(addr) => addr,
        Err(e) => daemon::elog(format!("Failed to parse API_BIND address: {}", e), 2),
    };

    if let Err(e) = daemon::run(r, api_bind).await {
        error!("Ratmand suffered fatal error: {}", e);
    }
    Ok(())
}

/// Start Ratman as a daemonised process
pub fn sysv_daemonize_app(cfg: Config) -> StdResult<(), Box<dyn Error>> {
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
    let pid_file = match daemon::pidfile::PidFile::new(&cfg.pid_file).and_then(|f| f.write_pid()) {
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

    let _ = async_std::task::block_on(run_app(cfg));

    // Unlock and close/delete pid file
    drop(pid_file);
    Ok(())
}
