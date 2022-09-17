use anyhow::Result;
use clap::Parser;
use irdest_mblog::proto::feed::Post;
use protobuf::Message;
use ratman_client::{Address, RatmanIpc};

/// sample microblog client - cli receiver.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Address to use. Automatic if not set.
    #[clap(long)]
    addr: Option<String>,
}

#[async_std::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Take an address from --addr, load it from disk, or generate and save one.
    let addr = if let Some(addr_str) = args.addr {
        Address::from_string(&addr_str)
    } else {
        irdest_mblog::load_or_create_addr().await?
    };

    let ipc = RatmanIpc::default_with_addr(addr).await?;
    while let Some((tt, msg)) = ipc.next().await {
        match Post::parse_from_bytes(&msg.get_payload()[..]) {
            Ok(post) => {
                println!("{}: {}", post.author.unwrap_or_default().nick, post.text);
            }
            Err(e) => {
                eprintln!("[invalid message]: {:}\n{:?} {:?}", e, tt, msg);
            }
        };
    }

    Ok(())
}
