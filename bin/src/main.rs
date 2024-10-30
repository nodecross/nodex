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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Controller) => {
            controller::run();
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
