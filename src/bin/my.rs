use clap::Parser;
use color_eyre::eyre::{anyhow, bail, Result};
use log::warn;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short, long, default_value_t = true)]
    verbose: bool,
    #[clap(short, long, default_value_t = false, conflicts_with = "verbose")]
    quiet: bool,
    #[clap(short, long)]
    profile: Option<String>,
    #[clap(short, long)]
    machine: Option<String>,
    #[clap(short, long, default_value_t=String::from("\n"))]
    separator: String,
    args: Vec<String>,
}

const PROFILE_FILE_NAME: &str = "my.toml";
const MAPPINGS_LABEL: &str = "mappings";
struct EffectiveProfiles {
    default: Option<toml::Table>,
    current: Option<toml::Table>,
}

trait TomlExt {
    fn retrieve_table(
        &mut self,
        key: &str,
    ) -> Option<std::result::Result<toml::Table, toml::de::Error>>;
}

impl TomlExt for toml::Table {
    fn retrieve_table(
        &mut self,
        key: &str,
    ) -> Option<std::result::Result<toml::Table, toml::de::Error>> {
        self.remove(key).map(|v| v.try_into())
    }
}

impl EffectiveProfiles {
    fn read_profile(path: Option<String>) -> Result<toml::Table> {
        let config_path = path
            .map(|x| color_eyre::Result::<PathBuf>::Ok(PathBuf::from(x)))
            .unwrap_or_else(|| {
                Ok(dirs::config_dir()
                    .ok_or_else(|| anyhow!("Can't find config directory!"))?
                    .join("funixtools")
                    .join(PROFILE_FILE_NAME))
            })?;
        let config_file = std::fs::read_to_string(config_path)?;
        let config = toml::from_str(&config_file)?;
        Ok(config)
    }

    fn parse_profile(mut profile: toml::Table, host: &str) -> Result<Self> {
        let default = profile
            .retrieve_table("localhost")
            .transpose()?
            .and_then(|mut t| t.retrieve_table(MAPPINGS_LABEL))
            .transpose()?;
        let current = profile
            .retrieve_table(host)
            .transpose()?
            .and_then(|mut t| t.retrieve_table(MAPPINGS_LABEL))
            .transpose()?;
        Ok(Self { default, current })
    }

    pub fn load(path: Option<String>, host: &str) -> Result<Self> {
        let profile = Self::read_profile(path)?;
        let profile = Self::parse_profile(profile, host)?;
        Ok(profile)
    }

    pub fn get(&self, key: &str) -> Option<&toml::Value> {
        self.current
            .as_ref()
            .and_then(|m| m.get(key))
            .or_else(|| self.default.as_ref().and_then(|m| m.get(key)))
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    color_eyre::install()?;
    env_logger::builder()
        .filter_level(match (cli.verbose, cli.quiet) {
            (false, false) => log::LevelFilter::Warn,
            (true, _) => log::LevelFilter::Debug,
            (_, true) => log::LevelFilter::Off,
        })
        .init();
    // let user = env::var("SUDO_USER")
    //     .or_else(|_| env::var("USER"))
    //     .with_note(|| "Can't determine current user via environment variable!")?;
    let host = nix::unistd::gethostname().unwrap_or_else(|_| {
        warn!("Unable to get hostname! Fallback to localhost.");
        "localhost".into()
    });
    let host = host.to_string_lossy();
    let profile = EffectiveProfiles::load(cli.profile, &host)?;
    if cli.args.is_empty() {
        bail!("Error: no arguments provided!");
    }
    for arg in cli.args {
        let Some(v) = profile.get(&arg) else { bail!("Error: key {:?} not found!", arg)};
        match v {
            toml::Value::String(s) => print!("{}{}", s, &cli.separator),
            _ => bail!("Error: value {:?} for key \"{}\" is not a string!", v, arg),
        }
    }
    Ok(())
}
