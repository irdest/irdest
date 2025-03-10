use crate::{encode_map, OutputFormat};
use clap::ArgMatches;
use directories::BaseDirs;
use libratman::{
    tokio::{fs::OpenOptions, io::AsyncWriteExt},
    types::{AddrAuth, Address},
    Result,
};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug)]
pub struct BaseArgs {
    pub identity_path: String,
    pub identity_data: Result<(Address, AddrAuth)>,
    pub out_fmt: OutputFormat,
    pub profile: String,
    pub quiet: bool,
}

pub fn parse_base_args(m: &ArgMatches) -> BaseArgs {
    let output_format = m.get_one::<String>("output-format").unwrap();
    let profile: String = m
        .get_one::<String>("profile")
        .map(Clone::clone)
        .unwrap_or("id".to_string());

    let selected_state_path: PathBuf = m
        .get_one::<String>("state-dir")
        .map(|x| PathBuf::new().join(x))
        .unwrap_or_else(|| {
            BaseDirs::new()
                .expect("failed to determine directories")
                .config_dir()
                .to_path_buf()
                .join("ratcat")
        });

    let selected_id_path = selected_state_path.join(&profile).clone();

    let out_fmt = match output_format.as_str() {
        "lines" => OutputFormat::Lines,
        "json" => OutputFormat::Json,
        _ => unreachable!(),
    };

    if let Err(e) = std::fs::create_dir_all(selected_state_path) {
        eprintln!("(nonfatal) failed to create ratcat state directory: {e}");
    }

    let identity_data = (|| -> Result<(Address, AddrAuth)> {
        let mut f = std::fs::File::open(selected_id_path.as_path())?;
        let mut s = String::new();

        use std::io::Read;
        f.read_to_string(&mut s)?;

        match out_fmt {
            OutputFormat::Lines => {
                let mut lines = s.lines();

                let addr = lines.next().unwrap().split("=").last().unwrap().to_string();
                let auth = lines.next().unwrap().split("=").last().unwrap().to_string();

                Ok((Address::from_string(&addr), AddrAuth::from_string(&auth)))
            }
            OutputFormat::Json => {
                let mut map: BTreeMap<String, String> = serde_json::from_str(s.as_str()).unwrap();
                Ok((
                    Address::from_string(&map.remove("addr").unwrap()),
                    AddrAuth::from_string(&map.remove("auth").unwrap()),
                ))
            }
        }
    })();

    let quiet = m.get_flag("quiet");

    BaseArgs {
        identity_path: selected_id_path
            .to_str()
            .expect("identity file path was unprintable in UTF-8")
            .to_string(),
        identity_data,
        out_fmt,
        profile,
        quiet,
    }
}

pub struct IdentityFile {
    pub addr: Address,
    pub auth: AddrAuth,
}

pub async fn write_new_identity(new_id: IdentityFile, base_args: BaseArgs) -> Result<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(base_args.identity_path)
        .await?;

    let identity_map = encode_map(
        vec![
            ("addr".to_owned(), new_id.addr.to_string()),
            ("auth".to_owned(), new_id.auth.to_string()),
        ],
        base_args.out_fmt,
    );

    f.write_all(identity_map.as_bytes()).await?;
    Ok(())
}
