#[macro_use]
extern crate tracing;

use clap::{App, AppSettings, Arg};
use qrpc_sdk::{io::Message, Identity, RpcSocket, Service};
use std::io::{self, Read};
use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

#[macro_export]
macro_rules! fatal {
    () => {
        error!("Unknown failure!");
        std::process::exit(2)
    };
    ($($arg:tt)*) => ({
        error!($($arg)*);
        std::process::exit(2)
    })
}

pub(crate) fn parse_log_level() {
    let filter = EnvFilter::try_from_env("QAUL_LOG")
        .unwrap_or_default()
        .add_directive(LevelFilter::INFO.into())
        .add_directive("async_std=error".parse().unwrap())
        .add_directive("mio=error".parse().unwrap());

    // Initialise the logger
    fmt().with_env_filter(filter).init();
    debug!("Initialised logger!");
}

#[async_std::main]
async fn main() {
    parse_log_level();

    let (addr, port) = qrpc_sdk::default_socket_path();
    let default_bind = format!("{}:{}", addr, port);

    let matches = App::new("qrpc-client")
        .version(env!("CARGO_PKG_VERSION"))
        .global_settings(&[AppSettings::ArgRequiredElseHelp, AppSettings::GlobalVersion])
        .about(
            "A simple QRPC client which takes input on the standard input and sends messages to a QRPC broker",
        )
        .arg(
            Arg::with_name("SERVICE ADDR")
                .required(true)
                .takes_value(true)
                .help("Valid QRPC service identifier to send the message to"),
        )
        .arg(
            Arg::with_name("BROKER ADDR")
                .short("a")
                .takes_value(true)
                .help("Specify the broker address to connect to")
                .default_value(default_bind.as_str()),
        )
        .arg(
            Arg::with_name("SUBSCRIBE")
                .short("s")
                .help("Keep qrpc-client running to echo subscription events"),
        )
        .get_matches();

    let name = matches.value_of("SERVICE ADDR").unwrap();
    let addr_str = matches
        .value_of("BROKER ADDR")
        .or(Some(default_bind.as_str()))
        .unwrap();

    let mut serv = Service::new(
        Identity::random().to_string(),
        1,
        "A dynamic qrpc-client service".into(),
    );

    serv.register(RpcSocket::connect(addr, port).await.unwrap_or_else(|_| {
        fatal!(
            "Failed to connect to QRPC socket '{}'.  Is the broker running?",
            addr_str
        )
    }))
    .await
    .unwrap_or_else(|_| {
        fatal!(
            "Registration for the qrpc-client failed!  Is there already a service with that name?"
        )
    });

    // Read json from stdin
    let mut json = String::new();
    io::stdin()
        .read_to_string(&mut json)
        .unwrap_or_else(|e| fatal!("Failed to read from stdin: {}", e));

    let msg = Message::to_addr(name, &serv.name, json.as_bytes().to_vec());

    let sock = serv.get_socket();
    let reply = sock
        .send(msg, |msg| {
            let json = std::str::from_utf8(&msg.data.as_slice())
                .unwrap_or_else(|e| fatal!("Error while parsing reply: {}", e))
                .to_owned();

            Ok(json)
        })
        .await
        .unwrap();

    println!("{}", reply);
}
