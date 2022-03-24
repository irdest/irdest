//! Ratman daemon entrypoint

#[macro_use]
extern crate tracing;

pub(crate) use ratman::*;

use clap::{App, Arg, ArgMatches};
use netmod_inet::InetEndpoint as Inet;
use netmod_lan::{default_iface, Endpoint as LanDiscovery};
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
use ratman::daemon::config::Config;
use std::convert::TryInto;
use std::os::unix::io::RawFd;
use std::path::Path;
use std::{fs::File, io::Read};

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
            Arg::with_name("NO_WEBUI")
                .long("no-webui")
                .help("Stop ratmand from serving a webui on port 8090")
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

// Ok(()) -> all good
// Err(_) -> emit warning but keep going
async fn setup_local_discovery(
    r: &Router,
    m: &ArgMatches<'_>,
    c: &Config,
) -> std::result::Result<(String, u16), String> {
    let iface = m.value_of("DISCOVERY_IFACE")
        .map(Into::into)
        .or_else(|| default_iface().map(|iface| {
            info!("Auto-selected interface '{}' for local peer discovery.  Is this wrong?  Pass --discovery-iface to ratmand instead!", iface);
            iface
        })).ok_or("failed to determine interface to bind on".to_string())?;

    let port = m
        .value_of("DISCOVERY_PORT")
        .unwrap_or(c.netmod_lan_bind.as_str())
        .parse()
        .map_err(|e| format!("failed to parse discovery port: {}", e))?;

    r.add_endpoint(LanDiscovery::spawn(&iface, port)).await;
    Ok((iface, port))
}

async fn run_app(m: ArgMatches<'static>, configuration: Config) -> std::result::Result<(), ()> {
    let dynamic = m.is_present("ACCEPT_UNKNOWN_PEERS") || configuration.accept_unknown_peers;

    // Setup logging
    daemon::setup_logging(m.value_of("VERBOSITY").unwrap());

    // Load peers or throw an error about missing cli data!
    let peers: Vec<_> = match m
        .value_of("PEERS")
        .map(|s| s.replace(" ", "\n").to_owned())
        .or(m.value_of("PEER_FILE").and_then(|path| {
            let mut f = File::open(path).ok()?;
            let mut buf = String::new();
            f.read_to_string(&mut buf).ok()?;
            Some(buf)
        }))
        .or(if m.is_present("NO_PEERING") {
            Some("".into())
        } else {
            None
        }) {
        Some(peer_str) => peer_str.split("\n").map(|s| s.trim().to_owned()).collect(),
        None if !dynamic => daemon::elog("Failed to initialise ratmand: missing peers data!", 2),
        None => vec![],
    };

    let r = Router::new();
    if !m.is_present("NO_INET") || configuration.netmod_inet_enabled {
        let tcp = match Inet::start(
            m.value_of("INET_BIND")
                .unwrap_or(configuration.netmod_inet_bind.as_str()),
        )
        .await
        {
            Ok(tcp) => {
                // Open the UPNP port if the user enabled this feature
                if m.is_present("USE_UPNP") {
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
    if !m.is_present("NO_DISCOVERY") || configuration.netmod_lan_enabled {
        match setup_local_discovery(&r, &m, &configuration).await {
            Ok((iface, port)) => debug!(
                "Local peer discovery running on interface {}, port {}",
                iface, port
            ),
            Err(e) => warn!("Failed to setup local peer discovery: {}", e),
        }
    }

    // If webui is enabled
    if !m.is_present("NO_WEBUI") {
        match daemon::web::start(r.clone(), "127.0.0.1", 8090).await {
            Ok(_) => {}
            Err(e) => warn!("Failed to setup webui bind {:?}", e),
        }
    }

    let api_bind = match m
        .value_of("API_BIND")
        .unwrap_or(configuration.api_socket_bind.as_str())
        .parse()
    {
        Ok(addr) => addr,
        Err(e) => daemon::elog(format!("Failed to parse API_BIND address: {}", e), 2),
    };
    if let Err(e) = daemon::run(r, api_bind).await {
        error!("Ratmand suffered fatal error: {}", e);
    }
    Ok(())
}

fn sysv_daemonize_app(
    m: ArgMatches<'static>,
    configuration: Config,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
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
    let pid_filepath = m.value_of("PID_FILE").unwrap();

    let pid_file = match daemon::pidfile::PidFile::new(&pid_filepath).and_then(|f| f.write_pid()) {
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

    let _ = async_std::task::block_on(run_app(m, configuration));

    // Unlock and close/delete pid file
    drop(pid_file);
    Ok(())
}

fn main() {
    let configuration = match daemon::config::Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!(
                "Failed to load/write configuration: {}. Resuming with default values.",
                e
            );
            daemon::config::Config::new()
        }
    };

    let m = build_cli();
    let daemonize = m.is_present("DAEMONIZE");
    if daemonize {
        if let Err(err) = sysv_daemonize_app(m, configuration) {
            eprintln!("Ratmand suffered fatal error: {}", err);
            std::process::exit(-1);
        }
    } else if let Err(()) = async_std::task::block_on(run_app(m, configuration)) {
        std::process::exit(-1);
    }
}
