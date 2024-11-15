extern crate env_logger;
use clap::{Parser, Subcommand};
use shadow_rs::shadow;

shadow!(build);

#[derive(Parser, Debug)]
#[clap(
    name = "nodex-agent",
    version = shadow_rs::formatcp!("v{} ({} {})\n{} @ {}", build::PKG_VERSION, build::SHORT_COMMIT, build::BUILD_TIME_3339, build::RUST_VERSION, build::BUILD_TARGET),
    about,
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[clap(flatten)]
    agent_options: agent::cli::AgentOptions,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Controller,
}

fn log_init() {
    let mut builder = env_logger::Builder::from_default_env();
    builder.format(|buf, record| {
        use std::io::Write;
        writeln!(
            buf,
            "{} [{}] - {} - {} - {}:{}",
            chrono::Utc::now().to_rfc3339(),
            record.level(),
            record.target(),
            record.args(),
            record.file().unwrap_or(""),
            record.line().unwrap_or(0),
        )
    });
    builder.init();
}

fn main() {
    std::env::set_var("RUST_LOG", "info");
    log_init();
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Controller) => {
            #[cfg(unix)]
            {
                let _ = controller::run();
            }
            #[cfg(not(unix))]
            {
                log::error!("Controller is not supported on this platform.");
            }
        }
        None => {
            if cli.agent_options.config || cli.agent_options.command.is_some() {
                let _ = agent::run(&cli.agent_options);
            } else {
                let _ = agent::run(&agent::cli::AgentOptions::default());
            }
        }
    }
}
