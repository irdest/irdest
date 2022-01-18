use clap::{App, Arg};
use ratman_client::{Identity, RatmanIpc};

const ASCII: &str = r#"      ,     .             
      (\,;,/)                    (\,/)
       (o o)\//,                 oo   '''//,        _
        \ /     \,             ,/_;~,        \,    / '
        `+'(  (   \    )       "'   \    (    \    !
           //  \   |_./              ',|  \    |__.'
         '~' '~----'                 '~  '~----''
                R A T    C O N T R O L
"#;

fn setup_cli() -> App<'static, 'static> {
    App::new("ratctl")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Management cli for Ratman router daemon")
        .after_help("ratcat(1) stores current address information in $XDG_CONFIG_DIR/ratcat/config\n\nThis is ALPHA level software and will include bugs and cause crashes.  If you encounter a reproducible issue, please report it in our issue tracker (https://git.irde.st/we/irdest) or our mailing list: https://lists.irde.st/archives/list/community@lists.irde.st")
        .max_term_width(120)
        .arg(
            Arg::with_name("API_BIND")
                .takes_value(true)
                .long("bind")
                .short("b")
                .help("Specify the API socket bind address")
                .default_value("127.0.0.1:9020"),
        )
        .arg(
            Arg::with_name("GET_PEERS")
                .long("get-peers")
                .required_unless("SUBSCRIBE_PEERS")
                .conflicts_with("SUBSCRIBE_PEERS")
                .help("Request the currently known list of peers from the router")
        )
        .arg(
            // Doesn't currently work :(
            Arg::with_name("SUBSCRIBE_PEERS")
                .hidden(true)
                .long("subscribe-peers")
                .required_unless("GET_PEERS")
                .conflicts_with("GET_PEERS")
                .help("Remain running and be notified about new peers as they are discovered")
        )
}

async fn connect_ipc(bind: &str) -> Result<RatmanIpc, Box<dyn std::error::Error>> {
    eprintln!("Connecting to IPC backend...");
    Ok(RatmanIpc::anonymous(bind).await?)
}

async fn get_peers(ipc: &RatmanIpc) -> Result<Vec<Identity>, Box<dyn std::error::Error>> {
    Ok(ipc.get_peers().await?)
}

#[async_std::main]
async fn main() {
    let cli = setup_cli();
    let m = cli.get_matches();

    let bind = m.value_of("API_BIND").unwrap();
    let ipc = match connect_ipc(bind).await {
        Ok(ipc) => ipc,
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            std::process::exit(1);
        }
    };

    //////// RELEASE THE RATS tztztztztztztztz
    eprintln!("{}", ASCII);

    if m.is_present("GET_PEERS") {
        let peers = match get_peers(&ipc).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to fetch peers: {}", e);
                std::process::exit(1);
            }
        };

        peers.into_iter().for_each(|p| println!("{}", p));
    } else if m.is_present("SUBSCRIBE_PEERS") {
        while let Some(peer) = ipc.discover().await {
            println!("Discovered {}", peer);
        }
    }
}
