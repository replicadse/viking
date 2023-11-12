use {
    crate::{
        config::{
            Config,
            WithVersion,
        },
        error::Error,
    },
    anyhow::Result,
    clap::Arg,
    itertools::Itertools,
    std::str::FromStr,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Privilege {
    Normal,
    Experimental,
}

#[derive(Debug)]
pub(crate) struct CallArgs {
    pub privileges: Privilege,
    pub command: Command,
}

impl CallArgs {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.privileges == Privilege::Experimental {
            return Ok(());
        }

        match &self.command {
            | _ => Ok(()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum ManualFormat {
    Manpages,
    Markdown,
}

#[derive(Debug)]
pub(crate) enum Command {
    Manual {
        path: String,
        format: ManualFormat,
    },
    Autocomplete {
        path: String,
        shell: clap_complete::Shell,
    },

    Init,
    Raid {
        config: Config,
        campaign: String,
        loot: Option<String>,
    },
}

pub(crate) struct ClapArgumentLoader {}

impl ClapArgumentLoader {
    pub(crate) fn root_command() -> clap::Command {
        clap::Command::new("viking")
            .version(env!("CARGO_PKG_VERSION"))
            .about("Raiding APIs in style.")
            .author("replicadse <aw@voidpointergroup.com>")
            .propagate_version(true)
            .subcommand_required(true)
            .args([Arg::new("experimental")
                .short('e')
                .long("experimental")
                .help("Enables experimental features.")
                .num_args(0)])
            .subcommand(
                clap::Command::new("man")
                    .about("Renders the manual.")
                    .arg(clap::Arg::new("out").short('o').long("out").required(true))
                    .arg(
                        clap::Arg::new("format")
                            .short('f')
                            .long("format")
                            .value_parser(["manpages", "markdown"])
                            .required(true),
                    ),
            )
            .subcommand(
                clap::Command::new("autocomplete")
                    .about("Renders shell completion scripts.")
                    .arg(clap::Arg::new("out").short('o').long("out").required(true))
                    .arg(
                        clap::Arg::new("shell")
                            .short('s')
                            .long("shell")
                            .value_parser(["bash", "zsh", "fish", "elvish", "powershell"])
                            .required(true),
                    ),
            )
            .subcommand(clap::Command::new("init").about("Renders and example configuration to STDOUT."))
            .subcommand(
                clap::Command::new("raid")
                    .about("Go on a raid campaign.")
                    .arg(clap::Arg::new("file").short('f').long("file").required(true))
                    .arg(clap::Arg::new("campaign").short('c').long("campaign").required(true))
                    .arg(clap::Arg::new("loot").short('l').long("loot").required(false)),
            )
    }

    pub(crate) fn load() -> Result<CallArgs> {
        let command = Self::root_command().get_matches();

        let privileges = if command.get_flag("experimental") {
            Privilege::Experimental
        } else {
            Privilege::Normal
        };

        let cmd = if let Some(subc) = command.subcommand_matches("man") {
            Command::Manual {
                path: subc.get_one::<String>("out").unwrap().into(),
                format: match subc.get_one::<String>("format").unwrap().as_str() {
                    | "manpages" => ManualFormat::Manpages,
                    | "markdown" => ManualFormat::Markdown,
                    | _ => return Err(Error::Argument("unknown format".into()).into()),
                },
            }
        } else if let Some(subc) = command.subcommand_matches("autocomplete") {
            Command::Autocomplete {
                path: subc.get_one::<String>("out").unwrap().into(),
                shell: clap_complete::Shell::from_str(subc.get_one::<String>("shell").unwrap().as_str()).unwrap(),
            }
        } else if let Some(_) = command.subcommand_matches("init") {
            Command::Init
        } else if let Some(subc) = command.subcommand_matches("raid") {
            let config_path = subc.get_one::<String>("file").unwrap();
            let config_file = std::fs::read_to_string(config_path)?;

            let expected_version = env!("CARGO_PKG_VERSION").split(".").take(2).join(".");
            let config_version = serde_yaml::from_str::<WithVersion>(&config_file)?.version;
            if config_version != expected_version {
                return Err(Error::VersionCompatibility(format!(
                    "version: {} is not supported by CLI {}",
                    config_version, expected_version
                ))
                .into());
            }

            Command::Raid {
                config: serde_yaml::from_str::<Config>(&config_file)?,
                campaign: subc.get_one::<String>("campaign").unwrap().to_owned(),
                loot: subc.get_one::<String>("loot").cloned(),
            }
        } else {
            return Err(Error::UnknownCommand.into());
        };

        let callargs = CallArgs {
            privileges,
            command: cmd,
        };

        callargs.validate()?;
        Ok(callargs)
    }
}
