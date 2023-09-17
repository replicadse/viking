use {
    args::{
        ClapArgumentLoader,
        Command,
    },
    engine::Engine,
};

include!("check_features.rs");

mod args;
mod config;
mod engine;
mod error;
mod reference;

use {
    anyhow::Result,
    args::ManualFormat,
    std::path::PathBuf,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = ClapArgumentLoader::load()?;

    match cmd.command {
        | Command::Manual { path, format } => {
            let out_path = PathBuf::from(path);
            std::fs::create_dir_all(&out_path)?;
            match format {
                | ManualFormat::Manpages => {
                    reference::build_manpages(&out_path)?;
                },
                | ManualFormat::Markdown => {
                    reference::build_markdown(&out_path)?;
                },
            }
            Ok(())
        },
        | Command::Autocomplete { path, shell } => {
            let out_path = PathBuf::from(path);
            std::fs::create_dir_all(&out_path)?;
            reference::build_shell_completion(&out_path, &shell)?;
            Ok(())
        },
        | Command::Init => {
            println!("{}", include_str!("../res/example.yaml"));
            Ok(())
        },
        | Command::Raid { config, campaign } => {
            let engine = Engine {};
            engine.raid(config.campaigns.get(&campaign).unwrap()).await?;
            Ok(())
        },
    }
}
