//! Wreckit CLI - A tool for turning ideas into automated PRs through an autonomous agent loop

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use wreckit::cli::{Cli, Commands};
use wreckit::errors::to_exit_code;

#[tokio::main]
async fn main() {
    // Initialize tracing
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    let result = run(cli).await;

    match result {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(to_exit_code(&e));
        }
    }
}

async fn run(cli: Cli) -> wreckit::Result<()> {
    match cli.command {
        Some(Commands::Init { force }) => {
            wreckit::cli::commands::init::run(cli.cwd.as_deref(), force, cli.dry_run).await
        }
        Some(Commands::Status { json }) => {
            wreckit::cli::commands::status::run(cli.cwd.as_deref(), json).await
        }
        Some(Commands::List { json, state }) => {
            wreckit::cli::commands::list::run(cli.cwd.as_deref(), json, state.as_deref()).await
        }
        Some(Commands::Show { id, json }) => {
            wreckit::cli::commands::show::run(cli.cwd.as_deref(), &id, json).await
        }
        Some(Commands::Research { id, force }) => {
            wreckit::cli::commands::research::run(cli.cwd.as_deref(), &id, force, cli.dry_run)
                .await
        }
        Some(Commands::Plan { id, force }) => {
            wreckit::cli::commands::plan::run(cli.cwd.as_deref(), &id, force, cli.dry_run).await
        }
        Some(Commands::Implement { id, force }) => {
            wreckit::cli::commands::implement::run(cli.cwd.as_deref(), &id, force, cli.dry_run)
                .await
        }
        Some(Commands::Pr { id, force }) => {
            wreckit::cli::commands::pr::run(cli.cwd.as_deref(), &id, force, cli.dry_run).await
        }
        Some(Commands::Complete { id }) => {
            wreckit::cli::commands::complete::run(cli.cwd.as_deref(), &id, cli.dry_run).await
        }
        Some(Commands::Run { id, force }) => {
            wreckit::cli::commands::run::run(cli.cwd.as_deref(), &id, force, cli.dry_run).await
        }
        Some(Commands::Next) => {
            wreckit::cli::commands::next::run(cli.cwd.as_deref(), cli.dry_run).await
        }
        Some(Commands::Doctor { fix }) => {
            wreckit::cli::commands::doctor::run(cli.cwd.as_deref(), fix).await
        }
        Some(Commands::Ideas { file }) => {
            wreckit::cli::commands::ideas::run(cli.cwd.as_deref(), file.as_deref()).await
        }
        None => {
            // Default to showing help - clap handles this
            println!("Use --help for usage information");
            Ok(())
        }
    }
}
