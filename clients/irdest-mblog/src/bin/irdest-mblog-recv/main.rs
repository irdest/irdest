use anyhow::Result;
use clap::Parser;
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

    let addr = if let Some(addr_str) = args.addr {
        Address::from_string(&addr_str)
    } else {
        irdest_mblog::load_or_create_addr().await?
    };
    println!("{:?}", &addr);

    let ipc = RatmanIpc::default_with_addr(addr).await?;
    while let Some((tt, msg)) = ipc.next().await {
        println!("{:?} {:?}", tt, msg);
    }

    Ok(())
}
