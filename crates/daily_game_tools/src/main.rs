mod archive;
mod config;
mod discover;
mod fetch;
mod generate;
mod lockfile;
mod resolver;
mod static_check;
mod validate;

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    ValidateHarnessConfig,
    SyncGames,
    ValidateGames,
    GenerateStaticRegistry,
    PreparePublicAssets,
    CheckStaticOutput {
        #[arg(long)]
        dist: String,
    },
    BuildReport,
    PrepareStaticBuild,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::ValidateHarnessConfig => {
            let _ = config::validate_harness_config("harness.config.json")?;
        }
        Cmd::SyncGames => fetch::sync_games("harness.config.json")?,
        Cmd::ValidateGames => {
            validate::validate_games("harness.config.json")?;
            validate::validate_content("harness.config.json")?;
        }
        Cmd::GenerateStaticRegistry => generate::generate_static_registry("harness.config.json")?,
        Cmd::PreparePublicAssets => generate::prepare_public_assets("harness.config.json")?,
        Cmd::CheckStaticOutput { dist } => static_check::check_static_output(&dist)?,
        Cmd::BuildReport => println!("build-report: ok"),
        Cmd::PrepareStaticBuild => {
            let _ = config::validate_harness_config("harness.config.json")?;
            fetch::sync_games("harness.config.json")?;
            validate::validate_games("harness.config.json")?;
            validate::build_or_verify_games("harness.config.json")?;
            discover::discover_dates("harness.config.json")?;
            validate::validate_content("harness.config.json")?;
            generate::generate_static_registry("harness.config.json")?;
            generate::prepare_public_assets("harness.config.json")?;
        }
    }
    Ok(())
}
