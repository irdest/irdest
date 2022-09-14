use anyhow::{anyhow, Result};
use ratman_client::{Address, RatmanIpc};

/// Loads an address from a file ('addr' in the system-appropriate config dir), or
/// if that doesn't exist, call the local ratmand to generate one, stashing it in
/// said file to be found on our next run.
pub async fn load_or_create_addr() -> Result<Address> {
    // Find our configuration directory. Make sure to respect $XDG_CONFIG_HOME!
    let dirs = directories::ProjectDirs::from("org", "irdest", "irdest-mblog")
        .ok_or(anyhow!("couldn't find config dir"))?;
    let cfg_dir = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(|path| path.into())
        .unwrap_or_else(|| dirs.config_dir().to_path_buf());

    // Try to read an existing "addr" file...
    let addr_path = cfg_dir.join("addr");
    match async_std::fs::read_to_string(&addr_path).await {
        // We've done this before - use the existing address.
        Ok(s) => Ok(Address::from_string(&s)),

        // There's no "addr" file - let's create one.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Create the config directory.
            match async_std::fs::create_dir_all(&cfg_dir).await {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
                Err(e) => Err(e),
            }?;

            // Connect to ratmand and generate a new address.
            let ipc = RatmanIpc::default().await?;
            let addr = ipc.address();

            // Write it to the "addr" file.
            async_std::fs::write(&addr_path, addr.to_string().as_bytes()).await?;

            Ok(addr)
        }

        // Something else went wrong, eg. the file has the wrong permissions set.
        // Don't attempt to clobber it; tell the user and let them figure it out.
        Err(e) => Err(e.into()),
    }
}
