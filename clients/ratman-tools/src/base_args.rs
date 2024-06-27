use crate::{encode_map, OutputFormat};
use clap::ArgMatches;
use libratman::{
    tokio::{fs::OpenOptions, io::AsyncWriteExt},
    types::{error::UserError, AddrAuth, Address},
    RatmanError, Result,
};
use std::{collections::BTreeMap, env, path::PathBuf};

pub struct BaseArgs {
    pub identity_path: String,
    pub identity_data: Result<(Address, AddrAuth)>,
    pub out_fmt: OutputFormat,
    pub profile: Result<String>,
    pub quiet: bool,
}

pub fn parse_base_args(m: &ArgMatches) -> BaseArgs {
    let output_format = m.get_one::<String>("output-format").unwrap();

    let profile: &String = m.get_one("profile").unwrap();

    let identity_file_path: String = m
        .get_one::<String>("curr-id")
        .map(|x| x.to_string())
        .unwrap_or_else(|| {
            env::var("XDG_CONFIG_HOME")
                .map(|config_home| {
                    PathBuf::new()
                        .join(config_home)
                        .join("ratcat")
                        .join(&profile)
                })
                .expect("Must set XDG_CONFIG_HOME")
                .to_str()
                .unwrap()
                .to_string()
        });

    let out_fmt = match output_format.as_str() {
        "lines" => OutputFormat::Lines,
        "json" => OutputFormat::Json,
        _ => unreachable!(),
    };

    let identity_data = (|| -> Result<(Address, AddrAuth)> {
        let mut f = std::fs::File::open(identity_file_path.clone())?;
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

    let profile = m
        .get_one::<String>("profile")
        .ok_or(RatmanError::User(UserError::MissingInput(
            "operation expected profile to be provided".to_string(),
        )))
        .cloned();

    let quiet = m.get_flag("quiet");

    BaseArgs {
        identity_path: identity_file_path,
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
