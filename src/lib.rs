mod capture;
mod cli;
mod config;
pub mod document;
mod error;
use crate::error::AppError;
pub mod fabric;
mod health;
pub mod mcp;
mod output;
mod policy;
mod proto;
mod transport;
mod tui;
mod valence;

pub(crate) const PROTO_DESCRIPTOR_SET: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/oscal_descriptor.bin"));
pub(crate) const OSCAL_PROTO_FILE_COUNT: &str = env!("OSCAL_PROTO_FILE_COUNT");
pub(crate) const OSCAL_SCHEMA_VERSION: &str = env!("OSCAL_SCHEMA_VERSION");
pub(crate) const OSCAL_SCHEMA_MANIFEST_SHA256: &str = env!("OSCAL_SCHEMA_MANIFEST_SHA256");
pub(crate) const OSCAL_SCHEMA_SOURCE_COMMIT: &str = env!("OSCAL_SCHEMA_SOURCE_COMMIT");
pub(crate) const OSCAL_SCHEMA_RELEASE_ZIP_SHA256: &str = env!("OSCAL_SCHEMA_RELEASE_ZIP_SHA256");

/// Run the read-only OSCAL observer application.
///
/// The application modules and generated protobuf bindings remain private so
/// the library surface cannot be used to reach mutation-capable generated
/// clients. The supported product surface is the executable itself.
pub async fn run() {
    if let Err(error) = run_inner().await {
        eprintln!("error: {}", output::terminal_text(&error.to_string()));
        std::process::exit(1);
    }
}

async fn run_inner() -> error::Result<()> {
    use clap::Parser;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_target(false)
        .compact()
        .init();

    let mut args = config::Cli::parse();
    if let Some(command) = args.command.as_ref() {
        match command {
            config::Command::Completions { shell } => {
                cli::run_completions(*shell)?;
                return Ok(());
            }
            config::Command::Doctor => {
                cli::doctor::run_doctor(&args)?;
                return Ok(());
            }
            _ => {}
        }
        if cli::is_local_command(command) {
            let command = args.command.take().ok_or_else(|| {
                AppError::Configuration("local command missing during dispatch".to_string())
            })?;
            cli::run_local(&args, command)?;
            return Ok(());
        }
    }
    let config = config::AppConfig::from_args(&args)?;

    match args.command {
        None | Some(config::Command::Tui) => tui::run(config).await,
        Some(command) => cli::run(config, command).await,
    }
}
