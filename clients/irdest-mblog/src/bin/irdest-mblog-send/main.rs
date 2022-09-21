use anyhow::Result;
use clap::Parser;
use irdest_mblog::{Author, Message, Post};
use protobuf::Message as _;
use ratman_client::{Address, RatmanIpc};

const NAMESPACE: [u8; 32] = [
    0xF3, 0xFA, 0x1B, 0xCC, 0x57, 0x01, 0x7A, 0xCF, 0x57, 0x4C, 0x0F, 0xCF, 0x2E, 0x6F, 0x4F, 0x2B,
    0x24, 0x02, 0x90, 0x36, 0xE0, 0x0D, 0xC9, 0x25, 0xFA, 0xCC, 0xBB, 0x53, 0x5F, 0x80, 0x5E, 0x48,
];

/// sample microblog client - cli sender.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Address to use. Automatic if not set.
    #[clap(long)]
    addr: Option<String>,

    /// Specify a nickname.
    #[clap(short, long)]
    nick: String,

    /// Text to send.
    #[clap()]
    text: String,
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

    // Create a message.
    let msg = Message::new(Post {
        author: Author { nick: args.nick },
        text: args.text,
    });

    // Connect and send!
    RatmanIpc::default_with_addr(addr)
        .await?
        .flood(NAMESPACE.into(), msg.into_proto().write_to_bytes()?)
        .await?;

    Ok(())
}
